use chrono::Local;
use eframe::egui::{
    Align, CentralPanel, Color32, Context, CursorIcon, Label, Layout, RichText, ScrollArea, Sense,
    SidePanel, TextEdit, TextStyle, Ui, Vec2, Widget,
};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
use egui_notify::ToastLevel;
use rfd::FileDialog;

use crate::{
    backup::Backup,
    data::{AutoStatus, FileType, Monitor, SaveItem, Status},
    entity::{AutoBackup, Command, Navigation},
    il8n::Lang,
    job,
    setting::Theme,
    statis::Statis,
    style, util,
    windows::{
        ShowConfirm, ShowConfirmDelBackup, ShowConfirmOverwriteBackup, ShowConfirmTotoallyDel,
    },
};

#[derive(Default)]
pub struct Form {
    pub new: bool,
    open: bool,
    save_item: SaveItem,
}

impl AutoBackup {
    pub fn home_view(&mut self, ui: &mut Ui) {
        let theme = &self.setting.get_theme();
        let col_num = 4;
        let mut detail_card = |ui: &mut Ui, item: &job::StartInfo| {
            let size = Vec2::new(150.0, 20.0);
            let name = item.save_item.name.to_string();
            let name = RichText::new(name).strong();
            let name = Label::new(name).truncate(true);
            if ui
                .add_sized(size, name)
                .on_hover_cursor(CursorIcon::PointingHand)
                .clicked()
            {
                if let Ok(mut active) = self.active.try_lock() {
                    *active = item.save_item.clone();
                }
                self.control.nav = Navigation::Manage;
                self.control.manage = item.save_item.name.to_string();
            }

            let parse_time = job::parse_time(&item.save_item.auto.cron);
            let parse_time = Label::new(parse_time[0].to_string()).truncate(true);
            ui.add_sized(size, parse_time)
                .on_hover_text(self.t.get("Next Start Time"));

            let monitor_label = Label::new(format!(
                "{}: {}{}",
                self.t.get("Monitor Files"),
                item.save_item.monitors.len(),
                self.t.get("number"),
            ));
            let monitor_label = ui.add_sized(size, monitor_label);
            if monitor_label.hovered() {
                let collect: Vec<String> = item
                    .save_item
                    .monitors
                    .to_vec()
                    .into_iter()
                    .map(|f| f.path.to_string())
                    .collect();
                monitor_label
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text(collect.join("\n"));
            }
        };

        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                self.statis = Statis::default();
                if let Ok(data) = self.data.try_lock() {
                    for item in &data.monitors {
                        match item.status {
                            Status::Valid => self.statis.active += 1,
                            Status::Archive => self.statis.archive += 1,
                            _ => {}
                        }
                        self.statis.total += 1;
                    }
                }
                if let Ok(job_hanlde) = self.job_handle.try_lock() {
                    if let Ok(datas) = job_hanlde.start_infos.try_lock() {
                        self.statis.jobs = datas.to_vec();
                        self.statis.auto_backup = datas.len();
                    };
                }

                let card_frame = style::shadow_frame(theme);
                // card
                ui.horizontal_wrapped(|ui| {
                    let color = |r| match r {
                        "Total" => style::primary_color(theme),
                        "Active" => style::rgb(127, 158, 122),
                        "Archive" => style::rgb(158, 153, 122),
                        "Auto Backup" => style::rgb(122, 148, 158),
                        _ => Color32::TRANSPARENT,
                    };
                    for (label, num, _nav) in [
                        ("Total", self.statis.total, Navigation::Home),
                        ("Active", self.statis.active, Navigation::Manage),
                        ("Archive", self.statis.archive, Navigation::Archive),
                        ("Auto Backup", self.statis.auto_backup, Navigation::Manage),
                    ] {
                        let width = 140.0;
                        card_frame.show(ui, |ui| {
                            ui.vertical(|ui| {
                                let color = color(label);
                                let label = RichText::new(self.t.get(&label)).strong();
                                let num = RichText::new(num.to_string()).color(color).size(32.0);
                                let label = Label::new(label).selectable(false);
                                let num = Label::new(num).selectable(false);
                                ui.add_sized(Vec2::new(width, 18.0), label);
                                ui.add_sized(Vec2::new(width, 48.0), num);
                            });
                        });
                    }
                });

                // task
                card_frame.show(ui, |ui| {
                    ui.strong(self.t.get("Auto Backup Tasks"));

                    let (w, d) = (235, 30);
                    let fill = match theme {
                        eframe::Theme::Dark => style::rgb(d, d, d),
                        eframe::Theme::Light => style::rgb(w, w, w),
                    };
                    let line_frame = style::shadow_frame_with_fill(fill).rounding(4.0);
                    ScrollArea::vertical().animated(true).show(ui, |ui| {
                        let mut lines = vec![];
                        lines.push(vec![]);
                        let mut item_index = 0;
                        let mut line_index = 0;
                        for item in &self.statis.jobs {
                            lines[line_index].push(item.clone());
                            if (item_index + 1) % col_num == 0 {
                                line_index += 1;
                                lines.push(vec![]);
                            }
                            item_index += 1;
                        }
                        for line in lines {
                            ui.columns(col_num, |ui| {
                                for (i, item) in line.iter().enumerate() {
                                    let ui: &mut Ui = &mut ui[i];
                                    // ui.vertical_centered_justified(|ui| {
                                    line_frame.clone().show(ui, |ui| {
                                        detail_card(ui, &item);
                                    });
                                    // });
                                }
                            });

                            // println!("{:?}", line);
                            /* ui.with_layout(
                                Layout {
                                    main_dir: eframe::egui::Direction::LeftToRight,
                                    main_wrap: false,
                                    main_align: Align::Center,
                                    main_justify: false,
                                    cross_align: Align::TOP,
                                    cross_justify: false,
                                },
                                |ui| {
                                    for item in line {
                                        // ui.horizontal(|ui| {
                                        line_frame.clone().show(ui, |ui| {
                                            detail_card(ui, &item);
                                        });
                                        // });
                                    }
                                },
                            ); */
                        }
                    });
                });
            })
        });
    }

    pub fn manage_view(&mut self, _ctx: &Context, ui: &mut Ui) {
        let theme = &self.setting.get_theme();
        let goto_new_backup = style::btn_primary_round(self.t.get("New Backup"), theme);
        if let Ok(data) = self.data.try_lock() {
            if data.monitors.is_empty() {
                if goto_new_backup
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.nav = Navigation::New;
                }
                return;
            }
        }

        SidePanel::left("backup_nav")
            // .exact_width(120.0)
            .show_inside(ui, |ui| {
                if goto_new_backup
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    self.control.nav = Navigation::New;
                }
                if let Ok(data) = self.data.try_lock() {
                    ui.separator();
                    ScrollArea::vertical().show(ui, |ui| {
                        for item in &data.monitors {
                            if item.status != Status::Valid {
                                continue;
                            }
                            if ui
                                .selectable_value(
                                    &mut self.control.manage,
                                    item.name.to_string(),
                                    item.name.to_string(),
                                )
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                if let Ok(mut active) = self.active.try_lock() {
                                    *active = item.clone();
                                }
                            }
                            ui.separator();
                        }
                    });
                }
            });

        self.manage_list(ui);
    }

    pub fn new_view(&mut self, ui: &mut Ui) {
        let theme = &self.setting.get_theme();
        CentralPanel::default().show_inside(ui, |ui| {
            let label_size = Vec2::new(80.0, 10.0);
            ui.horizontal(|ui| {
                ui.add_sized(label_size, Label::new(self.t.get("Name")));
                ui.text_edit_singleline(&mut self.form.save_item.name);
            });
            ui.horizontal(|ui| {
                ui.add_sized(label_size, Label::new(self.t.get("Description")));
                ui.text_edit_multiline(&mut self.form.save_item.description);
            });
            ui.horizontal(|ui| {
                ui.add_sized(label_size, Label::new(self.t.get("Startup File")));
                ui.text_edit_singleline(&mut self.form.save_item.startup_path);
                if style::btn_primary_round(self.t.get("Select"), theme)
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if let Some(path) = FileDialog::new().pick_file() {
                        let path = path.as_path().as_os_str().to_str().unwrap().to_string();
                        self.form.save_item.startup_path = path;
                    }
                }
            });

            ui.separator();

            ui.horizontal(|ui| {
                if style::btn_primary_round(self.t.get("Select Folder"), theme)
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if let Some(paths) = FileDialog::new().pick_folders() {
                        for path in paths {
                            let path = path.as_path().as_os_str().to_str().unwrap().to_string();
                            self.form.save_item.monitors.push(Monitor {
                                path,
                                backup_type: FileType::Folder,
                            });
                        }
                    }
                }
                if style::btn_primary_round(self.t.get("Select File"), theme)
                    .ui(ui)
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .clicked()
                {
                    if let Some(paths) = FileDialog::new().pick_files() {
                        for path in paths {
                            let path = path.as_path().as_os_str().to_str().unwrap().to_string();
                            self.form.save_item.monitors.push(Monitor {
                                path,
                                backup_type: FileType::File,
                            });
                        }
                    }
                }
            });

            let body_text_size = TextStyle::Body.resolve(ui.style()).size;
            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::exact(body_text_size))
                .vertical(|mut stript| {
                    stript.cell(|ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            for (index, item) in self
                                .form
                                .save_item
                                .monitors
                                .to_vec()
                                .into_iter()
                                .enumerate()
                            {
                                ui.separator();
                                ui.horizontal(|ui| {
                                    if style::btn_waring(self.t.get("Remove"), theme)
                                    .ui(ui)
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                    {
                                        self.form.save_item.monitors.remove(index);
                                    }
                                    ui.label(self.t.get(&item.backup_type.to_string()));
                                    ui.add(Label::new(item.path.to_string()).wrap(true));
                                });
                            }
                        });
                    });
                    stript.cell(|ui| {
                        ui.horizontal_centered(|ui| {
                            if self.form.new {
                                // new
                                if style::btn_success(self.t.get("Submit"), theme)
                                    .ui(ui)
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    if self.save_item_args_error() {
                                        return;
                                    }
                                    if self.form.save_item.id.is_empty() {
                                        self.form.save_item.id = uuid::Uuid::new_v4().to_string();
                                    }
                                    if self.form.save_item.auto.cron.is_empty() {
                                        self.form.save_item.auto.cron = "0 30 * * * *".to_string();
                                    }
                                    self.control.manage = self.form.save_item.name.to_string();
                                    if let Ok(mut data) = self.data.try_lock() {
                                        data.push_monitor(self.form.save_item.clone());
                                    }
                                    self.form.save_item = SaveItem::default();
                                    self.control.nav = Navigation::Manage;
                                }
                                if style::btn_danger(self.t.get("Reset"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                                {
                                    self.form.save_item = SaveItem::new();
                                }
                            } else {
                                // edit
                                if style::btn_success(self.t.get("Submit"), theme)
                                    .ui(ui)
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .clicked()
                                {
                                    if self.save_item_args_error() {
                                        return;
                                    }
                                    self.form.save_item.clone().save(self);
                                    if let Ok(mut active) = self.active.try_lock() {
                                        *active = self.form.save_item.clone();
                                    }
                                    self.form.new = true;
                                    self.control.nav = Navigation::Manage;
                                    self.control.manage = self.form.save_item.name.to_string();
                                    self.form.save_item = SaveItem::default();
                                }
                                if style::btn_info(self.t.get("Cancel"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                                {
                                    self.form.new = true;
                                    self.control.nav = Navigation::Manage;
                                    self.control.manage = self.form.save_item.name.to_string();
                                    self.form.save_item = SaveItem::default();
                                }
                            }
                        });
                    });
                });
        });
    }

    pub fn archive_view(&mut self, _ctx: &Context, ui: &mut Ui) {
        let theme = &self.setting.get_theme();

        TextEdit::singleline(&mut self.control.save_item_filter)
            .hint_text(self.t.get("Filter by Name or Descirption"))
            .ui(ui);

        let filter_text = &self.control.save_item_filter.to_lowercase();
        ScrollArea::vertical().show(ui, |ui| {
            let mut save = (false, SaveItem::default());
            if let Ok(mut data) = self.data.try_lock() {
                for item in &mut data.monitors {
                    if item.status != Status::Archive {
                        continue;
                    }

                    if !item.name.to_lowercase().contains(filter_text)
                        && !item.description.to_lowercase().contains(filter_text)
                    {
                        continue;
                    }

                    style::shadow_frame(theme).show(ui, |ui| {
                        ui.strong(item.name.to_string())
                            .on_hover_text(item.description.to_string());

                        ui.label(format!("{}: {}", self.t.get("Backup"), item.backups.len()));

                        ui.horizontal(|ui| {
                            if style::btn_success_round(self.t.get("Reactivate"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                item.status = Status::Valid;
                                save.0 = true;
                                save.1 = item.clone();
                            }
                            let totally_del_btn =
                                style::btn_danger_round(self.t.get("Totally Delete"), theme)
                                    .ui(ui)
                                    .on_hover_cursor(CursorIcon::PointingHand);
                            if totally_del_btn.clicked() {
                                self.control.show_confirm_totally_del = ShowConfirmTotoallyDel::new(
                                    item.clone(),
                                    totally_del_btn.interact_pointer_pos(),
                                );
                                self.control.show_confirm_totally_del.show();
                            }
                        });
                    });
                }
            }
            if save.0 {
                save.1.save(self);
            }
        });
    }

    pub fn setting_view(&mut self, ctx: &Context, ui: &mut Ui) {
        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(self.t.get("Theme"));
                    if ui
                        .selectable_value(&mut self.setting.theme, Theme::Dark, self.t.get("Dark"))
                        .changed()
                        || ui
                            .selectable_value(
                                &mut self.setting.theme,
                                Theme::Light,
                                self.t.get("Light"),
                            )
                            .changed()
                    {
                        ctx.set_visuals(self.setting.theme.visuals());
                        self.setting.save();
                    }
                });

                ui.horizontal(|ui| {
                    ui.label(self.t.get("Language"));
                    if ui
                        .selectable_value(&mut self.setting.lang, Lang::En, "English")
                        .changed()
                        || ui
                            .selectable_value(&mut self.setting.lang, Lang::Cn, "简体中文")
                            .changed()
                    {
                        self.t.change(&self.setting.lang);
                        self.setting.save();
                    }
                });
            })
        });
    }

    pub fn package_toasts(&mut self, results: Vec<(ToastLevel, String)>) {
        for (level, msg) in results {
            let msg = self.t.get(&msg);
            match level {
                ToastLevel::Info => {
                    self.toasts.info(msg);
                }
                ToastLevel::Warning => {
                    self.toasts.warning(msg);
                }
                ToastLevel::Error => {
                    self.toasts.error(msg);
                }
                ToastLevel::Success => {
                    self.toasts.success(msg);
                }
                _ => {}
            }
        }
    }

    pub fn manage_list(&mut self, ui: &mut eframe::egui::Ui) {
        if let Ok(active) = self.active.try_lock() {
            if active.id.is_empty() {
                return;
            }
        }

        let theme = &self.setting.get_theme();

        CentralPanel::default().show_inside(ui, |ui| {
            ui.vertical(|ui| {
                style::shadow_frame(theme).show(ui, |ui| {

                    if let Ok(ref mut active) = self.active.try_lock() {
                        ui.horizontal(|ui| {
                            ui.label(self.t.get("Name"));
                            ui.label(active.name.to_string());
                        });
                        ui.horizontal(|ui| {
                            ui.label(self.t.get("Description"));
                            Label::new(active.description.to_string()).wrap(true).ui(ui);
                        });

                        ui.horizontal(|ui| {
                            ui.label(self.t.get("Remark"));
                            TextEdit::singleline(&mut self.control.new_remark)
                            .hint_text(self.t.get("Please enter comments for the new backup"))
                            .ui(ui);
                        });
                    }

                    ui.horizontal(|ui| {
                        if style::btn_primary_round(self.t.get("Backup Now"), theme)
                            .ui(ui)
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .on_hover_text(self.t.get("If the file is too large, there may be stutters, please wait patiently"))
                            .clicked()
                        {
                            if self.control.debounce {
                                self.toasts.warning(self.t.get("Operation is too fast. Try again later."));
                            } else {
                                self.control.debounce = true;
                                let mut cl = SaveItem::default();
                                let mut msg = vec![];
                                if let Ok(mut active) = self.active.try_lock() {
                                    let (msgs, new_backup) = Backup::run(
                                    active.clone(),
                                    self.control.new_remark.to_string(),
                                    );
                                    active.backups.insert(0, new_backup);
                                    cl = active.clone();
                                    msg = msgs;
                                }
                                cl.save(self);
                                self.package_toasts(msg);
                                self.control.debounce = false;
                            }
                        }

                        if let Ok(active) = self.active.try_lock() {
                            if !active.startup_path.is_empty() {
                                if style::btn_success_round(self.t.get("Startup"), theme)
                                    .ui(ui)
                                    .on_hover_cursor(CursorIcon::PointingHand)
                                    .on_hover_text(active.startup_path.to_string())
                                    .clicked()
                                {
                                    let _ = open::that(active.startup_path.to_string());
                                }
                            }

                            if style::btn_primary_round(self.t.get("Open Backup Folder"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                let folder =
                                    format!("{}/data/backup/{}", util::current_dir(), active.id);
                                let _ = open::that(folder);
                            }
                            if style::btn_primary_round(self.t.get("Check Monitor"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.control.show_check_monitor = active.id.to_string();
                            }
                            if style::btn_primary_round(self.t.get("Edit"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                self.form.save_item = active.clone();
                                self.form.new = false;
                                self.form.open = true;
                                self.control.nav = Navigation::Edit;
                            }
                        }

                        if style::btn_waring_round(self.t.get("Archive"), theme)
                        .ui(ui)
                        .on_hover_ui(|ui| {
                            ui.label(self.t.get("When Archived"));
                            ui.label(self.t.get("Will move to archived tab"));
                            ui.label(self.t.get("and Stop the current auto backup task"));
                        })
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .clicked()
                        {
                            let mut save = SaveItem::default();
                            if let Ok(mut active) = self.active.try_lock() {
                                active.status = Status::Archive;
                                active.auto.status = AutoStatus::Stop;                        save = active.clone();
                                *active = SaveItem::default();
                            }
                            save.save(self);
                            let _ = self.command.clone().unwrap().send(Command::StartJob);
                            self.toasts.success(self.t.get("Auto Backup Already Restart"));
                        }
                    });

                    let mut save = false;
                    let mut toggle_auto = false;
                    ui.horizontal(|ui| {
                        if let Ok(mut active) = self.active.try_lock() {
                            // let auto = &mut active.auto;
                        if ui
                            .checkbox(&mut active.auto.open, self.t.get("Use Auto Backup"))
                            .on_hover_cursor(CursorIcon::PointingHand)
                            .clicked()
                        {
                            if !active.auto.open {
                                active.auto.status = AutoStatus::Stop;
                            }
                            // active.clone().save(self);
                            save = true;
                        }

                        if active.auto.open {
                            ui.label(self.t.get("Cron"));

                            let cron = ui.text_edit_singleline(&mut active.auto.cron);
                            if cron.changed() {
                                active.auto.status = AutoStatus::Stop;
                                // active.clone().save(self);
                                save = true;
                                let _ = self.command.clone().unwrap().send(Command::StartJob);
                                self.toasts.success(self.t.get("Auto Backup Already Restart"));
                            }
                            let mut parse_time = job::parse_time(&active.auto.cron);
                            if cron.hovered() {
                                if parse_time.is_empty() {
                                    parse_time = vec![self.t.get("Format Invalid").to_string()];
                                }
                                let tips = format!(
                                    "{}\nsec min hour day_of_month month day_of_week year\n{}\n{}",
                                    self.t.get("Format"),
                                    self.t.get("Next Start Time"),
                                    parse_time.join("\n")
                                );
                                cron.on_hover_text(tips);
                            }

                            let color = match active.auto.status {
                                AutoStatus::Running => {
                                    style::success_color(&self.setting.get_theme())
                                }
                                AutoStatus::Stop => style::danger_color(&self.setting.get_theme()),
                            };
                            ui.colored_label(color, self.t.get(&active.auto.status.to_string()));

                            match active.auto.status {
                                AutoStatus::Stop => {
                                    if style::btn_success(self.t.get("Startup"), theme)
                                        .ui(ui)
                                        .on_hover_cursor(CursorIcon::PointingHand)
                                        .clicked() 
                                    {
                                        if parse_time.is_empty() {
                                            self.toasts.warning(self.t.get("Cron format invalid"));
                                        } else {
                                            active.auto.status = AutoStatus::Running;
                                        }
                                        // active.clone().save(self);
                                        save = true;
                                        toggle_auto = true;
                                    }
                                }
                                AutoStatus::Running => {
                                    if style::btn_info(self.t.get("Stop"), theme)
                                        .ui(ui)
                                        .on_hover_cursor(CursorIcon::PointingHand)
                                        .clicked()
                                    {
                                        active.auto.status = AutoStatus::Stop;
                                        // active.clone().save(self);
                                        save = true;
                                        toggle_auto = true;
                                    }
                                }
                            }
                        }
                        }
                    });

                    if save {
                        let mut save = SaveItem::default();
                        if let Ok(active) = self.active.try_lock() {
                            save = active.clone();
                        }
                        save.save(self);
                    }
                    if toggle_auto {
                        let _ = self.command.clone().unwrap().send(Command::StartJob);
                        self.toasts.success(self.t.get("Auto Backup Already Restart"));
                    }

                    // ui.separator();
                });
                style::shadow_frame(theme).show(ui, |ui| {
                    ScrollArea::vertical().show(ui, |ui| {
                        self.backup_list(ui);
                    });
                });
            });
        });
    }

    pub fn backup_list(&mut self, ui: &mut Ui) {
        let theme = &self.setting.get_theme();

        let mut backups = vec![];
        if let Ok(active) = self.active.try_lock() {
            backups = active.backups.to_vec();
        }

        // let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::auto())
            .column(Column::auto().at_least(130.0))
            .column(Column::auto().clip(true).at_least(160.0))
            .column(Column::auto())
            .column(Column::auto())
            // .min_scrolled_height(0.0)
            // .max_scroll_height(available_height)
            .sense(Sense::click());

        table
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(self.t.get("Row"));
                });
                header.col(|ui| {
                    ui.strong(self.t.get("Bakup Time"));
                });
                header.col(|ui| {
                    ui.strong(self.t.get("Remark"));
                    ui.text_edit_singleline(&mut self.control.backup_filter);
                });
                header.col(|ui| {
                    ui.strong(self.t.get("Protect"));
                });
                header.col(|ui| {
                    ui.strong(self.t.get("Operate"));
                });
            })
            .body(|mut body| {
                // if let Ok( active) = self.active.try_lock() {

                for (index, item) in backups.iter().enumerate() {
                    let filter = &self.control.backup_filter;
                    if !filter.is_empty() && !item.remark.contains(filter) {
                        continue;
                    }
                    body.row(18.0, |mut row| {
                        let mut protect = item.protect;
                        row.set_selected(protect);
                        row.col(|ui| {
                            ui.label((index + 1).to_string());
                        });
                        row.col(|ui: &mut Ui| {
                            let time = if let Some(time) =
                                chrono::DateTime::from_timestamp_millis(item.time)
                            {
                                let fmt = "%Y-%m-%d %H:%M:%S";
                                format!("{}", time.with_timezone(&Local).format(fmt))
                            } else {
                                self.t.get("Unknow").to_string()
                            };
                            ui.label(time);
                        });
                        row.col(|ui| {
                            ui.label(item.remark.to_string())
                                .on_hover_text(item.remark.clone());
                        });
                        row.col(|ui| {
                            if ui.checkbox(&mut protect, "").clicked() {
                                let _ =
                                    self.command.clone().unwrap().send(Command::ProtectBackup {
                                        id: item.id.to_string(),
                                        protect,
                                    });
                            }
                        });
                        row.col(|ui| {
                            let overwrite_btn = style::btn_waring(self.t.get("Overwrite"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand);
                            if overwrite_btn.clicked() {
                                self.control.show_confirm_overwrite_backup =
                                    ShowConfirmOverwriteBackup::new(
                                        // item.id.to_string(),
                                        item.datas.to_vec(),
                                        overwrite_btn.interact_pointer_pos(),
                                    );
                                self.control.show_confirm_overwrite_backup.show();
                            }

                            if style::btn_info(self.t.get("Remark"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                if self.control.show_backup_remark == index as isize {
                                    self.control.show_backup_remark = -1;
                                } else {
                                    self.control.show_backup_remark = index as isize;
                                }
                            }

                            if style::btn_primary(self.t.get("Open"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand)
                                .clicked()
                            {
                                let _ = open::that(&item.backup_folder);
                            }

                            let delete_backup_btn = style::btn_danger(self.t.get("Delete"), theme)
                                .ui(ui)
                                .on_hover_cursor(CursorIcon::PointingHand);
                            if delete_backup_btn.clicked() {
                                if protect {
                                    self.toasts
                                        .warning(self.t.get("Please cancel the protection first"));
                                } else if !self.control.show_confirm_del_backup.is_show() {
                                    self.control.show_confirm_del_backup =
                                        ShowConfirmDelBackup::new(
                                            item.id.to_string(),
                                            delete_backup_btn.interact_pointer_pos(),
                                        );
                                    self.control.show_confirm_del_backup.show();
                                } else {
                                    self.toasts
                                        .warning(self.t.get("Please confirm the operation first"));
                                }
                            }
                        });

                        // if row.response().clicked() {
                        //     mutex_data.selected_index = index;
                        // }
                    });
                }

                // }
            });
    }

    fn save_item_args_error(&mut self) -> bool {
        if self.form.save_item.name.is_empty() {
            self.toasts.warning(self.t.get("Please input name"));
            return true;
        }
        if self.form.save_item.monitors.is_empty() {
            self.toasts
                .warning(self.t.get("Please select monitor file"));
            return true;
        }
        false
    }
}
