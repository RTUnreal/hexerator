use egui_sfml::egui::{self, Layout};
use rand::{thread_rng, RngCore};
use sfml::window::clipboard;

use crate::{
    app::App, damage_region::DamageRegion, msg_if_fail, region::Region, source::Source, ui::Dialog,
};

pub fn top_menu(ui: &mut egui::Ui, app: &mut App, window_height: i16) {
    ui.horizontal(|ui| {
        ui.menu_button("File", |ui| {
            if ui.button("Open").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    msg_if_fail(
                        app.load_file(file, false, window_height),
                        "Failed to load file (read-write)",
                    );
                }
                ui.close_menu();
            }
            if ui.button("Open (read only)").clicked() {
                if let Some(file) = rfd::FileDialog::new().pick_file() {
                    msg_if_fail(
                        app.load_file(file, true, window_height),
                        "Failed to load file (read-only)",
                    );
                }
                ui.close_menu();
            }
            ui.menu_button("Recent", |ui| {
                let mut load = None;
                app.cfg.recent.retain(|entry| {
                    let mut retain = true;
                    ui.horizontal(|ui| {
                        if ui
                            .button(
                                entry
                                    .file
                                    .as_ref()
                                    .map(|path| path.display().to_string())
                                    .unwrap_or_else(|| String::from("Unnamed file")),
                            )
                            .clicked()
                        {
                            load = Some(entry.clone());
                            ui.close_menu();
                        }
                        ui.separator();
                        if ui.button("🗑").clicked() {
                            retain = false;
                        }
                    });
                    ui.separator();
                    retain
                });
                if let Some(args) = load {
                    msg_if_fail(
                        app.load_file_args(args, window_height),
                        "Failed to load file",
                    );
                }
            });
            ui.separator();
            if ui
                .add_enabled(
                    !app.args.read_only && app.dirty_region.is_some(),
                    egui::Button::new("Save (ctrl+S)"),
                )
                .clicked()
            {
                msg_if_fail(app.save(), "Failed to save");
            }
            if ui.add(egui::Button::new("Reload (ctrl+R)")).clicked() {
                msg_if_fail(app.reload(), "Failed to reload");
            }
            ui.separator();
            if ui.button("Create backup").clicked() {
                msg_if_fail(app.create_backup(), "Failed to create backup");
            }
            if ui.button("Restore backup").clicked() {
                msg_if_fail(app.restore_backup(), "Failed to restore backup");
            }
            ui.separator();
            if ui.button("Close").clicked() {
                app.close_file();
                ui.close_menu();
            }
        });
        ui.menu_button("Edit", |ui| {
            if ui.button("Find (ctrl+F)").clicked() {
                app.ui.find_dialog.open ^= true;
                ui.close_menu();
            }
            ui.separator();
            if ui.button("Set select begin to cursor").clicked() {
                match &mut app.selection {
                    Some(sel) => sel.begin = app.edit_state.cursor,
                    None => app.select_begin = Some(app.edit_state.cursor),
                }
            }
            if ui.button("Set select end to cursor").clicked() {
                if let Some(begin) = app.select_begin {
                    match &mut app.selection {
                        None => {
                            app.selection = Some(Region {
                                begin,
                                end: app.edit_state.cursor,
                            })
                        }
                        Some(sel) => sel.end = app.edit_state.cursor,
                    }
                }
            }
            if ui.button("Unselect all").clicked() {
                app.selection = None;
            }
            ui.separator();
            if ui.button("Fill selection with random").clicked() {
                if let Some(sel) = app.selection {
                    let range = sel.begin..=sel.end;
                    thread_rng().fill_bytes(&mut app.data[range.clone()]);
                    app.widen_dirty_region(DamageRegion::RangeInclusive(range));
                }
            }
            if ui.button("Copy selection as hex").clicked() {
                if let Some(sel) = app.selection {
                    use std::fmt::Write;
                    let mut s = String::new();
                    for &byte in &app.data[sel.begin..=sel.end] {
                        write!(&mut s, "{:02x} ", byte).unwrap();
                    }
                    clipboard::set_string(s.trim_end());
                }
            }
            if ui.button("Save selection to file").clicked() && let Some(file_path) = rfd::FileDialog::new().save_file() && let Some(sel) = app.selection {
                let result = std::fs::write(file_path, &app.data[sel.begin..=sel.end]);
                msg_if_fail(result, "Failed to save selection to file");
            }
        });
        ui.menu_button("Seek", |ui| {
            let re = ui
                .button("Set cursor to initial position")
                .on_hover_text("Set to --jump argument, 0 otherwise");
            if re.clicked() {
                app.set_cursor_init();
                ui.close_menu();
            }
            if ui.button("Set cursor position").clicked() {
                ui.close_menu();
                #[derive(Debug, Default)]
                struct SetCursorDialog {
                    offset: usize,
                }
                impl Dialog for SetCursorDialog {
                    fn title(&self) -> &str {
                        "Set cursor"
                    }

                    fn ui(&mut self, ui: &mut egui::Ui, app: &mut App) -> bool {
                        ui.horizontal(|ui| {
                            ui.label("Offset");
                            ui.add(egui::DragValue::new(&mut self.offset));
                        });
                        if ui.input().key_pressed(egui::Key::Enter) {
                            app.edit_state.cursor = self.offset;
                            app.center_view_on_offset(self.offset);
                            false
                        } else {
                            true
                        }
                    }
                }
                app.ui.add_dialog(SetCursorDialog::default());
            }
        });
        ui.menu_button("View", |ui| {
            if ui.button("Configure views...").clicked() {
                app.ui.views_window.open ^= true;
                ui.close_menu();
            }
            if ui.button("Flash cursor").clicked() {
                app.flash_cursor();
                ui.close_menu();
            }
            if ui.button("Center view on cursor").clicked() {
                app.center_view_on_offset(app.edit_state.cursor);
                app.flash_cursor();
                ui.close_menu();
            }
            if ui.button("Set view offset to cursor").clicked() {
                app.perspective.region.begin = app.edit_state.cursor;
            }
            ui.horizontal(|ui| {
                ui.label("Seek to byte offset");
                let re = ui.text_edit_singleline(&mut app.ui.seek_byte_offset_input);
                if re.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                    if let Some(idx) = app.focused_view {
                        app.views[idx].scroll_to_byte_offset(
                            app.ui.seek_byte_offset_input.parse().unwrap_or(0),
                            &app.perspective,
                            app.col_change_lock_x,
                            app.col_change_lock_y,
                        );
                    }
                }
            });
            ui.checkbox(&mut app.col_change_lock_x, "Lock x on column change");
            ui.checkbox(&mut app.col_change_lock_y, "Lock y on column change");
            ui.checkbox(
                &mut app.perspective.flip_row_order,
                "Flip row order (experimental)",
            );
        });
        if ui.button("Regions").clicked() {
            app.ui.regions_window.open ^= true;
        }
        ui.menu_button("Help", |ui| {
            if ui.button("debug panel (F12)").clicked() {
                ui.close_menu();
                gamedebug_core::toggle();
            }
        });
        ui.with_layout(Layout::right_to_left(), |ui| {
            match &app.source {
                Some(src) => match src {
                    Source::File(_) => {
                        match &app.args.file {
                            Some(file) => ui.label(file.display().to_string()),
                            None => ui.label("File path unknown"),
                        };
                    }
                    Source::Stdin(_) => {
                        ui.label("Standard input");
                    }
                },
                None => {
                    ui.label("No source loaded");
                }
            }
            if app.args.stream {
                if app.stream_end {
                    ui.label("[finished stream]");
                } else {
                    ui.spinner();
                    ui.label("[streaming]");
                }
            }
        });
    });
}