use std::sync::{Arc, Mutex};

use egui_notify::ToastLevel;
use fs_more::{directory::DirectoryCopyOptions, file::FileCopyOptions};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// use std::io::Write;
// use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    data::{FileType, SaveItem},
    log, util,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupData {
    pub source: String, // user data path
    pub target: String, // backup path
    pub file_type: FileType,
}
const BACKUP_FOLDER_FMT: &str = "%Y%m%d_%H%M%S";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Backup {
    pub id: String,
    pub datas: Vec<BackupData>, // 备份内容
    pub backup_folder: String,  // 备份所在路径
    pub protect: bool,          // 保护
    pub remark: String,         // 备注
    pub time: i64,
}
impl Backup {
    pub fn new(datas: Vec<BackupData>, backup_folder: String, remark: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            datas,
            backup_folder,
            protect: false,
            remark,
            time: chrono::Local::now().timestamp_millis(),
        }
    }

    pub fn run(save_item: SaveItem, remark: String) -> (Vec<(ToastLevel, String)>, Backup) {
        let monitors = save_item.monitors.to_vec();
        let mut msgs = vec![];
        let backup = backup_file(&save_item.id, remark, monitors, &mut msgs);
        (msgs, backup)
    }
}

pub fn backup_file(
    id: &str,
    remark: String,
    monitors: Vec<crate::data::Monitor>,
    toasts: &mut Vec<(ToastLevel, String)>,
) -> Backup {
    let current_dir = util::current_dir();
    let time = chrono::Local::now();
    let backup_folder = format!(
        "{}/data/backup/{}/{}",
        current_dir,
        id,
        time.format(BACKUP_FOLDER_FMT)
    );
    // println!("f = {}", backup_folder);
    // let target = "./data/a.txt";
    // let target = String::from(target);
    util::check_dir_and_create(&backup_folder);

    let error = Arc::new(Mutex::new(vec![]));
    let mut datas = vec![];
    for item in monitors {
        let backup_source = item.path.clone();
        let arc_error = error.clone();

        let path = std::path::Path::new(&item.path);
        if let Some(file_name) = path.file_name() {
            let target = match item.backup_type {
                FileType::Folder => {
                    let target = format!("{}/{}", backup_folder, file_name.to_str().unwrap());
                    let backup_targer = target.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = fs_more::directory::copy_directory(
                            &backup_source,
                            &backup_targer,
                            DirectoryCopyOptions {
                                target_directory_rule:
                                    fs_more::directory::TargetDirectoryRule::AllowNonEmpty {
                                        overwrite_existing_subdirectories: true,
                                        overwrite_existing_files: true,
                                    },
                                maximum_copy_depth: Some(1000),
                            },
                        ) {
                            log::log_err(format!(
                                "backup error spurce = {}, target = {}, e={:?}",
                                backup_source, backup_targer, e
                            ));
                            arc_error.lock().unwrap().push(backup_source);
                        }
                    });
                    target
                }
                FileType::File => {
                    let target = format!("{}/{}", backup_folder, file_name.to_str().unwrap());
                    let backup_targer = target.clone();
                    std::thread::spawn(move || {
                        if let Err(e) = fs_more::file::copy_file(
                            &backup_source,
                            &backup_targer,
                            FileCopyOptions {
                                overwrite_existing: true,
                                skip_existing: false,
                            },
                        ) {
                            log::log_err(format!(
                                "backup error spurce = {}, target = {}, e={:?}",
                                backup_source, backup_targer, e
                            ));
                            arc_error.lock().unwrap().push(backup_source);
                        }
                    });
                    target
                }
            };
            datas.push(BackupData {
                source: item.path,
                target,
                file_type: item.backup_type,
            });
        }
    }

    let error = error.lock().unwrap();
    if error.is_empty() {
        toasts.push((ToastLevel::Success, "Backup Success".to_string()));
    } else {
        toasts.push((
            ToastLevel::Error,
            format!("Backup Error\n{}", error.join("\n")),
        ));
    }

    Backup::new(datas, backup_folder, remark)
}

pub fn overwrite(datas: &Vec<BackupData>, toasts: &mut Vec<(ToastLevel, String)>) {
    let mut errors = vec![];
    for item in datas {
        let source = item.target.clone();
        let targer = item.source.clone();

        let success = match item.file_type {
            FileType::Folder => {
                if let Err(e) = fs_more::directory::copy_directory(
                    &source,
                    &targer,
                    DirectoryCopyOptions {
                        target_directory_rule:
                            fs_more::directory::TargetDirectoryRule::AllowNonEmpty {
                                overwrite_existing_subdirectories: true,
                                overwrite_existing_files: true,
                            },
                        maximum_copy_depth: Some(1000),
                    },
                ) {
                    log::log_err(format!(
                        "overwrite error source = {}, target = {}, e={:?}",
                        source, targer, e
                    ));
                    false
                } else {
                    true
                }
            }
            FileType::File => {
                if let Err(e) = fs_more::file::copy_file(
                    &source,
                    &targer,
                    FileCopyOptions {
                        overwrite_existing: true,
                        skip_existing: false,
                    },
                ) {
                    log::log_err(format!(
                        "overwrite error source = {}, target = {}, e={:?}",
                        source, targer, e
                    ));
                    false
                } else {
                    true
                }
            }
        };

        if !success {
            errors.push(item.source.to_string());
        }
    }
    if errors.is_empty() {
        toasts.push((ToastLevel::Success, "Overwrite Success".to_string()));
    } else {
        toasts.push((
            ToastLevel::Error,
            format!("Has Error\n{}", errors.join("\n")),
        ));
    };
}
