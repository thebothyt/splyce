use eframe::egui::{self, Color32, Rounding};

// Studio Dark base: near-black surface with a violet accent
const SURFACE: Color32 = Color32::from_rgb(0x12, 0x10, 0x16);
const SURFACE_HIGH: Color32 = Color32::from_rgb(0x2B, 0x29, 0x30);
const ACCENT: Color32 = Color32::from_rgb(0xCF, 0xBC, 0xFF);

pub fn apply(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();
    visuals.panel_fill = SURFACE;
    visuals.window_fill = SURFACE_HIGH;
    visuals.selection.bg_fill = ACCENT.gamma_multiply(0.45);
    visuals.hyperlink_color = ACCENT;
    visuals.widgets.inactive.bg_fill = SURFACE_HIGH;
    visuals.widgets.inactive.rounding = Rounding::same(12.0);
    visuals.widgets.hovered.rounding = Rounding::same(12.0);
    visuals.widgets.active.rounding = Rounding::same(12.0);
    ctx.set_visuals(visuals);

    // Bigger touch targets than desktop defaults
    ctx.style_mut(|style| {
        style.spacing.interact_size.y = 44.0;
        style.spacing.button_padding = egui::vec2(16.0, 10.0);
    });
}
