use eframe::egui::{
    Align2, Context, CursorIcon, Label, Pos2, ScrollArea, TextEdit, TextStyle, Vec2, Widget, Window,
};
use egui_extras::{Size, StripBuilder};

// use std::io::Write;
// use zip::{write::SimpleFileOptions, ZipWriter};

use crate::{
    backup::{self, BackupData},
    data::{FileType, SaveItem},
    entity::{AutoBackup, Command},
    style,
};

pub trait ShowConfirm {
    fn close(&mut self);
    fn show(&mut self);
    fn is_show(&self) -> bool;
}

#[derive(Default)]
pub struct ShowConfirmDelBackup {
    show: bool,
    pub id: String,
    pub pos: Option<Pos2>,
}
impl ShowConfirm for ShowConfirmDelBackup {
    fn close(&mut self) {
        self.show = false;
    }
    fn show(&mut self) {
        self.show = true;
    }
    fn is_show(&self) -> bool {
        self.show
    }
}
impl ShowConfirmDelBackup {
    pub fn new(id: String, pos: Option<Pos2>) -> Self {
        Self {
            show: false,
            id,
            pos,
        }
    }
}

#[derive(Default)]
pub struct ShowConfirmOverwriteBackup {
    show: bool,
    pub datas: Vec<BackupData>,
    pub pos: Option<Pos2>,
}
impl ShowConfirm for ShowConfirmOverwriteBackup {
    fn close(&mut self) {
        self.show = false;
    }
    fn show(&mut self) {
        self.show = true;
    }
    fn is_show(&self) -> bool {
        self.show
    }
}
impl ShowConfirmOverwriteBackup {
    pub fn new(datas: Vec<BackupData>, pos: Option<Pos2>) -> Self {
        Self {
            show: false,
            datas,
            pos,
        }
    }
}

#[derive(Default)]
pub struct ShowConfirmTotoallyDel {
    show: bool,
    pub save_item: SaveItem,
    pub pos: Option<Pos2>,
}
impl ShowConfirm for ShowConfirmTotoallyDel {
    fn close(&mut self) {
        self.show = false;
    }
    fn show(&mut self) {
        self.show = true;
    }
    fn is_show(&self) -> bool {
        self.show
    }
}
impl ShowConfirmTotoallyDel {
    pub fn new(save_item: SaveItem, pos: Option<Pos2>) -> Self {
        Self {
            show: false,
            save_item,
            pos,
        }
    }
}

impl AutoBackup {
    pub fn form_set_backup_remark(&mut self, ctx: &Context) {
        let mut is_open = self.control.show_backup_remark >= 0;
        Window::new(self.t.get("Remark"))
            .title_bar(false)
            .movable(true)
            .open(&mut is_open)
            .collapsible(false)
            .auto_sized()
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                let title = "Please input Remark";

                ui.strong(self.t.get(title));

                if let Ok(mut active) = self.active.try_lock() {
                    if ui
                        .text_edit_multiline(
                            &mut active.backups[self.control.show_backup_remark as usize].remark,
                        )
                        .changed()
                    {
                        if let Ok(mut data) = self.data.try_lock() {
                            data.set_monitor(active.id.to_string(), active.clone());
                        }
                    }
                }

                let theme = &self.setting.get_theme();
                if style::btn_info(self.t.get("Close"), theme)
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.show_backup_remark = -1;
                }
            });
    }

    pub fn check_monitor(&mut self, ctx: &Context) {
        let mut is_open = !self.control.show_check_monitor.is_empty();
        Window::new(self.t.get("Monitor"))
            .title_bar(false)
            .open(&mut is_open)
            .collapsible(false)
            .max_size(Vec2::new(450.0, 450.0))
            .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            .show(ctx, |ui| {
                let title = "The following is a list of files that are being monitored";

                ui.strong(self.t.get(title));

                let body_text_size = TextStyle::Body.resolve(ui.style()).size;
                StripBuilder::new(ui)
                    .size(Size::remainder().at_most(350.0))
                    .size(Size::exact(body_text_size))
                    .vertical(|mut stript| {
                        stript.cell(|ui| {
                            ScrollArea::vertical().show(ui, |ui| {
                                if let Ok(active) = self.active.try_lock() {
                                    for item in &active.monitors {
                                        ui.separator();
                                        ui.horizontal(|ui| {
                                            ui.label(self.t.get(&item.backup_type.to_string()));
                                            if ui
                                                .add(Label::new(item.path.to_string()).wrap(true))
                                                .on_hover_and_drag_cursor(CursorIcon::PointingHand)
                                                .on_hover_text(self.t.get("Open In FileSystem"))
                                                .clicked()
                                            {
                                                let folder = match item.backup_type {
                                                    FileType::Folder => &item.path,
                                                    FileType::File => {
                                                        std::path::Path::new(&item.path)
                                                            .parent()
                                                            .unwrap()
                                                            .to_str()
                                                            .unwrap()
                                                    }
                                                };
                                                let _ = open::that(folder);
                                            }
                                        });
                                    }
                                }
                            });
                        });
                        stript.cell(|ui| {
                            if style::btn_info(self.t.get("Close"), &self.setting.get_theme())
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.control.show_check_monitor = String::new();
                            }
                        });
                    });
            });
    }

    pub fn confirm_overwrite_bakup(&mut self, ctx: &Context) {
        let mut is_open = self.control.show_confirm_overwrite_backup.is_show();

        let mut window = Window::new(self.t.get("confirm_overwrite_bakup"))
            .title_bar(false)
            .open(&mut is_open)
            .collapsible(false)
            // .max_size(Vec2::new(450.0, 450.0))
            .auto_sized()
            // .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            ;

        if self.control.show_confirm_overwrite_backup.is_show() {
            if let Some(pos) = self.control.show_confirm_overwrite_backup.pos {
                window = window.fixed_pos(pos);
            }
        }

        window.show(ctx, |ui| {
            let title = "Confirm to Overwrite?";
            ui.strong(self.t.get(title));
            ui.label(self.t.get("This operation cannot be recovered"));

            ui.horizontal(|ui| {
                if style::btn_waring(self.t.get("Confirm"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    let mut results = vec![];
                    backup::overwrite(
                        &self.control.show_confirm_overwrite_backup.datas,
                        &mut results,
                    );
                    self.package_toasts(results);
                    self.control.show_confirm_overwrite_backup.close();
                }
                if style::btn_info(self.t.get("Cancel"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.show_confirm_overwrite_backup.close();
                }
            });
        });
    }

    pub fn confirm_delete_bakup(&mut self, ctx: &Context) {
        let mut is_open = self.control.show_confirm_del_backup.is_show();

        let mut window = Window::new(self.t.get("confirm_delete_bakup"))
            .title_bar(false)
            .open(&mut is_open)
            .collapsible(false)
            // .max_size(Vec2::new(450.0, 450.0))
            .auto_sized()
            // .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
            ;

        if self.control.show_confirm_del_backup.is_show() {
            // println!("pos = {:?}", self.control.show_confirm_del_backup.pos);
            if let Some(pos) = self.control.show_confirm_del_backup.pos {
                window = window.fixed_pos(pos);
            }
        }

        window.show(ctx, |ui| {
            let title = "Confirm to delete?";
            ui.strong(self.t.get(title));
            ui.label(self.t.get("This operation cannot be recovered"));
            ui.horizontal(|ui| {
                if style::btn_danger(self.t.get("Confirm"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if let Err(_) = self.command.clone().unwrap().send(Command::DeleteBackup {
                        id: self.active.try_lock().unwrap().id.to_string(),
                        backup_id: self.control.show_confirm_del_backup.id.to_string(),
                    }) {
                        self.toasts.error(self.t.get("Delete error"));
                    } else {
                        self.toasts.error(self.t.get("Delete Success"));
                    }
                    self.control.show_confirm_del_backup.close();
                }
                if style::btn_info(self.t.get("Cancel"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.show_confirm_del_backup.close();
                }
            });
        });
    }

    pub fn confirm_totally_delete(&mut self, ctx: &Context) {
        let mut is_open = self.control.show_confirm_totally_del.is_show();

        let mut window = Window::new(self.t.get("confirm_totally_del"))
            .title_bar(false)
            .open(&mut is_open)
            .collapsible(false)
            .auto_sized();

        if self.control.show_confirm_totally_del.is_show() {
            if let Some(pos) = self.control.show_confirm_totally_del.pos {
                window = window.fixed_pos(pos);
            }
        }

        window.show(ctx, |ui| {
            let title = "Confirm to delete?";
            ui.strong(self.t.get(title));
            ui.label(self.t.get("This operation cannot be recovered"));

            ui.label(
                self.control
                    .show_confirm_totally_del
                    .save_item
                    .name
                    .to_string(),
            );
            TextEdit::singleline(&mut self.control.totally_delete_name)
                .hint_text(self.t.get("Please Input the name to confirm delete"))
                .ui(ui);

            ui.horizontal(|ui| {
                if style::btn_danger(self.t.get("Confirm"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if self
                        .control
                        .show_confirm_totally_del
                        .save_item
                        .name
                        .ne(&self.control.totally_delete_name)
                    {
                        self.toasts.error(self.t.get("Delete Error,Name not match"));
                    } else {
                        if let Err(_) = self.command.clone().unwrap().send(Command::TotallyDelete {
                            save_item: self.control.show_confirm_totally_del.save_item.clone(),
                        }) {
                            self.toasts.error(self.t.get("Delete error"));
                        } else {
                            self.toasts.error(self.t.get("Delete Success"));
                        }
                    }

                    self.control.show_confirm_totally_del.close();
                }
                if style::btn_info(self.t.get("Cancel"), &self.setting.get_theme())
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.show_confirm_totally_del.close();
                }
            });
        });
    }
}
