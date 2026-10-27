//! Compact native video wall. All network sources require explicit operator configuration.
use eframe::egui::{self, Align2, Color32, FontId, Sense, Stroke, StrokeKind, Vec2};
use rust_stream_core::Quad;
#[cfg(feature = "native-video")]
mod native_preview;
#[cfg(feature = "native-video")]
use native_preview::NativePreview;

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
    settings: bool,
    #[cfg(feature = "native-video")]
    choices: [usize; 4],
    #[cfg(feature = "native-video")]
    custom_urls: [String; 4],
    #[cfg(feature = "native-video")]
    custom_names: [String; 4],
    #[cfg(feature = "native-video")]
    editor_open: bool,
    #[cfg(feature = "native-video")]
    editor_slot: usize,
}
impl Default for Viewer {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut viewer = Self {
            quad: Quad::default(),
            section: Section::News,
            #[cfg(feature = "native-video")]
            previews: std::array::from_fn(NativePreview::new),
            #[cfg(feature = "native-video")]
            settings: false,
            #[cfg(feature = "native-video")]
            choices: [0; 4],
            #[cfg(feature = "native-video")]
            custom_urls: std::array::from_fn(|_| String::new()),
            #[cfg(feature = "native-video")]
            custom_names: std::array::from_fn(|i| format!("My stream {}", i + 1)),
            #[cfg(feature = "native-video")]
            editor_open: false,
            #[cfg(feature = "native-video")]
            editor_slot: 0,
        };
        #[cfg(feature = "native-video")]
        if std::env::var("RUST_STREAM_TEST_PATTERN").as_deref() == Ok("1") {
            for (i, preview) in viewer.previews.iter_mut().enumerate() {
                preview.start_test();
                viewer.choices[i] = 7;
            }
        }
        #[cfg(feature = "native-video")]
        if std::env::var("RUST_STREAM_TEST_PATTERN").as_deref() != Ok("1") {
            viewer.load_section();
        }
        viewer
    }
}
#[cfg(feature = "native-video")]
const COMET_HLS: &str = "https://fast-channels.sinclairstoryline.com/COMET/index.m3u8";
#[cfg(feature = "native-video")]
const SOURCES: [&str; 8] = [
    "Off", "COMET TV", "Configured 1", "Configured 2",
    "Configured 3", "Configured 4", "Custom HLS", "Test pattern",
];
#[cfg(feature = "native-video")]
fn configured_name(channel: usize) -> String {
    std::env::var(format!("RUST_STREAM_HLS_NAME_{}", channel + 1))
        .ok().filter(|name| !name.trim().is_empty())
        .unwrap_or_else(|| format!("User feed {}", channel + 1))
}
#[cfg(feature = "native-video")]
fn configured_url(channel: usize) -> Option<String> {
    std::env::var(format!("RUST_STREAM_HLS_URL_{}", channel + 1))
        .or_else(|_| if channel == 0 { std::env::var("RUST_STREAM_HLS_URL") }
                 else { Err(std::env::VarError::NotPresent) })
        .ok().filter(|url| url.starts_with("https://") && !url.trim().is_empty())
}
const ACCENT: Color32 = Color32::from_rgb(145, 230, 154);
impl Viewer {
    #[cfg(feature = "native-video")]
    fn load_section(&mut self) {
        match self.section {
            Section::News => {
                for i in 0..4 {
                    self.choices[i] = if configured_url(i).is_some() { i + 2 } else { 0 };
                    self.apply_source(i);
                }
            }
            Section::FreeTv => {
                self.choices = [1, 0, 0, 0];
                for i in 0..4 { self.apply_source(i); }
            }
            Section::Security | Section::Traffic | Section::Custom => {
                for i in 0..4 {
                    self.choices[i] = 0;
                    self.apply_source(i);
                }
            }
        }
    }
    #[cfg(feature = "native-video")]
    fn apply_source(&mut self, i: usize) {
        match self.choices[i] {
            0 => self.previews[i].stop(),
            1 => self.previews[i].start_url("COMET TV", COMET_HLS),
            2..=5 => self.previews[i].start_hls(self.choices[i] - 2),
            6 => self.previews[i].start_url(&self.custom_names[i], &self.custom_urls[i]),
            7 => self.previews[i].start_test(),
            _ => self.previews[i].stop(),
        }
    }
    #[cfg(feature = "native-video")]
    fn source_picker(&mut self, ui: &mut egui::Ui, i: usize) {
        let previous = self.choices[i];
        egui::ComboBox::from_id_salt(("source", i))
            .width(140.0)
            .selected_text(if (2..=5).contains(&self.choices[i]) {
                configured_name(self.choices[i] - 2)
            } else if self.choices[i] == 6 { self.custom_names[i].clone() } else { SOURCES[self.choices[i]].to_owned() })
            .show_ui(ui, |ui| {
                for (n, label) in SOURCES.iter().enumerate() {
                    if (2..=5).contains(&n) {
                        ui.add_enabled_ui(configured_url(n - 2).is_some(), |ui| {
                            ui.selectable_value(&mut self.choices[i], n, configured_name(n - 2));
                        });
                    } else {
                        ui.selectable_value(&mut self.choices[i], n, *label);
                    }
                }
            });
        if previous != self.choices[i] {
            self.apply_source(i);
        }
    }
    #[cfg(feature = "native-video")]
    fn settings(&mut self, ctx: &egui::Context) {
        let mut open = self.settings;
        egui::Window::new("Streams")
            .open(&mut open)
            .resizable(true)
            .show(ctx, |ui| {
                ui.label("Choose a feed for each box. Changes load immediately.");
                for i in 0..4 {
                    ui.horizontal(|ui| {
                        ui.label(format!("BOX {}", i + 1));
                        self.source_picker(ui, i);
                        if ui.small_button("Restart").clicked() {
                            self.apply_source(i);
                        }
                    });
                    if self.choices[i] == 6 {
                        ui.horizontal(|ui| {
                            ui.label("Name");
                            ui.text_edit_singleline(&mut self.custom_names[i]);
                            ui.label("HTTPS HLS URL");
                            ui.text_edit_singleline(&mut self.custom_urls[i]);
                            if ui.button("Load").clicked() {
                                self.apply_source(i);
                            }
                        });
                    }
                }
                ui.separator();
                ui.small("Custom URLs are session-only and must be trusted/authorized HTTPS HLS manifests.");
                ui.small("Named feeds: RUST_STREAM_HLS_NAME_1..4 and RUST_STREAM_HLS_URL_1..4.");
                ui.small("A validated playlist is not proof of live video or permission to rebroadcast.");
            });
        self.settings = open;
    }
    #[cfg(feature = "native-video")]
    fn inline_stream_editor(&mut self, ui: &mut egui::Ui) {
        if !self.editor_open { return; }
        ui.horizontal(|ui| {
            ui.label("BOX");
            egui::ComboBox::from_id_salt("inline-editor-slot")
                .selected_text(format!("{}", self.editor_slot + 1))
                .show_ui(ui, |ui| {
                    for i in 0..4 {
                        ui.selectable_value(&mut self.editor_slot, i, format!("{}", i + 1));
                    }
                });
            ui.label("Name");
            ui.add(egui::TextEdit::singleline(&mut self.custom_names[self.editor_slot]).desired_width(120.0));
            ui.label("Stream URL");
            let entered = ui.add(egui::TextEdit::singleline(&mut self.custom_urls[self.editor_slot])
                .hint_text("https://publisher.example/live.m3u8")
                .desired_width(240.0));
            let load = ui.button("PLAY").clicked() ||
                (entered.lost_focus() && ui.input(|input| input.key_pressed(egui::Key::Enter)));
            if load {
                let i = self.editor_slot;
                if !self.custom_names[i].trim().is_empty() && !self.custom_urls[i].trim().is_empty() {
                    self.choices[i] = 6;
                    self.apply_source(i);
                }
            }
            if ui.small_button("X").clicked() { self.editor_open = false; }
        });
        ui.small("Trusted HTTPS HLS only. Playback starts in the selected box; unsupported or invalid links stay offline.");
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
                ui.label("BOX 1:");
                self.source_picker(ui, 0);
                if ui.button("+ STREAM").clicked() {
                    self.editor_open = !self.editor_open;
                    self.editor_slot = self.quad.focused().unwrap_or(0);
                }
                if ui.button("STREAMS").clicked() {
                    self.settings = true;
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
                (Section::News, "NEWS"),
                (Section::FreeTv, "FREE TV"),
                (Section::Traffic, "TRAFFIC"),
                (Section::Security, "SECURITY"),
                (Section::Custom, "CUSTOM"),
            ] {
                if ui
                    .selectable_label(self.section == section, label)
                    .clicked()
                {
                    if self.section != section {
                        self.section = section;
                        #[cfg(feature = "native-video")]
                        self.load_section();
                    }
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
            #[cfg(feature = "native-video")]
            self.inline_stream_editor(ui);
            #[cfg(feature = "native-video")]
            if self.section == Section::News && self.choices.iter().all(|choice| *choice == 0) {
                ui.small("No news video configured. Add named HTTPS HLS feeds in your launch environment, or use STREAMS to load a custom feed.");
            }
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
