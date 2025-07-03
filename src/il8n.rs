#![allow(unused)]

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::util;

pub struct Translator {
    map: HashMap<String, String>,
    lang: Lang,
}

#[derive(Serialize, Deserialize, PartialEq)]
pub enum Lang {
    En,
    Cn,
}
impl Lang {
    fn file(&self) -> String {
        match self {
            Lang::En => "en.lang",
            Lang::Cn => "zh-cn.lang",
        }
        .to_string()
    }
}

fn insert(map: &mut HashMap<String, String>, k: &str, v: &str) {
    map.insert(k.to_string(), v.to_string());
}

impl Default for Translator {
    fn default() -> Self {
        let lang = Lang::Cn;
        Self {
            map: load_data(&lang),
            lang,
        }
    }
}

impl Translator {
    pub fn get<'a>(&'a self, text: &'a str) -> &'a str {
        if let Some(value) = self.map.get(text) {
            if value.is_empty() || value == "" {
                text
            } else {
                value
            }
        } else {
            text
        }
    }

    pub fn change(&mut self, lang: &Lang) {
        self.map = load_data(lang)
    }
}

fn load_data(lang: &Lang) -> HashMap<String, String> {
    let folder = format!("{}/data/lang", util::current_dir());
    util::check_dir_and_create(&folder);
    let path = format!("{}/{}", folder, lang.file());
    let mut map = HashMap::new();
    if let Ok(data) = util::read_data(&path) {
        let mut collect: Vec<(String, String)> = vec![];
        for line in data.split("\r\n") {
            let line: Vec<String> = line.split(":").map(|item| item.to_string()).collect();
            if line.is_empty() {
                continue;
            }
            if line.len() == 1 {
                // collect.push((line[0].to_string(), "".to_string()));
                map.insert(line[0].to_string(), "".to_string());
                continue;
            }
            // collect.push((line[0].to_string(), line[0].to_string()));
            map.insert(line[0].to_string(), line[1].to_string());
        }
    }
    map
}

fn save_data(map: &HashMap<String, String>) {
    let collect: Vec<String> = map
        .keys()
        .map(|key| format!("{}:{}", key, map.get(key).unwrap()))
        .collect();

    let folder = format!("{}/data/lang", util::current_dir());
    util::check_dir_and_create(&folder);
    let path = format!("{}/{}", folder, Lang::En.file());

    util::write_data(&path, collect.join("\n"));

    let path = format!("{}/{}", folder, Lang::Cn.file());
    util::write_data(&path, collect.join("\n"));
}
