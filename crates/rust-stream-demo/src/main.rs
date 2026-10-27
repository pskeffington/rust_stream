//! Compact native video wall. All network sources require explicit operator configuration.
use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Vec2};
use rust_stream_core::Quad;
#[cfg(feature = "native-video")]
use rust_stream_core::LIVE_NEWS_CANDIDATES;
#[cfg(feature = "native-video")]
mod native_preview;
#[cfg(feature = "native-video")]
use native_preview::NativePreview;
#[cfg(feature = "native-video")]
use std::sync::{Arc, atomic::{AtomicUsize, Ordering}};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section {
    Security,
    News,
    Traffic,
    FreeTv,
    Custom,
}
struct Viewer {
    quad: Quad,
    section: Section,
    #[cfg(feature = "native-video")]
    previews: [NativePreview; 4],
    #[cfg(feature = "native-video")]
    audio_owner: Arc<AtomicUsize>,
    #[cfg(feature = "native-video")]
    settings: bool,
    #[cfg(feature = "native-video")]
    choices: [usize; 4],
}
impl Default for Viewer {
    fn default() -> Self {
        #[allow(unused_mut)]
        #[cfg(feature = "native-video")]
        let audio_owner = Arc::new(AtomicUsize::new(0));
        let mut viewer = Self {
            quad: Quad::default(),
            section: Section::Security,
            #[cfg(feature = "native-video")]
            previews: std::array::from_fn(|i| NativePreview::new(i, audio_owner.clone())),
            #[cfg(feature = "native-video")]
            audio_owner,
            #[cfg(feature = "native-video")]
            settings: false,
            #[cfg(feature = "native-video")]
            choices: [0; 4],
        };
        #[cfg(feature = "native-video")]
        if std::env::var("RUST_STREAM_TEST_PATTERN").as_deref() == Ok("1") {
            for (i, preview) in viewer.previews.iter_mut().enumerate() {
                preview.start_test();
                viewer.choices[i] = 1;
            }
        }
        viewer
    }
}
const ACCENT: Color32 = Color32::from_rgb(145, 230, 154);
impl Viewer {
    #[cfg(feature = "native-video")]
    fn settings(&mut self, ctx: &egui::Context) {
        egui::Window::new("Sources")
            .open(&mut self.settings)
            .resizable(false)
            .show(ctx, |ui| {
                ui.label("Configure each slot, then press Apply / Restart.");
                ui.label(
                    "HLS sources must be trusted and authorized, including all child resources.",
                );
                const SOURCES: [&str; 6] = [
                    "No source",
                    "Synthetic video",
                    "Configured HLS 1",
                    "Configured HLS 2",
                    "Configured HLS 3",
                    "Configured HLS 4",
                ];
                for i in 0..4 {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}", i + 1));
                        let selected = if self.choices[i] < SOURCES.len() {
                            SOURCES[self.choices[i]].to_string()
                        } else {
                            LIVE_NEWS_CANDIDATES
                                .get(self.choices[i] - SOURCES.len())
                                .map(|c| c.name.to_string())
                                .unwrap_or_else(|| "Unknown source".into())
                        };
                        egui::ComboBox::from_id_salt(("source", i))
                            .selected_text(selected)
                            .show_ui(ui, |ui| {
                                for (n, label) in SOURCES.iter().enumerate() {
                                    ui.selectable_value(&mut self.choices[i], n, *label);
                                }
                                for (n, candidate) in LIVE_NEWS_CANDIDATES.iter().enumerate() {
                                    ui.selectable_value(&mut self.choices[i], SOURCES.len() + n, candidate.name);
                                }
                            });
                        if ui.button("Apply / Restart").clicked() {
                            match self.choices[i] {
                                0 => self.previews[i].stop(),
                                1 => self.previews[i].start_test(),
                                n if n < SOURCES.len() => self.previews[i].start_hls(n - 2),
                                n => self.previews[i].start_candidate(n - SOURCES.len()),
                            }
                        }
                        if ui.button("Remove").clicked() {
                            self.previews[i].stop();
                            self.choices[i] = 0;
                        }
                    });
                }
                ui.label("Set RUST_STREAM_HLS_URL_1 through _4 before launch.");
                ui.label("Curated candidates are unverified and OFF by default.");
                ui.label("Approve exact candidate IDs in RUST_STREAM_APPROVED_FEED_IDS (comma-separated).");
                for candidate in LIVE_NEWS_CANDIDATES {
                    ui.label(format!("{}: {}", candidate.name, candidate.id));
                }
                ui.label("DNS, redirect and nested HLS request enforcement is still pending.");
            });
    }
    fn draw_slot(&mut self, ui: &mut egui::Ui, index: usize, size: Vec2) -> bool {
        let (rect, response) = ui.allocate_exact_size(size, Sense::click());
        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 3.0, Color32::from_rgb(11, 25, 19));
        #[cfg(feature = "native-video")]
        self.previews[index].paint(&painter, rect);
        painter.rect_stroke(rect, 3.0, Stroke::new(1.0, ACCENT), StrokeKind::Inside);
        #[cfg(feature = "native-video")]
        let (label, status) = (
            &self.previews[index].label,
            self.previews[index].status.clone(),
        );
        #[cfg(not(feature = "native-video"))]
        let (label, status) = (
            format!("FEED {:02}", index + 1),
            "NATIVE VIDEO DISABLED".to_string(),
        );
        let font = FontId::monospace(if size.x < 250.0 { 10.0 } else { 13.0 });
        if size.y > 30.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(rect.min, Vec2::new(size.x, 22.0)),
                0.0,
                Color32::from_black_alpha(190),
            );
            painter.text(
                rect.left_top() + Vec2::splat(5.0),
                Align2::LEFT_TOP,
                label,
                font.clone(),
                ACCENT,
            );
        }
        if size.y > 60.0 {
            painter.rect_filled(
                egui::Rect::from_min_size(
                    rect.left_bottom() - Vec2::new(0.0, 22.0),
                    Vec2::new(size.x, 22.0),
                ),
                0.0,
                Color32::from_black_alpha(190),
            );
            painter.text(
                rect.left_bottom() + Vec2::new(5.0, -5.0),
                Align2::LEFT_BOTTOM,
                status,
                font,
                ACCENT,
            );
        }
        response.clicked()
    }
    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.horizontal_wrapped(|ui| {
            ui.strong("MEDIA");
            if self.quad.focused().is_some() && ui.button("GRID").clicked() {
                let _ = self.quad.set_focus(None);
            }
            #[cfg(feature = "native-video")]
            {
                if ui.button("SOURCES").clicked() {
                    self.settings = true;
                }
                ui.separator();
                ui.label("AUDIO");
                for i in 0..4 {
                    let selected = self.audio_owner.load(Ordering::SeqCst) == i;
                    if ui.selectable_label(selected, format!("{}", i + 1)).clicked() {
                        self.audio_owner.store(i, Ordering::SeqCst);
                    }
                }
                if ui.button("STOP ALL").clicked() {
                    for p in &mut self.previews {
                        p.stop();
                    }
                }
            }
        });
        ui.horizontal_wrapped(|ui| {
            for (section, label) in [
                (Section::Security, "SECURITY"),
                (Section::News, "NEWS"),
                (Section::Traffic, "TRAFFIC"),
                (Section::FreeTv, "FREE TV"),
                (Section::Custom, "CUSTOM"),
            ] {
                if ui
                    .selectable_label(self.section == section, label)
                    .clicked()
                {
                    self.section = section;
                    let _ = self.quad.set_focus(None);
                }
            }
        });
    }
    fn wall(&mut self, ui: &mut egui::Ui) {
        let available = ui.available_size().max(Vec2::splat(1.0));
        if let Some(index) = self.quad.focused() {
            if self.draw_slot(ui, index, available) {
                let _ = self.quad.set_focus(None);
            }
        } else {
            let spacing = ui.spacing().item_spacing;
            let size = ((available - spacing) / 2.0).max(Vec2::splat(1.0));
            for row in 0..2 {
                ui.horizontal(|ui| {
                    for col in 0..2 {
                        let index = row * 2 + col;
                        if self.draw_slot(ui, index, size) {
                            let _ = self.quad.set_focus(Some(index));
                        }
                    }
                });
            }
        }
    }
}
impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        #[cfg(feature = "native-video")]
        {
            for preview in &mut self.previews {
                preview.update(ui.ctx());
            }
            self.settings(ui.ctx());
        }
        egui::CentralPanel::default().show(ui, |ui| {
            self.controls(ui);
            // Category catalogs remain pending. The same operator-configured wall is shared.
            self.wall(ui);
        });
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Rust Stream",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 480.0]),
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(Viewer::default()))),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compact_wall_stays_inside_display() {
        for size in [
            egui::vec2(320.0, 240.0),
            egui::vec2(480.0, 320.0),
            egui::vec2(800.0, 480.0),
        ] {
            let ctx = egui::Context::default();
            let mut viewer = Viewer::default();
            let input = egui::RawInput {
                screen_rect: Some(egui::Rect::from_min_size(egui::Pos2::ZERO, size)),
                ..Default::default()
            };
            let mut output = ctx.run_ui(input, |ui| {
                egui::CentralPanel::default().show(ui, |ui| {
                    viewer.controls(ui);
                    viewer.wall(ui);
                    assert!(ui.min_rect().right() <= size.x + 0.1);
                    assert!(ui.min_rect().bottom() <= size.y + 0.1);
                });
            });
            output.textures_delta.clear();
        }
    }
}
