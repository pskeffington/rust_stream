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
    let compact = size.x < 300.0 || size.y < 140.0;
    let title = slot.source.as_ref().map_or_else(
        || format!("FEED {:02}   NO SOURCE", slot.index + 1),
        |s| format!("FEED {:02}   {}", slot.index + 1, s.label),
    );
    painter.text(rect.left_top() + Vec2::new(6.0, 6.0), Align2::LEFT_TOP,
        if compact { format!("CAM {:02}", slot.index + 1) } else { title },
        FontId::monospace(if compact { 10.0 } else { 14.0 }), accent);
    if rect.height() > 100.0 && rect.width() > 180.0 {
        painter.text(rect.center(), Align2::CENTER_CENTER,
            "AWAITING APPROVED IP FEED", FontId::monospace(12.0),
            Color32::from_rgb(100, 140, 107));
    }
    if rect.height() > 56.0 {
        painter.text(rect.left_bottom() + Vec2::new(12.0, -12.0), Align2::LEFT_BOTTOM,
            format!("{:?}", slot.status).to_uppercase(),
            FontId::monospace(if compact { 9.0 } else { 11.0 }), Color32::from_rgb(100, 150, 116));
    }
    response.clicked()
}

impl eframe::App for Viewer {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 4.0;
                if ui.available_width() >= 440.0 { ui.strong("RUST STREAM / MEDIA"); } else { ui.strong("MEDIA"); }
                ui.menu_button("FREE TV LINKS", |ui| {
                    ui.label("EXTERNAL VIEWING SITES - NOT NATIVE IP FEEDS");
                    for (name, url) in [
                        ("Pluto TV", "https://www.pluto.tv/"),
                        ("Plex Live TV", "https://www.plex.tv/watch-free-tv/"),
                        ("The Roku Channel", "https://therokuchannel.roku.com/"),
                        ("Tubi", "https://tubitv.com/"),
                        ("Samsung TV Plus", "https://www.samsungtvplus.com/"),
                    ] {
                        ui.hyperlink_to(name, url);
                    }
                });
                if self.quad.focused().is_some() && ui.button("RETURN TO GRID").clicked() {
                    let _ = self.quad.set_focus(None);
                }
                if ui.available_width() > 170.0 { ui.label("OFFLINE DEMO"); }
            });
            ui.separator();
            let available = ui.available_size();
            let gap = 4.0;
            // Use the actual central-panel remainder; never allocate beyond the HUD viewport.
            let available = Vec2::new(available.x.max(0.0), available.y.max(0.0));
            if let Some(index) = self.quad.focused() {
                if draw_slot(ui, &self.quad.slots()[index], available, true) {
                    let _ = self.quad.set_focus(None);
                }
            } else {
                let width = ((available.x - gap - ui.spacing().item_spacing.x) / 2.0).max(1.0);
                let height = ((available.y - gap - ui.spacing().item_spacing.y) / 2.0).max(1.0);
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
