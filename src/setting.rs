#![allow(unused)]

use eframe::egui::Visuals;
use serde::{Deserialize, Serialize};

use crate::{il8n::Lang, util};

const PATH: &str = "./data/setting.json";

#[derive(Serialize, Deserialize)]
pub struct Setting {
    pub left_side_width: f32,
    pub theme: Theme,
    pub lang: Lang,
}
impl Setting {
    pub fn get_theme(&mut self) -> eframe::Theme {
        match self.theme {
            Theme::Dark => eframe::Theme::Dark,
            Theme::Light => eframe::Theme::Light,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Theme {
    Dark,
    Light,
}
impl Theme {
    pub fn visuals(&self) -> Visuals {
        match self {
            Theme::Dark => Visuals::dark(),
            Theme::Light => Visuals::light(),
        }
    }
}

impl Setting {
    fn new() -> Self {
        Self {
            left_side_width: 80.0,
            theme: Theme::Dark,
            lang: Lang::Cn,
        }
    }

    pub fn save(&self) {
        if let Ok(data) = serde_json::to_string(self) {
            util::write_data(&PATH, data);
        }
    }
}
impl Default for Setting {
    fn default() -> Self {
        if let Ok(data) = util::read_data(&PATH) {
            if let Ok(value) = serde_json::from_str::<Setting>(&data) {
                value
            } else {
                Self::new()
            }
        } else {
            Self::new()
        }
    }
}
