#![allow(unused)]
use std::sync::Arc;

use eframe::{
    egui::{self, Button, Color32, IconData, Stroke, Vec2, WidgetText},
    Theme,
};

pub fn application_icon() -> Option<Arc<IconData>> {
    let icon_data = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory_with_format(icon_data, image::ImageFormat::Png).unwrap();
    let rgba_data = img.into_rgba8();
    let (width, height) = (rgba_data.width(), rgba_data.height());
    let rgba: Vec<u8> = rgba_data.into_raw();
    Some(Arc::<IconData>::new(IconData {
        rgba,
        width,
        height,
    }))
}

pub fn load_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "my_font".to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/MapleMono-NF-CN-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../assets/SourceHanSansCN-VF.ttf")),
        // egui::FontData::from_static(include_bytes!("../assets/SmileySans-Oblique.ttf")),
        // egui::FontData::from_static(include_bytes!("../assets/LXGWWenKaiGBScreenR.ttf")),
        // egui::FontData::from_static(include_bytes!("../assets/LXGWWenKaiMono-Regular.ttf")),
        // egui::FontData::from_static(include_bytes!("../assets/LXGWWenKaiMonoGB-Regular.ttf")),
    );
    fonts
        .families
        .get_mut(&egui::FontFamily::Proportional)
        .unwrap()
        .insert(0, "my_font".to_owned());
    fonts
        .families
        .get_mut(&egui::FontFamily::Monospace)
        .unwrap()
        .push("my_font".to_owned());

    ctx.set_fonts(fonts);
}

pub fn revert_theme(theme: &Theme) -> Theme {
    match theme {
        Theme::Dark => Theme::Light,
        Theme::Light => Theme::Dark,
    }
}

pub fn shadow_frame(theme: &Theme) -> eframe::egui::Frame {
    let (w, d) = (245, 60);
    let fill = match theme {
        Theme::Dark => rgb(d, d, d),
        Theme::Light => rgb(w, w, w),
    };
    shadow_frame_with_fill(fill)
}

pub fn shadow_frame_with_fill(fill: Color32) -> eframe::egui::Frame {
    egui::containers::Frame {
        shadow: eframe::epaint::Shadow {
            color: Color32::from_black_alpha(40),
            offset: Vec2::new(0.0, 2.0),
            blur: 4.0,
            spread: 2.0,
        },
        fill,
        ..Default::default()
    }
    .rounding(8.0)
    .outer_margin(10.0)
    .inner_margin(10.0)
}

pub fn rgb(r: u8, g: u8, b: u8) -> Color32 {
    Color32::from_rgb(r, g, b)
}

pub fn success_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => rgb(50, 186, 64),
        Theme::Light => rgb(60, 222, 76),
    }
}
pub fn success_text(theme: &Theme) -> Color32 {
    // success_color(&revert_theme(theme))
    rgb(238, 255, 235)
}

pub fn primary_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => rgb(106, 52, 199),
        Theme::Light => rgb(123, 57, 237),
    }
}
pub fn primary_text(theme: &Theme) -> Color32 {
    // primary_color(&revert_theme(theme))
    rgb(235, 235, 255)
}

pub fn waring_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => rgb(204, 114, 49),
        Theme::Light => rgb(237, 132, 57),
    }
}
pub fn waring_text(theme: &Theme) -> Color32 {
    // match theme {
    //     Theme::Dark => rgb(240, 212, 170),
    //     Theme::Light => rgb(255, 238, 194),
    // }
    // danger_color(&revert_theme(theme))
    rgb(255, 242, 232)
}

pub fn danger_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => rgb(194, 48, 48),
        Theme::Light => rgb(237, 57, 57),
    }
}
pub fn danger_text(theme: &Theme) -> Color32 {
    // match theme {
    //     Theme::Dark => rgb(255, 125, 125),
    //     Theme::Light => rgb(255, 225, 225),
    // }
    // danger_color(&revert_theme(theme))
    rgb(255, 235, 235)
}

pub fn info_color(theme: &Theme) -> Color32 {
    match theme {
        Theme::Dark => rgb(99, 99, 99),
        Theme::Light => rgb(130, 130, 130),
    }
}
pub fn info_text(theme: &Theme) -> Color32 {
    // info_color(&revert_theme(theme))
    rgb(250, 250, 250)
}

pub fn btn_primary_round<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    btn_primary(text, theme).rounding(100.0)
}
pub fn btn_primary<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    Button::new(WidgetText::RichText(text.into()).color(primary_text(theme)))
        .stroke(Stroke::new(1.0, primary_color(theme)))
        .fill(primary_color(theme))
}

pub fn btn_success_round<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    btn_success(text, theme).rounding(100.0)
}
pub fn btn_success<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    Button::new(WidgetText::RichText(text.into()).color(success_text(theme)))
        .stroke(Stroke::new(1.0, success_color(theme)))
        .fill(success_color(theme))
}

pub fn btn_waring_round<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    btn_waring(text, theme).rounding(100.0)
}
pub fn btn_waring<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    Button::new(WidgetText::RichText(text.into()).color(waring_text(theme)))
        .stroke(Stroke::new(1.0, waring_color(theme)))
        .fill(waring_color(theme))
}

pub fn btn_danger_round<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    btn_danger(text, theme).rounding(100.0)
}
pub fn btn_danger<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    Button::new(WidgetText::RichText(text.into()).color(danger_text(theme)))
        .stroke(Stroke::new(1.0, danger_color(theme)))
        .fill(danger_color(theme))
}

pub fn btn_info_round<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    btn_info(text, theme).rounding(100.0)
}
pub fn btn_info<'a>(text: &'a str, theme: &'a Theme) -> Button<'a> {
    Button::new(WidgetText::RichText(text.into()).color(info_text(theme)))
        .stroke(Stroke::new(1.0, info_color(theme)))
        .fill(info_color(theme))
}
