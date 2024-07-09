#![allow(unused)]

use std::io::Write;
use std::{fmt::Debug, fs::OpenOptions};

use chrono::Local;

use crate::util;

pub fn log_time(arg: impl Debug) {
    let time = chrono::Local::now();
    println!("{} {:?}", time.format("%m-%d %H:%M:%S%.3f"), arg);
}
pub fn log_info(arg: impl Debug) {
    do_log("info", arg);
}
pub fn log_debug(arg: impl Debug) {
    do_log("debug", arg);
}

pub fn log_err(arg: impl Debug) {
    do_log("error", arg);
}

const TIME_FMT: &str = "%m-%d %H:%M:%S";
fn do_log(level: &str, arg: impl Debug) {
    // let fmt = "%Y年%m月%d日 %H:%M:%S";
    let time = chrono::Local::now();
    let mut arg = format!("[{}] {} {:?}", level, time.format(TIME_FMT), arg);
    if level.eq("info") {
        arg = arg
            .replace("\\\\", "/")
            .replace("\\\"", "")
            .replace("\"", "");
    }

    let fmt = "%Y-%m";
    let now = Local::now().format(fmt);
    let path = format!("./log/{}.txt", now);
    if !util::file_exist(&path) {
        let _create = std::fs::File::create(&path);
    }
    if let Ok(mut file) = OpenOptions::new().append(true).open(path) {
        let _ = writeln!(file, "{}", arg);
    }
}
