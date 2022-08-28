use egui_sfml::egui;
use slotmap::Key;

use crate::{
    app::{LayoutKey, ViewKey},
    layout::{default_margin, Layout},
};

use super::window_open::WindowOpen;

#[derive(Default)]
pub struct LayoutsWindow {
    pub open: WindowOpen,
    selected: LayoutKey,
}
impl LayoutsWindow {
    pub(crate) fn ui(ui: &mut egui_sfml::egui::Ui, app: &mut crate::app::App) {
        for (k, v) in &app.view_layout_map {
            if ui
                .selectable_label(app.ui.layouts_window.selected == k, &v.name)
                .clicked()
            {
                app.ui.layouts_window.selected = k;
            }
        }
        if !app.ui.layouts_window.selected.is_null() {
            ui.separator();
            let layout = &mut app.view_layout_map[app.ui.layouts_window.selected];
            ui.heading(&layout.name);
            let unused_views: Vec<ViewKey> = app
                .view_map
                .keys()
                .filter(|&k| !layout.iter().any(|k2| k2 == k))
                .collect();
            egui::Grid::new("view_grid").show(ui, |ui| {
                layout.view_grid.retain_mut(|row| {
                    let mut retain_row = true;
                    row.retain_mut(|view_key| {
                        let mut retain = true;
                        let view = &app.view_map[*view_key];
                        ui.menu_button(&view.name, |ui| {
                            for &k in &unused_views {
                                if ui.button(&app.view_map[k].name).clicked() {
                                    *view_key = k;
                                    ui.close_menu();
                                }
                            }
                            if ui.button("🗑 Delete").clicked() {
                                retain = false;
                                ui.close_menu();
                            }
                        });
                        retain
                    });
                    ui.menu_button("✚", |ui| {
                        for &k in &unused_views {
                            if ui.button(&app.view_map[k].name).clicked() {
                                row.push(k);
                                ui.close_menu();
                            }
                        }
                    })
                    .response
                    .on_hover_text("Add view");
                    if ui.button("🗑").on_hover_text("Delete row").clicked() {
                        retain_row = false;
                    }
                    ui.end_row();
                    retain_row
                });
                if ui.button("✚").on_hover_text("Add row").clicked() {
                    layout.view_grid.push(Vec::new());
                }
            });
        }
        ui.separator();
        if ui.button("New layout").clicked() {
            app.ui.layouts_window.selected = app.view_layout_map.insert(Layout {
                name: "New layout".into(),
                view_grid: Vec::new(),
                margin: default_margin(),
            });
        }
    }
}