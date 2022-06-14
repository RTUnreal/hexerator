use egui_inspect::inspect;
use egui_sfml::{
    egui::{
        self, Button, ComboBox, DragValue, Layout, ScrollArea, TextEdit, TopBottomPanel, Window,
    },
    SfEgui,
};
use gamedebug_core::{per_msg, Info, PerEntry, IMMEDIATE, PERSISTENT};

use crate::{
    app::{App, Source},
    color::ColorMethod,
    msg_if_fail,
    slice_ext::SliceExt,
    InteractMode, Region,
};

#[expect(
    clippy::significant_drop_in_scrutinee,
    reason = "this isn't a useful lint for for loops"
)]
// https://github.com/rust-lang/rust-clippy/issues/8987
pub fn do_egui(sf_egui: &mut SfEgui, mut app: &mut App) {
    sf_egui.do_frame(|ctx| {
        let mut open = app.show_debug_panel;
        Window::new("Debug").open(&mut open).show(ctx, |ui| {
            inspect! {
                ui,
                app
            }
            ui.separator();
            ui.heading("More Debug");
            for info in IMMEDIATE.lock().unwrap().iter() {
                if let Info::Msg(msg) = info {
                    ui.label(msg);
                }
            }
            gamedebug_core::clear_immediates();
            ui.separator();
            for PerEntry { frame, info } in PERSISTENT.lock().unwrap().iter() {
                if let Info::Msg(msg) = info {
                    ui.label(format!("{}: {}", frame, msg));
                }
            }
        });
        app.show_debug_panel = open;
        open = app.find_dialog.open;
        Window::new("Find").open(&mut open).show(ctx, |ui| {
            if ui
                .text_edit_singleline(&mut app.find_dialog.input)
                .lost_focus()
                && ui.input().key_pressed(egui::Key::Enter)
            {
                let needle = app.find_dialog.input.parse().unwrap();
                app.find_dialog.result_offsets.clear();
                for (offset, &byte) in app.data.iter().enumerate() {
                    if byte == needle {
                        app.find_dialog.result_offsets.push(offset);
                    }
                }
                if let Some(&off) = app.find_dialog.result_offsets.first() {
                    app.search_focus(off);
                }
            }
            ScrollArea::vertical().max_height(480.).show(ui, |ui| {
                for (i, &off) in app.find_dialog.result_offsets.iter().enumerate() {
                    let re =
                        ui.selectable_label(app.find_dialog.result_cursor == i, off.to_string());
                    if let Some(scroll_off) = app.find_dialog.scroll_to && scroll_off == i {
                        re.scroll_to_me(None);
                        app.find_dialog.scroll_to = None;
                    }
                    if re.clicked() {
                        app.search_focus(off);
                        app.find_dialog.result_cursor = i;
                        break;
                    }
                }
            });
            ui.horizontal(|ui| {
                ui.set_enabled(!app.find_dialog.result_offsets.is_empty());
                if (ui.button("Previous (P)").clicked() || ui.input().key_pressed(egui::Key::P))
                    && app.find_dialog.result_cursor > 0
                {
                    app.find_dialog.result_cursor -= 1;
                    let off = app.find_dialog.result_offsets[app.find_dialog.result_cursor];
                    app.search_focus(off);
                    app.find_dialog.scroll_to = Some(app.find_dialog.result_cursor);
                }
                ui.label((app.find_dialog.result_cursor + 1).to_string());
                if (ui.button("Next (N)").clicked() || ui.input().key_pressed(egui::Key::N))
                    && app.find_dialog.result_cursor + 1 < app.find_dialog.result_offsets.len()
                {
                    app.find_dialog.result_cursor += 1;
                    let off = app.find_dialog.result_offsets[app.find_dialog.result_cursor];
                    app.search_focus(off);
                    app.find_dialog.scroll_to = Some(app.find_dialog.result_cursor);
                }
                ui.label(format!("{} results", app.find_dialog.result_offsets.len()));
            });
        });
        app.find_dialog.open = open;
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Open").clicked() {
                        if let Some(file) = rfd::FileDialog::new().pick_file() {
                            msg_if_fail(app.load_file(file), "Failed to load file");
                        }
                        ui.close_menu();
                    }
                    if ui.button("Close").clicked() {
                        app.close_file();
                        ui.close_menu();
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
                });
                ui.menu_button("View", |ui| {
                    if ui.button("Flash cursor").clicked() {
                        app.flash_cursor();
                        ui.close_menu();
                    }
                    if ui.button("Center view on cursor").clicked() {
                        app.center_view_on_offset(app.cursor);
                        app.flash_cursor();
                        ui.close_menu();
                    }
                    ui.horizontal(|ui| {
                        ui.label("Seek to byte offset");
                        let re = ui.text_edit_singleline(&mut app.seek_byte_offset_input);
                        if re.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                            app.set_view_to_byte_offset(
                                app.seek_byte_offset_input.parse().unwrap_or(0),
                            );
                        }
                    });
                    ui.checkbox(&mut app.col_change_lock_x, "Lock x on column change");
                    ui.checkbox(&mut app.col_change_lock_y, "Lock y on column change");
                });
                ui.with_layout(Layout::right_to_left(), |ui| {
                    match &app.source {
                        Some(src) => match src {
                            Source::File(_) => {
                                match &app.args.file {
                                    Some(file) => match file.canonicalize() {
                                        Ok(path) => ui.label(path.display().to_string()),
                                        Err(e) => ui.label(format!("path error: {}", e)),
                                    },
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
                        ui.label("[stream]");
                    }
                });
            });
            ui.horizontal(|ui| {
                let begin_text = match app.select_begin {
                    Some(begin) => begin.to_string(),
                    None => "-".to_owned(),
                };
                ui.label(format!("Select begin: {}", begin_text));
                if ui.button("set").clicked() {
                    match &mut app.selection {
                        Some(sel) => sel.begin = app.cursor,
                        None => app.select_begin = Some(app.cursor),
                    }
                }
                let end_text = match app.selection {
                    Some(sel) => sel.end.to_string(),
                    None => "-".to_owned(),
                };
                ui.label(format!("end: {}", end_text));
                if ui.button("set").clicked() {
                    match app.select_begin {
                        Some(begin) => match &mut app.selection {
                            None => {
                                app.selection = Some(Region {
                                    begin,
                                    end: app.cursor,
                                })
                            }
                            Some(sel) => sel.end = app.cursor,
                        },
                        None => {}
                    }
                }
                if ui.button("deselect").clicked() {
                    app.selection = None;
                }
                ui.text_edit_singleline(&mut app.fill_text);
                if ui.button("fill").clicked() {
                    if let Some(sel) = app.selection {
                        let values: Result<Vec<u8>, _> = app
                            .fill_text
                            .split(' ')
                            .map(|token| u8::from_str_radix(token, 16))
                            .collect();
                        match values {
                            Ok(values) => {
                                app.data[sel.begin..=sel.end].pattern_fill(&values);
                                app.widen_dirty_region(sel.begin, Some(sel.end));
                            }
                            Err(e) => {
                                per_msg!("Fill parse error: {}", e);
                            }
                        }
                    }
                }
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.checkbox(&mut app.invert_color, "invert");
                    ComboBox::new("color_combo", "Color")
                        .selected_text(app.color_method.name())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Default,
                                ColorMethod::Default.name(),
                            );
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Mono,
                                ColorMethod::Mono.name(),
                            );
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Rgb332,
                                ColorMethod::Rgb332.name(),
                            );
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Vga13h,
                                ColorMethod::Vga13h.name(),
                            );
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Grayscale,
                                ColorMethod::Grayscale.name(),
                            );
                            ui.selectable_value(
                                &mut app.color_method,
                                ColorMethod::Aitd,
                                ColorMethod::Aitd.name(),
                            );
                        });
                    ui.color_edit_button_rgb(&mut app.bg_color);
                    ui.label("Bg color");
                });
            });
        });
        TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui
                    .selectable_label(app.interact_mode == InteractMode::View, "View (F1)")
                    .clicked()
                {
                    app.interact_mode = InteractMode::View;
                }
                if ui
                    .selectable_label(app.interact_mode == InteractMode::Edit, "Edit (F2)")
                    .clicked()
                {
                    app.interact_mode = InteractMode::Edit;
                }
                ui.separator();
                match app.interact_mode {
                    InteractMode::View => {
                        ui.label("offset");
                        ui.add(DragValue::new(&mut app.view.start_offset));
                        ui.label("columns");
                        ui.add(DragValue::new(&mut app.view.cols));
                        let data_len = app.data.len();
                        if data_len != 0 {
                            let offsets = app.view_offsets();
                            ui.label(format!(
                                "view offset: row {} col {} byte {} ({:.2}%)",
                                offsets.row,
                                offsets.col,
                                offsets.byte,
                                (offsets.byte as f64 / data_len as f64) * 100.0
                            ));
                            let re = ui.add(
                                TextEdit::singleline(&mut app.center_offset_input)
                                    .hint_text("Center view on offset"),
                            );
                            if re.lost_focus() && ui.input().key_pressed(egui::Key::Enter) {
                                if let Ok(offset) = app.center_offset_input.parse() {
                                    app.center_view_on_offset(offset);
                                }
                            }
                        }
                    }
                    InteractMode::Edit => 'edit: {
                        if app.data.is_empty() {
                            break 'edit;
                        }
                        ui.label(format!("app.cursor: {}", app.cursor));
                        ui.separator();
                        ui.label("u8");
                        if ui
                            .add(TextEdit::singleline(&mut app.u8_buf).desired_width(28.0))
                            .lost_focus()
                            && ui.input().key_pressed(egui::Key::Enter)
                        {
                            app.data[app.cursor] = app.u8_buf.parse().unwrap();
                            app.widen_dirty_region(app.cursor, None);
                        }
                        ui.label("ascii");
                        ui.add(
                            TextEdit::singleline(&mut (app.data[app.cursor] as char).to_string())
                                .desired_width(28.0),
                        );
                    }
                }
                ui.with_layout(Layout::right_to_left(), |ui| {
                    ui.checkbox(&mut app.show_debug_panel, "debug (F12)");
                    ui.checkbox(&mut app.show_block, "block");
                    ui.checkbox(&mut app.show_text, "text");
                    ui.checkbox(&mut app.show_hex, "hex");
                    ui.separator();
                    if ui.add(Button::new("Reload (ctrl+R)")).clicked() {
                        msg_if_fail(app.reload(), "Failed to reload");
                    }
                    if ui
                        .add_enabled(
                            !app.args.read_only && app.dirty_region.is_some(),
                            Button::new("Save (ctrl+S)"),
                        )
                        .clicked()
                    {
                        msg_if_fail(app.save(), "Failed to save");
                    }
                    ui.separator();
                    if ui.button("Restore").clicked() {
                        msg_if_fail(app.restore_backup(), "Failed to restore backup");
                    }
                    if ui.button("Backup").clicked() {
                        msg_if_fail(app.create_backup(), "Failed to create backup");
                    }
                })
            })
        });
        egui::SidePanel::right("right_panel").show(ctx, |ui| {
            ui.label("Hello world");
        });
    });
}
