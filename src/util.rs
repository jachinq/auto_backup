#![allow(unused)]

use std::{
    env, fs::{self, DirEntry}, io, path::Path, sync::Arc, time::{Duration, SystemTime}
};

use chrono::{DateTime, Datelike, Local, Timelike};
use eframe::egui::{self, IconData};

use crate::{backup::BackupData, log};

// 获取当前程序运行路径
pub fn current_dir() -> String {
    match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(_) => ".".to_string(),
    }
    // env!("CARGO_MANIFEST_DIR").replace("\\", "/")
    // ".".to_string()
    // env::current_dir().unwrap().display().to_string()
}

/// 文件是否存在 可以判断 路径是否存在，文件、文件夹都可以
pub fn file_exist(path: &str) -> bool {
    Path::new(path).exists()
}

/// 检查路径是否存在，不存在则创建路径
pub fn check_dir_and_create(path: &str) {
    if file_exist(path) {
        return;
    }

    if let Err(err) = std::fs::create_dir_all(path) {
        log::log_err(format!("create path {} error: {}", path, err));
    } else {
        log::log_info(format!("create path {} ok", path));
    }
}

/// 读取目录
pub fn read_dir(path: &str) -> Vec<DirEntry> {
    let mut vec = vec![];
    if !file_exist(path) {
        return vec;
    }

    if let Ok(paths) = fs::read_dir(path) {
        for path in paths {
            if let Ok(path) = path {
                log::log_info(format!("init music dir: {}", path.path().display()));
                vec.push(path);
            }
        }
    }
    vec
}

//遍历dir目录，找出修改日期距离当前超过age天的文件名称，存入file_list中
fn visit_dir(dir: &Path, file_list: &mut Vec<String>, age: u64) -> io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dir(&path, file_list, age)?;
            } else {
                let file_matedata = fs::metadata(entry.path())?;
                let modify_time = file_matedata.modified()?;
                if modify_time + Duration::from_secs(age * 24 * 60 * 60) < SystemTime::now() {
                    file_list.push(entry.path().to_str().unwrap().to_string());
                }
            }
        }
    }
    Ok(())
}

//遍历dir目录，找出空目录（内部无文件，无目录）
fn get_empty_dir(dir: &Path, dir_list: &mut Vec<String>) -> io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    let read_dir = fs::read_dir(dir)?;
    let cnt = read_dir.count();
    if cnt == 0 {
        dir_list.push(dir.to_str().unwrap().to_owned());
        return Ok(());
    }

    let read_dir = fs::read_dir(dir)?;
    for entry in read_dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            get_empty_dir(path.as_path(), dir_list)?;
        }
    }
    Ok(())
}

//遍历dir目录，找出空目录（内部无文件，无目录）
pub fn get_files(dir: &String, file_list: &mut Vec<String>) -> io::Result<()> {
    let dir = Path::new(dir);
    if !dir.is_dir() {
        return Ok(());
    }
    let read_dir = fs::read_dir(dir)?;
    let cnt = read_dir.count();
    if cnt == 0 {
        return Ok(());
    }

    let read_dir = fs::read_dir(dir)?;
    for entry in read_dir {
        let path = entry?.path();
        if path.is_dir() {
            let path = path.as_path().to_str().unwrap().to_string();
            file_list.push(path.clone());
            get_files(&path, file_list)?;
        } else {
            let path = path.to_str().unwrap().to_owned();
            file_list.push(path);
        }
    }
    Ok(())
}

pub fn copy_file(source: &str, target: &str) -> io::Result<()> {
    // tar
    // ./data/backup/dsfasd-fac/20240522/b

    //source
    // D:\a\b
    // D:\a\b\a.txt
    // D:\a\b\c.txt
    if Path::new(source).is_dir() {
        check_dir_and_create(source);
        return Ok(());
    }

    // std::io::copy(reader, writer)?;

    Ok(())
}

/// read data from file
pub fn read_data(path: &str) -> Result<String, String> {
    if !file_exist(&path) {
        return Err(format!("file not exist: {}", &path));
    }
    match std::fs::read_to_string(&path) {
        Err(err) => {
            let msg = format!("read file {} error: {}", &path, err);
            log::log_err(msg.to_string());
            Err(msg)
        }
        Ok(data) => Ok(data),
    }
}

/// save data to file
pub fn write_data(path: &str, data: String) -> Result<(), String> {
    match std::fs::write(&path, &data) {
        Err(err) => {
            let msg = format!("write data error {}; path:{}, data:{}", err, &path, &data);
            log::log_err(msg.to_string());
            Err(msg)
        }
        _ => Ok(()),
    }
}

/// cmp time's year,month,day of month,hour,minute,second
pub fn time_eq(t1: &DateTime<Local>, t2: &DateTime<Local>) -> bool {
    // println!("{}, {}", t1.format("%m-%d %H:%M:%S%.3f"), t2.format("%m-%d %H:%M:%S%.3f"));
    t1.year() == t2.year()
        && t1.month() == t2.month()
        && t1.day() == t2.day()
        && t1.hour() == t2.hour()
        && t1.minute() == t2.minute()
        && t1.second() == t2.second()
}

pub fn delete_dirs(path: &str) {
    if file_exist(&path) {
        fs::remove_dir_all(path);
    }
}
pub fn delete_file(path: &str) {
    if file_exist(&path) {
        fs::remove_file(path);
    }
}
