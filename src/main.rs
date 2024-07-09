// hide console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(rustdoc::missing_crate_level_docs)] // it's an example

mod backup;
mod data;
mod entity;
mod il8n;
mod job;
mod log;
mod setting;
mod statis;
mod style;
mod util;
mod view;
mod windows;

use std::sync::mpsc::Receiver;

use eframe::{
    egui::{CentralPanel, Context, Vec2, ViewportBuilder, WindowLevel},
    Frame, HardwareAcceleration,
};
use entity::{AutoBackup, Command, Navigation};
use windows::ShowConfirm;

fn main() -> Result<(), eframe::Error> {
    let mut auto_backup = AutoBackup::default();

    auto_backup.init();

    let viewport = ViewportBuilder {
        title: None,
        app_id: Some("AutoBackup".to_string()),
        position: None,
        inner_size: Some(Vec2::new(1460.0, 640.0)),
        min_inner_size: Some(Vec2::new(260.0, 240.0)),
        max_inner_size: None,
        fullscreen: Some(false),
        maximized: Some(true),
        resizable: Some(true),
        transparent: Some(false),
        decorations: Some(true),
        icon: style::application_icon(),
        active: Some(true),
        visible: Some(true),
        fullsize_content_view: Some(true),
        title_shown: Some(true),
        titlebar_buttons_shown: Some(true),
        titlebar_shown: Some(true),
        drag_and_drop: Some(true),
        taskbar: Some(true),
        close_button: Some(true),
        minimize_button: Some(true),
        maximize_button: Some(true),
        window_level: Some(WindowLevel::Normal),
        mouse_passthrough: Some(false),
        window_type: ViewportBuilder::default().window_type,
    };

    let options = eframe::NativeOptions {
        viewport,
        vsync: true,
        multisampling: 0,
        depth_buffer: 0,
        stencil_buffer: 0,
        hardware_acceleration: HardwareAcceleration::Preferred, // 硬件加速
        renderer: eframe::Renderer::Glow,
        follow_system_theme: false,
        default_theme: auto_backup.setting.get_theme(),
        run_and_return: true, // 关闭窗口退出程序
        event_loop_builder: None,
        window_builder: None,
        shader_version: None,
        centered: true,
        persist_window: true,
    };

    eframe::run_native(
        "Auto Backup | 自动备份",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);
            style::load_fonts(&cc.egui_ctx);
            // 控制缩放
            cc.egui_ctx.set_pixels_per_point(1.5);

            Box::new(auto_backup)
        }),
    )
}

impl eframe::App for AutoBackup {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.control.nav, Navigation::Home, self.t.get("Home"));
                ui.selectable_value(
                    &mut self.control.nav,
                    Navigation::Manage,
                    self.t.get("Manage"),
                );
                if self.form.new {
                    ui.selectable_value(&mut self.control.nav, Navigation::New, self.t.get("New"));
                } else {
                    ui.selectable_value(
                        &mut self.control.nav,
                        Navigation::Edit,
                        self.t.get("Edit"),
                    );
                }
                ui.selectable_value(
                    &mut self.control.nav,
                    Navigation::Archive,
                    self.t.get("Archive"),
                );
                ui.selectable_value(
                    &mut self.control.nav,
                    Navigation::Setting,
                    self.t.get("Setting"),
                );
            });

            match self.control.nav {
                Navigation::Home => self.home_view(ui),
                Navigation::Manage => self.manage_view(ctx, ui),
                Navigation::New => self.new_view(ui),
                Navigation::Edit => self.new_view(ui),
                Navigation::Archive => self.archive_view(ctx, ui),
                Navigation::Setting => self.setting_view(ctx, ui),
            }
        });

        // self.monitor_form(ctx);
        if self.control.show_backup_remark >= 0 {
            self.form_set_backup_remark(ctx);
        }
        if !self.control.show_check_monitor.is_empty() {
            self.check_monitor(ctx);
        }
        if self.control.show_confirm_overwrite_backup.is_show() {
            self.confirm_overwrite_bakup(ctx);
        }
        if self.control.show_confirm_del_backup.is_show() {
            self.confirm_delete_bakup(ctx);
        }
        if self.control.show_confirm_totally_del.is_show() {
            self.confirm_totally_delete(ctx);
        }

        self.toasts.show(ctx);
        // ui.image(egui::include_image!("./assets/icon.ico"));
    }
}

impl AutoBackup {
    fn init(&mut self) {
        self.form.new = true;
        self.control.show_backup_remark = -1;
        self.t.change(&self.setting.lang);

        let (tx, rx) = std::sync::mpsc::channel();
        self.command = Some(tx);
        self.command_proc(rx);
        let _ = self.command.clone().unwrap().send(Command::StartJob);
    }

    fn command_proc(&self, rx: Receiver<Command>) {
        let active = self.active.clone();
        let data = self.data.clone();
        let job_handle = self.job_handle.clone();
        let _ = std::thread::spawn(move || loop {
            if let Ok(commamd) = rx.recv() {
                match commamd {
                    Command::StartJob => {
                        if let Ok(_data) = data.try_lock() {
                            if let Ok(mut job_handle) = job_handle.try_lock() {
                                let job_rx = job_handle.start_job(_data.list());
                                job::receive_data_signal(data.clone(), active.clone(), job_rx);
                                if let Ok(start_infos) = job_handle.start_infos.try_lock() {
                                    
                                    for item in start_infos.to_vec() {
                                        if !item.success {
                                            let error = item.error;
                                            log::log_err(format!("start job error = {:?}", error));
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Command::ProtectBackup { id, protect } => {
                        if let Ok(mut active) = active.try_lock() {
                            for item in &mut active.backups {
                                if item.id == id {
                                    item.protect = protect;
                                    break;
                                }
                            }
                        }
                    }
                    Command::DeleteBackup { id, backup_id } => {
                        let mut save = None;
                        if let Ok(mut active) = active.try_lock() {
                            for (index, item) in active.backups.to_vec().iter().enumerate() {
                                if item.id == backup_id {
                                    active.backups.remove(index);
                                    save = Some(active.clone());
                                    util::delete_dirs(&item.backup_folder);
                                    break;
                                }
                            }
                        }
                        if let Some(active) = save {
                            if let Ok(mut data) = data.try_lock() {
                                data.set_monitor(id.to_string(), active)
                            }
                        }
                    }
                    Command::TotallyDelete { save_item } => {
                        // 根据id删掉对应文件夹
                        println!("{save_item:?}");
                        let current_dir = env!("CARGO_MANIFEST_DIR").replace("\\", "/");
                        let backup_folder =
                            format!("{}/data/backup/{}", current_dir, save_item.id.to_string());
                        util::delete_dirs(&backup_folder);

                        // 提出掉删除的数据，然后存档
                        if let Ok(mut data) = data.try_lock() {
                            for (index, item) in data.monitors.to_vec().iter().enumerate() {
                                if item.id == save_item.id {
                                    data.monitors.remove(index);
                                    data.save();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
    }
}
