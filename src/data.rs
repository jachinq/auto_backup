use serde::{Deserialize, Serialize};

use crate::{backup::Backup, entity::AutoBackup, util};

const PATH: &str = "./data/data.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Data {
    pub monitors: Vec<SaveItem>,
}
impl Default for Data {
    fn default() -> Self {
        if let Ok(data) = util::read_data(PATH) {
            if let Ok(data) = serde_json::from_str::<Data>(&data) {
                data
            } else {
                Self::new()
            }
        } else {
            Self::new()
        }
    }
}
impl Data {
    fn new() -> Self {
        Self {
            monitors: Default::default(),
        }
    }

    pub fn save(&mut self) {
        util::check_dir_and_create("./data");
        let _ = util::write_data(PATH, serde_json::to_string(&self).unwrap());
    }

    pub fn list(&self) -> Vec<SaveItem> {
        self.monitors.to_vec()
    }

    pub fn push_monitor(&mut self, monitor: SaveItem) {
        self.monitors.push(monitor);
        self.save();
    }

    /* pub fn set_status(&mut self, id: String, status: Status) {
           for item in &mut self.monitors {
               if item.id.eq(&id) {
                   item.status = status;
                   self.save();
                   break;
               }
           }
       }

       pub fn set_cron(&mut self, id: String, cron: String) {
           for item in &mut self.monitors {
               if id == item.id {
                   item.cron = cron;
                   self.save();
                   break;
               }
           }
       }
    */
    pub fn set_monitor(&mut self, id: String, monitor: SaveItem) {
        for item in &mut self.monitors {
            if item.id == id {
                // item.path = monitor.path;
                // item.name = monitor.name;
                // item.description = monitor.description;
                // item.cron = monitor.cron;
                // item.status = monitor.status;
                *item = SaveItem { ..monitor };
                self.save();
                break;
            }
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SaveItem {
    pub id: String,
    pub name: String,
    pub description: String,
    pub startup_path: String,
    pub status: Status,
    pub auto: Auto,
    pub monitors: Vec<Monitor>,
    pub backups: Vec<Backup>,
}
impl SaveItem {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            ..Self::default()
        }
    }
}
impl SaveItem {
    pub fn save(self, app: &mut AutoBackup) {
        if self.id.is_empty() {
            return;
        }
        if let Ok(mut data) = app.data.try_lock() {
            data.set_monitor(self.id.to_string(), self)
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Monitor {
    pub path: String,
    pub backup_type: FileType,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub struct Auto {
    pub cron: String,
    pub status: AutoStatus,
    pub open: bool,
}
impl ToString for AutoStatus {
    fn to_string(&self) -> String {
        match self {
            AutoStatus::Running => "Running",
            AutoStatus::Stop => "Stop",
        }
        .to_string()
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum AutoStatus {
    Running,
    #[default]
    Stop,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum Status {
    #[default]
    Valid,
    Archive,
    Delete,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
pub enum FileType {
    #[default]
    Folder,
    File,
}

impl ToString for FileType {
    fn to_string(&self) -> String {
        match self {
            FileType::Folder => "Folder",
            FileType::File => "File",
        }
        .to_string()
    }
}
