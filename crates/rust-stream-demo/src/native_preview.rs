//! Explicit opt-in preview for operator-provided, trusted HLS URLs only.
//! GStreamer follows child playlist and segment URLs: no arbitrary URL input or
//! connection-time network policy enforcement is provided in this prototype.
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use rust_stream_core::{GstBackend, MediaBackend, Source, SourceKind, SourceScope};
use std::sync::mpsc::{self, Receiver};
use std::time::Duration;

pub struct NativePreview {
    receiver: Option<Receiver<ColorImage>>,
    texture: Option<TextureHandle>,
    status: String,
}
impl Default for NativePreview {
    fn default() -> Self {
        Self { receiver: None, texture: None, status: "Configure RUST_STREAM_HLS_URL, then press PLAY".into() }
    }
}
impl NativePreview {
    pub fn start(&mut self) {
        self.stop();
        let Ok(endpoint) = std::env::var("RUST_STREAM_HLS_URL") else {
            self.status = "RUST_STREAM_HLS_URL not set".into();
            return;
        };
        if rust_stream_core::validate_direct_hls_url(&endpoint).is_err() {
            self.status = "Invalid public HTTPS HLS URL".into();
            return;
        }
        // Only a locally configured source, never a scraped or catalog endpoint.
        let source = Source { id: "manual-preview".into(), label: "Operator HLS".into(),
            endpoint, kind: SourceKind::Hls, scope: SourceScope::Public, enabled: true };
        let (tx, rx) = mpsc::sync_channel(1);
        std::thread::spawn(move || {
            let backend = GstBackend;
            let Ok(mut session) = backend.open(&source) else { return; };
            loop {
                match session.poll_latest_frame() {
                    Ok(Some(frame)) => {
                        let pixels = ColorImage::from_rgba_unmultiplied(
                            [frame.width as usize, frame.height as usize], &frame.pixels);
                        if tx.send(pixels).is_err() { break; }
                    }
                    Ok(None) => std::thread::sleep(Duration::from_millis(15)),
                    Err(_) => break,
                }
            }
            session.close();
        });
        self.receiver = Some(rx);
        self.status = "Connecting to configured HLS stream".into();
    }
    pub fn stop(&mut self) {
        self.receiver = None;
        self.texture = None;
        self.status = "Stopped".into();
    }
    pub fn show(&mut self, ui: &mut egui::Ui) {
        if let Some(rx) = &self.receiver {
            if let Ok(image) = rx.try_recv() {
                if let Some(texture) = &mut self.texture {
                    texture.set(image, TextureOptions::LINEAR);
                } else {
                    self.texture = Some(ui.ctx().load_texture("stream-preview", image, TextureOptions::LINEAR));
                }
                self.status = "LIVE / DECODING".into();
            } else if matches!(rx.try_recv(), Err(mpsc::TryRecvError::Disconnected)) {
                self.status = "Decoder stopped or stream unavailable".into();
                self.receiver = None;
            }
            ui.ctx().request_repaint_after(Duration::from_millis(33));
        }
        ui.label(&self.status);
        if let Some(texture) = &self.texture {
            let size = ui.available_size();
            let aspect = texture.size_vec2().x / texture.size_vec2().y.max(1.0);
            let width = size.x.min(size.y * aspect);
            ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(width, width / aspect)));
        } else {
            ui.centered_and_justified(|ui| { ui.label("NO DECODED VIDEO"); });
        }
    }
}
