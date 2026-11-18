use eframe::egui::{self, Color32, FontFamily, FontId, Rounding, TextStyle};

pub const SURFACE: Color32 = Color32::from_rgb(0x12, 0x10, 0x16);
pub const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2B, 0x29, 0x30);
pub const ACCENT: Color32 = Color32::from_rgb(0xCF, 0xBC, 0xFF);
pub const ON_ACCENT: Color32 = Color32::from_rgb(0x38, 0x1E, 0x72);
pub const TEXT_DIM: Color32 = Color32::from_rgb(0xAE, 0xA9, 0xB5);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE_HIGH;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.45);
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.inactive.bg_fill = SURFACE_HIGH;
    visuals.widgets.inactive.weak_bg_fill = SURFACE_HIGH;
    visuals.widgets.inactive.rounding = Rounding::same(14.0);
    visuals.widgets.active.rounding = Rounding::same(14.0);

    // Touch has no hover, and egui leaves the last widget "hovered" after a
    // release, which shows up as a stuck white highlight
    visuals.widgets.hovered = visuals.widgets.inactive;

    ctx.set_visuals(visuals);

    ctx.style_mut(|style| {
        style.spacing.interact_size.y = 44.0;
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
        style.spacing.item_spacing = egui::vec2(10.0, 10.0);
        style.text_styles.insert(TextStyle::Heading, FontId::new(30.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Body, FontId::new(16.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Button, FontId::new(16.0, FontFamily::Proportional));
        style.text_styles.insert(TextStyle::Small, FontId::new(12.0, FontFamily::Proportional));
    });
}
