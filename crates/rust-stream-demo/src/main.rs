//! Compact media shell. Native video is explicit opt-in via feature and environment.
use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Vec2};
use rust_stream_core::{FreeTvPlayer, FreeTvService, PlayerStatus, Quad, Slot};
#[cfg(feature = "native-video")]
mod native_preview;
#[cfg(feature = "native-video")]
use native_preview::NativePreview;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Section { Security, News, Traffic, FreeTv, Custom }
struct Viewer { quad: Quad, section: Section, news_choice: usize, free_tv: FreeTvPlayer,
    #[cfg(feature = "native-video")] preview: NativePreview,
}

impl Default for Viewer {
    fn default() -> Self { Self { quad: Quad::default(), section: Section::Security, news_choice: 0, free_tv: FreeTvPlayer::default(), #[cfg(feature = "native-video")] preview: NativePreview::default() } }
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.strong("MEDIA");
                if self.quad.focused().is_some() && ui.small_button("GRID").clicked() {
                    let _ = self.quad.set_focus(None);
                }
            });
            ui.horizontal_wrapped(|ui| {
                for (section, label) in [
                    (Section::Security, "SECURITY"), (Section::News, "NEWS"),
                    (Section::Traffic, "TRAFFIC"), (Section::FreeTv, "FREE TV"),
                    (Section::Custom, "CUSTOM"),
                ] {
                    if ui.selectable_label(self.section == section, label).clicked() {
                        self.section = section;
                        let _ = self.quad.set_focus(None);
                    }
                }
            });
            ui.separator();
            match self.section {
                Section::FreeTv => {
                    // A native player surface; web-service DRM/embedding support is not yet available.
                    // Never pretend a provider webpage is a playable HLS stream.
                    ui.horizontal(|ui| {
                        egui::ComboBox::from_id_salt("free-tv-service")
                            .selected_text(self.free_tv.service().name())
                            .show_ui(ui, |ui| {
                                for service in FreeTvService::ALL {
                                    if ui.selectable_label(self.free_tv.service() == service, service.name()).clicked() {
                                        self.free_tv.select(service);
                                    }
                                }
                            });
                        ui.hyperlink_to("OPEN OFFICIAL SITE", self.free_tv.service().official_site());
                    });
                    let available = ui.available_size();
                    let (rect, _) = ui.allocate_exact_size(available, egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 3.0, Color32::from_rgb(11, 25, 19));
                    painter.rect_stroke(rect, 3.0,
                        Stroke::new(1.0, Color32::from_rgb(75, 119, 80)), StrokeKind::Inside);
                    if rect.width() >= 100.0 && rect.height() >= 56.0 {
                        painter.text(rect.center() + Vec2::new(0.0, -12.0),
                            Align2::CENTER_CENTER, self.free_tv.service().name(),
                            FontId::monospace(15.0), Color32::from_rgb(145, 230, 154));
                        painter.text(rect.center() + Vec2::new(0.0, 16.0),
                            Align2::CENTER_CENTER, match self.free_tv.status() {
                                PlayerStatus::Unconfigured => "NO AUTHORIZED NATIVE STREAM",
                                PlayerStatus::Ready => "SOURCE READY / DECODER PENDING",
                                _ => "NATIVE DECODER NOT CONNECTED",
                            },
                            FontId::monospace(if rect.width() < 360.0 { 9.0 } else { 11.0 }),
                            Color32::from_rgb(100, 150, 116));
                    }
                }
                Section::News => {
                    #[cfg(feature = "native-video")]
                    {
                        ui.horizontal_wrapped(|ui| {
                            if ui.button("PLAY CONFIGURED HLS").clicked() { self.preview.start(); }
                            if ui.button("STOP").clicked() { self.preview.stop(); }
                        });
                        self.preview.show(ui);
                        return;
                    }
                    const CHANNELS: [&str; 4] = ["AL JAZEERA", "DW ENGLISH", "BOSTON 25", "WCVB 5"];
                    ui.horizontal_wrapped(|ui| {
                        for (index, name) in CHANNELS.iter().enumerate() {
                            if ui.selectable_label(self.news_choice == index, *name).clicked() {
                                self.news_choice = index;
                            }
                        }
                    });
                    let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::hover());
                    let painter = ui.painter_at(rect);
                    painter.rect_filled(rect, 3.0, Color32::from_rgb(11, 25, 19));
                    painter.text(rect.center(), Align2::CENTER_CENTER, CHANNELS[self.news_choice],
                        FontId::monospace(15.0), Color32::from_rgb(145, 230, 154));
                    painter.text(rect.left_bottom() + Vec2::new(8.0, -8.0), Align2::LEFT_BOTTOM,
                        "DIRECT VIDEO NOT VERIFIED / OFFLINE", FontId::monospace(10.0),
                        Color32::from_rgb(100, 150, 116));
                }
                Section::Security | Section::Traffic | Section::Custom => {
                    let available = ui.available_size();
                    let gap = 4.0;
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
                                        Vec2::new(width, height), false) {
                                        let _ = self.quad.set_focus(Some(index));
                                    }
                                    if col == 0 { ui.add_space(gap); }
                                }
                            });
                            if row == 0 { ui.add_space(gap); }
                        }
                    }
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
