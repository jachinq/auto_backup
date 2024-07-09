use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use egui_notify::Toasts;

use crate::data::{Data, SaveItem};
use crate::il8n::Translator;
use crate::job::JobHandle;
use crate::setting::Setting;
use crate::statis::Statis;
use crate::view::Form;
use crate::windows::{ShowConfirmDelBackup, ShowConfirmOverwriteBackup, ShowConfirmTotoallyDel};

pub enum Command {
    StartJob, // 启动备份任务
    ProtectBackup { id: String, protect: bool },
    DeleteBackup { id: String, backup_id: String },
    TotallyDelete { save_item: SaveItem },
}

#[derive(Default)]
pub struct AutoBackup {
    pub control: Control,
    pub statis: Statis,
    pub setting: Setting,
    pub t: Translator,
    pub job_handle: Arc<Mutex<JobHandle>>,
    pub form: Form,
    pub active: Arc<Mutex<SaveItem>>,
    pub data: Arc<Mutex<Data>>,
    pub toasts: Toasts,
    pub command: Option<Sender<Command>>,
}

#[derive(Default, PartialEq, Clone)]
pub enum Navigation {
    #[default]
    Home,
    Manage,
    New,
    Edit,
    Archive,
    Setting,
}

#[derive(Default)]
pub struct Control {
    pub debounce: bool, // 防抖
    pub manage: String,
    pub backup_filter: String,
    pub save_item_filter: String,
    pub totally_delete_name: String, // 彻底删除前的输入确认
    pub nav: Navigation,
    pub show_backup_remark: isize,
    pub show_check_monitor: String, // 检查监听内容
    pub show_confirm_overwrite_backup: ShowConfirmOverwriteBackup, // 二次确认操作
    pub show_confirm_del_backup: ShowConfirmDelBackup, // 二次确认操作
    pub show_confirm_totally_del: ShowConfirmTotoallyDel, // 二次确认操作
    pub new_remark: String,         // 新备份的备注
}
