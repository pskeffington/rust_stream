//! Quad-layout shell only. This demo never opens a network connection.
//! The production media adapter will provide frame textures through rust-stream-core.
use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Vec2};
use rust_stream_core::{Quad, Slot};

struct Viewer { quad: Quad }

impl Default for Viewer {
    fn default() -> Self { Self { quad: Quad::default() } }
}

fn draw_slot(ui: &mut egui::Ui, slot: &Slot, size: Vec2, focused: bool) -> bool {
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    let painter = ui.painter_at(rect);
    let bg = Color32::from_rgb(11, 25, 19);
    let accent = if focused { Color32::from_rgb(145, 230, 154) }
        else { Color32::from_rgb(72, 122, 88) };
    painter.rect_filled(rect, 4.0, bg);
    painter.rect_stroke(rect, 4.0, Stroke::new(1.0, accent), StrokeKind::Inside);
    let title = slot.source.as_ref().map_or_else(
        || format!("FEED {:02}   NO SOURCE", slot.index + 1),
        |s| format!("FEED {:02}   {}", slot.index + 1, s.label),
    );
    painter.text(rect.left_top() + Vec2::new(12.0, 12.0), Align2::LEFT_TOP,
        title, FontId::monospace(14.0), accent);
    if rect.height() > 76.0 {
        painter.text(rect.center(), Align2::CENTER_CENTER,
            "AWAITING APPROVED IP FEED", FontId::monospace(12.0),
            Color32::from_rgb(100, 140, 107));
    }
    if rect.height() > 40.0 {
        painter.text(rect.left_bottom() + Vec2::new(12.0, -12.0), Align2::LEFT_BOTTOM,
            format!("{:?}  /  CLICK TO FOCUS", slot.status).to_uppercase(),
            FontId::monospace(11.0), Color32::from_rgb(100, 150, 116));
    }
    response.clicked()
}

impl eframe::App for Viewer {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("RUST STREAM  /  QUAD VIEW");
                if self.quad.focused().is_some() && ui.button("RETURN TO GRID").clicked() {
                    let _ = self.quad.set_focus(None);
                }
                ui.label("DEMO SHELL  -  NO NETWORK");
            });
            ui.separator();
            let available = ui.available_size();
            let gap = 6.0;
            if let Some(index) = self.quad.focused() {
                if draw_slot(ui, &self.quad.slots()[index], available, true) {
                    let _ = self.quad.set_focus(None);
                }
            } else {
                let width = ((available.x - gap) / 2.0).max(1.0);
                let height = ((available.y - gap) / 2.0).max(1.0);
                for row in 0..2 {
                    ui.horizontal(|ui| {
                        for col in 0..2 {
                            let index = row * 2 + col;
                            if draw_slot(ui, &self.quad.slots()[index],
                                Vec2::new(width, height), false)
                            {
                                let _ = self.quad.set_focus(Some(index));
                            }
                            if col == 0 { ui.add_space(gap); }
                        }
                    });
                    if row == 0 { ui.add_space(gap); }
                }
            }
        });
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Rust Stream",
        eframe::NativeOptions::default(),
        Box::new(|_cc| Ok(Box::new(Viewer::default()))),
    )
}
