use eframe::egui;
use egui::{FontId, Stroke, TextStyle};

fn gray(v: u8) -> egui::Color32 {
    egui::Color32::from_gray(v)
}

fn rgb(r: u8, g: u8, b: u8) -> egui::Color32 {
    egui::Color32::from_rgb(r, g, b)
}

const STROKE_W: f32 = 1.0;

pub fn apply(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();

    style.text_styles.insert(TextStyle::Heading, FontId::proportional(13.0));
    style.text_styles.insert(TextStyle::Body, FontId::proportional(12.5));
    style.text_styles.insert(TextStyle::Button, FontId::proportional(12.0));
    style.text_styles.insert(TextStyle::Small, FontId::proportional(10.5));

    style.spacing.item_spacing = egui::vec2(6.0, 3.0);
    style.spacing.button_padding = egui::vec2(7.0, 2.5);
    style.spacing.menu_margin = egui::Margin::symmetric(4.0, 3.0);
    style.spacing.indent = 14.0;

    let v = &mut style.visuals;

    v.panel_fill = gray(0x2a);
    v.window_fill = gray(0x33);
    v.extreme_bg_color = gray(0x1d);
    v.faint_bg_color = gray(0x31);
    v.override_text_color = Some(gray(0xc6));

    v.widgets.noninteractive.bg_fill = gray(0x2a);
    v.widgets.noninteractive.fg_stroke = Stroke::new(STROKE_W, gray(0x8f));
    v.widgets.noninteractive.bg_stroke = Stroke::new(STROKE_W, gray(0x20));

    v.widgets.inactive.bg_fill = gray(0x3a);
    v.widgets.inactive.fg_stroke = Stroke::new(STROKE_W, gray(0xc6));
    v.widgets.inactive.bg_stroke = Stroke::NONE;

    v.widgets.hovered.bg_fill = gray(0x44);
    v.widgets.hovered.fg_stroke = Stroke::new(STROKE_W, gray(0xe0));
    v.widgets.hovered.bg_stroke = Stroke::new(STROKE_W, gray(0x55));

    v.widgets.active.bg_fill = gray(0x4d);
    v.widgets.active.fg_stroke = Stroke::new(STROKE_W, gray(0xff));

    v.widgets.open.bg_fill = gray(0x36);

    v.selection.bg_fill = rgb(0x2f, 0x5d, 0x8a);
    v.selection.stroke = Stroke::new(STROKE_W, rgb(0x51, 0x84, 0xb8));

    v.window_rounding = egui::Rounding::same(3.0);
    v.window_stroke = Stroke::new(STROKE_W, gray(0x11));
    v.popup_shadow = egui::epaint::Shadow::NONE;

    let widget_rounding = egui::Rounding::same(2.0);
    v.widgets.noninteractive.rounding = widget_rounding;
    v.widgets.inactive.rounding = widget_rounding;
    v.widgets.hovered.rounding = widget_rounding;
    v.widgets.active.rounding = widget_rounding;
    v.widgets.open.rounding = widget_rounding;

    ctx.set_style(style);
}

pub fn panel_title(ui: &mut egui::Ui, title: &str) {
    ui.add_space(1.0);
    ui.label(egui::RichText::new(title).strong().small());
    ui.separator();
}

