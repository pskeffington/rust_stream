//! Egui adapter for one independently controlled native player.
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use rust_stream_core::{GstBackend, Playback, PlaybackStatus, Source, SourceKind, SourceScope};
use std::sync::Arc;
use std::time::Duration;

pub struct NativePreview {
    player: Playback,
    texture: Option<TextureHandle>,
    pub status: String,
    pub label: String,
    pub frames: u64,
    id: usize,
}
impl NativePreview {
    pub fn new(id: usize) -> Self {
        Self {
            player: Playback::default(),
            texture: None,
            status: "OFFLINE".into(),
            label: format!("FEED {:02}", id + 1),
            frames: 0,
            id,
        }
    }
    pub fn start_test(&mut self) {
        let backend = GstBackend::test_pattern(self.id);
        self.label = backend.source().label.clone();
        let source = backend.source().clone();
        self.texture = None;
        self.player.start(Arc::new(backend), source);
    }
    pub fn start_url(&mut self, label: &str, endpoint: &str) {
        self.stop();
        let source = Source {
            id: format!("selected-hls-{}", self.id),
            label: label.to_owned(),
            endpoint: endpoint.trim().to_owned(),
            kind: SourceKind::Hls,
            scope: SourceScope::Public,
            enabled: true,
        };
        match GstBackend::trusted_hls(source.clone()) {
            Ok(backend) => {
                self.label = source.label.clone();
                self.player.start(Arc::new(backend), source);
            }
            Err(_) => self.status = "INVALID HTTPS HLS URL".into(),
        }
    }
    pub fn start_hls(&mut self, channel: usize) {
        let variable = format!("RUST_STREAM_HLS_URL_{}", channel + 1);
        let endpoint = std::env::var(&variable).or_else(|_| {
            if channel == 0 {
                std::env::var("RUST_STREAM_HLS_URL")
            } else {
                Err(std::env::VarError::NotPresent)
            }
        });
        self.stop();
        let Ok(endpoint) = endpoint else {
            self.status = format!("Set {variable}");
            return;
        };
        self.start_url(&format!("CONFIGURED HLS {}", channel + 1), &endpoint);
    }
    pub fn stop(&mut self) {
        self.player.stop();
        self.texture = None;
        self.status = "OFFLINE".into();
        self.frames = 0;
    }
    pub fn update(&mut self, ctx: &egui::Context) {
        let snapshot = self.player.poll();
        self.frames = snapshot.frames_decoded;
        // Keep configuration errors visible until a new action is taken.
        if snapshot.status != PlaybackStatus::Offline || self.status == "OFFLINE" {
            self.status = format!("{:?}", snapshot.status).to_uppercase();
        }
        if let Some(frame) = snapshot.frame {
            let image = ColorImage::from_rgba_unmultiplied(
                [frame.width as usize, frame.height as usize],
                &frame.pixels,
            );
            if let Some(texture) = &mut self.texture {
                texture.set(image, TextureOptions::LINEAR);
            } else {
                self.texture = Some(ctx.load_texture(
                    format!("stream-{}", self.id),
                    image,
                    TextureOptions::LINEAR,
                ));
            }
        }
        if snapshot.status != PlaybackStatus::Offline {
            ctx.request_repaint_after(Duration::from_millis(33));
        }
    }
    pub fn paint(&self, painter: &egui::Painter, rect: egui::Rect) {
        if let Some(texture) = &self.texture {
            let size = fit_video(texture.size_vec2(), rect.size());
            painter.image(
                texture.id(),
                egui::Rect::from_center_size(rect.center(), size),
                egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1.0, 1.0)),
                egui::Color32::WHITE,
            );
        }
    }
}
fn fit_video(video: egui::Vec2, available: egui::Vec2) -> egui::Vec2 {
    video * (available.x.max(0.0) / video.x).min(available.y.max(0.0) / video.y)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn preserves_aspect_and_fits_compact_bounds() {
        for available in [
            egui::vec2(150.0, 60.0),
            egui::vec2(1.0, 1.0),
            egui::vec2(320.0, 240.0),
        ] {
            let fitted = fit_video(egui::vec2(640.0, 360.0), available);
            assert!(fitted.x <= available.x && fitted.y <= available.y);
            assert!((fitted.x / fitted.y - 16.0 / 9.0).abs() < 0.001);
        }
    }
}
