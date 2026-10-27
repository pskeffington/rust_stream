//! In-process GStreamer HLS decoder. Feature-gated for systems with GStreamer installed.
use gstreamer as gst;
use gstreamer_app as gst_app;
use gst::prelude::*;
use crate::{FrameRgba,MediaBackend,MediaSession,Source,SourceKind,StreamError};
pub struct GstBackend;
pub struct GstSession { pipeline: gst::Pipeline, sink: gst_app::AppSink }
impl Drop for GstSession { fn drop(&mut self) { let _=self.pipeline.set_state(gst::State::Null); } }
impl MediaBackend for GstBackend {
 fn open(&self, source:&Source)->Result<Box<dyn MediaSession>,StreamError>{
  if !source.enabled || source.kind!=SourceKind::Hls { return Err(StreamError::MediaBackend); }
  gst::init().map_err(|_|StreamError::MediaBackend)?;
  let pipeline=gst::Pipeline::new();
  let decoder=gst::ElementFactory::make("uridecodebin").property("uri",&source.endpoint).build().map_err(|_|StreamError::MediaBackend)?;
  let converter=gst::ElementFactory::make("videoconvert").build().map_err(|_|StreamError::MediaBackend)?;
  let scaler=gst::ElementFactory::make("videoscale").build().map_err(|_|StreamError::MediaBackend)?;
  let caps=gst::Caps::builder("video/x-raw").field("format","RGBA").field("width",640i32).field("height",360i32).build();
  let filter=gst::ElementFactory::make("capsfilter").property("caps",&caps).build().map_err(|_|StreamError::MediaBackend)?;
  let sink=gst_app::AppSink::builder().caps(&caps).max_buffers(1).drop(true).sync(false).build();
  pipeline.add_many([&decoder,&converter,&scaler,&filter,sink.upcast_ref()]).map_err(|_|StreamError::MediaBackend)?;
  gst::Element::link_many([&converter,&scaler,&filter,sink.upcast_ref()]).map_err(|_|StreamError::MediaBackend)?;
  let target=converter.static_pad("sink").ok_or(StreamError::MediaBackend)?;
  decoder.connect_pad_added(move |_,pad|{
   if pad.current_caps().and_then(|c|c.structure(0).map(|s|s.name().starts_with("video/"))).unwrap_or(false) && !target.is_linked(){let _=pad.link(&target);}
  });
  pipeline.set_state(gst::State::Playing).map_err(|_|StreamError::MediaBackend)?;
  Ok(Box::new(GstSession{pipeline,sink}))
 }
}
impl MediaSession for GstSession {
 fn poll_latest_frame(&mut self)->Result<Option<FrameRgba>,StreamError>{
  let Some(sample)=self.sink.try_pull_sample(gst::ClockTime::ZERO) else{return Ok(None)};
  let buffer=sample.buffer().ok_or(StreamError::MediaBackend)?;
  let map=buffer.map_readable().map_err(|_|StreamError::MediaBackend)?;
  let frame=FrameRgba::new(640,360,map.as_slice().to_vec(),buffer.pts().map_or(0,|t|t.mseconds()))?;
  Ok(Some(frame))
 }
 fn close(&mut self){let _=self.pipeline.set_state(gst::State::Null);}
}
