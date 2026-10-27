//! Trusted, operator-configured HLS and network-free synthetic video.
use crate::{FrameRgba, MediaBackend, MediaSession, Source, SourceKind, SourceScope, StreamError};
use gst::prelude::*;
use gst_video::prelude::*;
use gstreamer as gst;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

/// This adapter does not constrain GStreamer's nested requests or DNS/redirects.
/// Construct only for a publisher and all child resources the operator trusts.
/// The exact approved source is bound to the backend and checked again at open.
pub struct GstBackend {
    source: Source,
    pattern: Option<&'static str>,
}
impl GstBackend {
    pub fn trusted_hls(source: Source) -> Result<Self, StreamError> {
        if !source.enabled
            || source.kind != SourceKind::Hls
            || source.scope != SourceScope::Public
            || source.endpoint.len() > 2048
            || source.endpoint.chars().any(char::is_control)
            || crate::validate_direct_hls_url(&source.endpoint).is_err()
        {
            return Err(StreamError::MediaBackend);
        }
        let host = url::Url::parse(&source.endpoint)
            .map_err(|_| StreamError::MediaBackend)?
            .host_str()
            .ok_or(StreamError::MediaBackend)?
            .trim_end_matches('.')
            .to_string();
        source.validate_definition(&crate::SourcePolicy {
            allowed_hosts: [host].into(),
            ..Default::default()
        })?;
        Ok(Self {
            source,
            pattern: None,
        })
    }
    /// Built-in moving patterns exercise the same appsink/render path without network access.
    pub fn test_pattern(index: usize) -> Self {
        Self {
            source: Source {
                id: format!("test-{index}"),
                label: format!("TEST PATTERN {}", index + 1),
                endpoint: String::new(),
                kind: SourceKind::Hls,
                scope: SourceScope::Public,
                enabled: true,
            },
            pattern: Some(["smpte", "ball", "snow", "smpte100"][index % 4]),
        }
    }
    pub fn source(&self) -> &Source {
        &self.source
    }
}
impl MediaBackend for GstBackend {
    fn open(&self, source: &Source) -> Result<Box<dyn MediaSession>, StreamError> {
        if source != &self.source {
            return Err(StreamError::MediaBackend);
        }
        open_pipeline(self.pattern, &source.endpoint)
    }
}
struct GstSession {
    pipeline: gst::Pipeline,
    sink: gst_app::AppSink,
}
impl Drop for GstSession {
    fn drop(&mut self) {
        self.close();
    }
}
fn element(name: &str) -> Result<gst::Element, StreamError> {
    gst::ElementFactory::make(name)
        .build()
        .map_err(|_| StreamError::MediaBackend)
}
fn open_pipeline(
    pattern: Option<&'static str>,
    uri: &str,
) -> Result<Box<dyn MediaSession>, StreamError> {
    gst::init().map_err(|_| StreamError::MediaBackend)?;
    let pipeline = gst::Pipeline::new();
    let decoder = if let Some(pattern) = pattern {
        gst::ElementFactory::make("videotestsrc")
            .property("is-live", true)
            .property_from_str("pattern", pattern)
            .build()
            .map_err(|_| StreamError::MediaBackend)?
    } else {
        gst::ElementFactory::make("uridecodebin3")
            .property("uri", uri)
            // Discard audio rather than create an unconsumed audio branch.
            .property(
                "caps",
                gst::Caps::builder("video/x-raw").any_features().build(),
            )
            .build()
            .map_err(|_| StreamError::MediaBackend)?
    };
    if pattern.is_none() {
        decoder.connect("select-stream", false, |values| {
            let video = values[2]
                .get::<gst::Stream>()
                .map(|stream| stream.stream_type().contains(gst::StreamType::VIDEO))
                .unwrap_or(false);
            Some(i32::from(video).to_value())
        });
        decoder.connect("source-setup", false, |values| {
            if let Ok(source) = values[1].get::<gst::Element>() {
                if source.find_property("timeout").is_some() {
                    source.set_property("timeout", 10u32);
                }
            }
            None
        });
    }
    let converter = element("videoconvert")?;
    let scaler = element("videoscale")?;
    let rate = element("videorate")?;
    rate.set_property("drop-only", true);
    let caps = gst::Caps::builder("video/x-raw")
        .field("format", "RGBA")
        .field("width", 640i32)
        .field("height", 360i32)
        .field("pixel-aspect-ratio", gst::Fraction::new(1, 1))
        .field("framerate", gst::Fraction::new(15, 1))
        .build();
    let sink = gst_app::AppSink::builder()
        .caps(&caps)
        .max_buffers(1)
        .drop(true)
        .sync(true)
        .wait_on_eos(false)
        .build();
    // Own the pipeline before starting it, so every error path tears it down.
    let session = GstSession { pipeline, sink };
    session
        .pipeline
        .add_many([
            &decoder,
            &converter,
            &scaler,
            &rate,
            session.sink.upcast_ref(),
        ])
        .map_err(|_| StreamError::MediaBackend)?;
    gst::Element::link_many([&converter, &scaler, &rate, session.sink.upcast_ref()])
        .map_err(|_| StreamError::MediaBackend)?;
    if pattern.is_some() {
        decoder
            .link(&converter)
            .map_err(|_| StreamError::MediaBackend)?;
    } else {
        let target = converter
            .static_pad("sink")
            .ok_or(StreamError::MediaBackend)?;
        decoder.connect_pad_added(move |_, pad| {
            let caps = pad.current_caps().unwrap_or_else(|| pad.query_caps(None));
            if caps
                .structure(0)
                .is_some_and(|s| s.name().starts_with("video/"))
                && !target.is_linked()
            {
                let _ = pad.link(&target);
            }
        });
    }
    session
        .pipeline
        .set_state(gst::State::Playing)
        .map_err(|_| StreamError::MediaBackend)?;
    Ok(Box::new(session))
}
impl MediaSession for GstSession {
    fn poll_latest_frame(&mut self) -> Result<Option<FrameRgba>, StreamError> {
        if let Some(bus) = self.pipeline.bus() {
            // State/diagnostic messages must be drained as well, keeping the bus bounded.
            for message in bus.iter() {
                match message.view() {
                    gst::MessageView::Error(_) => return Err(StreamError::MediaBackend),
                    gst::MessageView::Eos(_) => return Err(StreamError::EndOfStream),
                    _ => {}
                }
            }
        }
        let Some(sample) = self.sink.try_pull_sample(gst::ClockTime::ZERO) else {
            return Ok(None);
        };
        let info = gst_video::VideoInfo::from_caps(sample.caps().ok_or(StreamError::InvalidFrame)?)
            .map_err(|_| StreamError::InvalidFrame)?;
        if info.format() != gst_video::VideoFormat::Rgba
            || info.width() > 640
            || info.height() > 360
        {
            return Err(StreamError::InvalidFrame);
        }
        let buffer = sample.buffer().ok_or(StreamError::InvalidFrame)?;
        let mapped = gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info)
            .map_err(|_| StreamError::InvalidFrame)?;
        let stride =
            usize::try_from(mapped.plane_stride()[0]).map_err(|_| StreamError::InvalidFrame)?;
        FrameRgba::from_strided(
            info.width(),
            info.height(),
            stride,
            mapped
                .plane_data(0)
                .map_err(|_| StreamError::InvalidFrame)?,
            buffer.pts().map_or(0, |t| t.mseconds()),
        )
        .map(Some)
    }
    fn close(&mut self) {
        let _ = self.pipeline.set_state(gst::State::Null);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};
    #[test]
    fn synthetic_pipeline_decodes_and_closes_repeatedly() {
        for index in 0..4 {
            let backend = GstBackend::test_pattern(index);
            let mut session = backend.open(backend.source()).unwrap();
            let deadline = Instant::now() + Duration::from_secs(5);
            let mut timestamps = Vec::new();
            while timestamps.len() < 3 && Instant::now() < deadline {
                if let Some(frame) = session.poll_latest_frame().unwrap() {
                    assert_eq!(
                        (frame.width, frame.height, frame.pixels.len()),
                        (640, 360, 640 * 360 * 4)
                    );
                    timestamps.push(frame.pts_ms);
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            assert_eq!(timestamps.len(), 3);
            assert!(timestamps.windows(2).all(|t| t[1] > t[0]));
            session.close();
            session.close();
        }
    }
    #[test]
    fn requires_exact_approved_source() {
        let backend = GstBackend::test_pattern(0);
        let mut source = backend.source().clone();
        source.endpoint = "https://unapproved.example/live.m3u8".into();
        assert!(backend.open(&source).is_err());
        assert!(GstBackend::trusted_hls(source.clone()).is_ok());
        source.endpoint = "https://127.0.0.1/live.m3u8".into();
        assert!(GstBackend::trusted_hls(source).is_err());
    }
}

#[cfg(test)]
#[path = "gst_tests.rs"]
mod integration_tests;
