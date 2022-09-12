mod advanced_open_window;
mod bookmarks_window;
mod bottom_panel;
mod debug_window;
pub mod dialogs;
mod file_diff_result_window;
mod find_dialog;
mod find_memory_pointers_window;
mod help_window;
pub mod inspect_panel;
mod layouts_window;
mod meta_diff_window;
mod open_process_window;
mod ops;
mod perspectives_window;
mod regions_window;
mod top_menu;
mod top_panel;
mod util;
mod views_window;
mod window_open;

use {
    self::{
        advanced_open_window::AdvancedOpenWindow, bookmarks_window::BookmarksWindow,
        file_diff_result_window::FileDiffResultWindow, find_dialog::FindDialog,
        find_memory_pointers_window::FindMemoryPointersWindow, help_window::HelpWindow,
        inspect_panel::InspectPanel, layouts_window::LayoutsWindow,
        meta_diff_window::MetaDiffWindow, open_process_window::OpenProcessWindow,
        perspectives_window::PerspectivesWindow, regions_window::RegionsWindow,
        views_window::ViewsWindow,
    },
    crate::{
        app::App,
        meta::{Bookmark, ValueType, ViewKey},
        view::{ViewportScalar, ViewportVec},
    },
    egui_sfml::{
        egui::{self, TopBottomPanel, Window},
        sfml::graphics::Font,
        SfEgui,
    },
    std::fmt::Debug,
};

#[derive(Default)]
pub struct Gui {
    pub inspect_panel: InspectPanel,
    pub find_dialog: FindDialog,
    pub center_offset_input: String,
    pub seek_byte_offset_input: String,
    pub regions_window: RegionsWindow,
    pub bookmarks_window: BookmarksWindow,
    pub dialogs: Vec<Box<dyn Dialog>>,
    pub layouts_window: LayoutsWindow,
    pub views_window: ViewsWindow,
    pub perspectives_window: PerspectivesWindow,
    pub help_window: HelpWindow,
    pub file_diff_result_window: FileDiffResultWindow,
    pub context_menu: Option<ContextMenu>,
    pub meta_diff_window: MetaDiffWindow,
    pub open_process_window: OpenProcessWindow,
    pub find_memory_pointers_window: FindMemoryPointersWindow,
    pub advanced_open_window: AdvancedOpenWindow,
}

pub struct ContextMenu {
    pos: egui::Pos2,
    data: ContextMenuData,
}

impl ContextMenu {
    pub fn new(mx: ViewportScalar, my: ViewportScalar, data: ContextMenuData) -> Self {
        Self {
            pos: egui::pos2(f32::from(mx), f32::from(my)),
            data,
        }
    }
}

pub enum ContextMenuData {
    ViewByte { view: ViewKey, byte_off: usize },
}

pub trait Dialog: Debug {
    fn title(&self) -> &str;
    /// Do the ui for this dialog. Returns whether to keep this dialog open.
    fn ui(&mut self, ui: &mut egui::Ui, app: &mut App) -> bool;
}

impl Gui {
    pub fn add_dialog<D: Dialog + 'static>(&mut self, dialog: D) {
        self.dialogs.push(Box::new(dialog));
    }
}

pub fn do_egui(
    sf_egui: &mut SfEgui,
    gui: &mut crate::gui::Gui,
    app: &mut App,
    mouse_pos: ViewportVec,
    font: &Font,
) {
    sf_egui.do_frame(|ctx| {
        let mut open = gamedebug_core::enabled();
        let was_open = open;
        Window::new("Debug")
            .open(&mut open)
            .show(ctx, debug_window::ui);
        if was_open && !open {
            gamedebug_core::toggle();
        }
        macro_rules! windows {
            ($($title:expr, $field:ident, $ty:ty: $($arg:ident)+;)*) => {
                $(
                    open = gui.$field.open.is();
                    Window::new($title).open(&mut open).show(ctx, |ui| <$ty>::ui(ui, $($arg,)+));
                    if !open {
                        gui.$field.open.set(false);
                    }
                )*
            };
        }
        windows! {
            "Find",                    find_dialog,                 FindDialog: gui app;
            "Regions",                 regions_window,              RegionsWindow: gui app;
            "Bookmarks",               bookmarks_window,            BookmarksWindow: gui app;
            "Layouts",                 layouts_window,              LayoutsWindow: gui app;
            "Views",                   views_window,                ViewsWindow: gui app font;
            "Perspectives",            perspectives_window,         PerspectivesWindow: gui app;
            "Help",                    help_window,                 HelpWindow: gui;
            "File Diff results",       file_diff_result_window,     FileDiffResultWindow: gui app;
            "Diff against clean meta", meta_diff_window,            MetaDiffWindow: app;
            "Open process",            open_process_window,         OpenProcessWindow: gui app font;
            "Find memory pointers",    find_memory_pointers_window, FindMemoryPointersWindow: gui app font;
            "Advanced open",           advanced_open_window,        AdvancedOpenWindow: gui app font;
        }
        // Context menu
        if let Some(menu) = &gui.context_menu {
            let mut close = false;
            egui::Area::new("rootless_ctx_menu")
                .fixed_pos(menu.pos)
                .show(ctx, |ui| {
                    ui.set_max_width(180.0);
                    egui::Frame::menu(ui.style())
                        .inner_margin(2.0)
                        .show(ui, |ui| match &menu.data {
                            &ContextMenuData::ViewByte { view, byte_off } => {
                                if let Some(sel) = app.hex.ui.selection() {
                                    if ui.button("Add selection as region").clicked() {
                                        ops::add_region_from_selection(sel, &mut app.hex.meta_state, &mut gui.regions_window)
                                    }
                                    ui.separator();
                                }
                                if ui
                                    .button("Add bookmark")
                                    .clicked()
                                {
                                    let bms = &mut app.hex.meta_state.meta.bookmarks;
                                    let idx = bms.len();
                                    bms.push(Bookmark {
                                        offset: byte_off,
                                        label: format!("New @ offset {}", byte_off),
                                        desc: String::new(),
                                        value_type: ValueType::None,
                                    });
                                    gui.bookmarks_window.open.set(true);
                                    gui.bookmarks_window.selected = Some(idx);
                                    close = true;
                                }
                                ui.separator();
                                if ui.button("View properties...").clicked() {
                                    gui.views_window.selected = view;
                                    gui.views_window.open.set(true);
                                    close = true;
                                }
                            }
                        });
                });
            if close {
                gui.context_menu = None;
            }
        }
        // Panels
        let top_re =
            TopBottomPanel::top("top_panel").show(ctx, |ui| top_panel::ui(ui, gui, app, font));
        let bot_re = TopBottomPanel::bottom("bottom_panel")
            .show(ctx, |ui| bottom_panel::ui(ui, app, mouse_pos));
        let right_re = egui::SidePanel::right("right_panel")
            .show(ctx, |ui| inspect_panel::ui(ui, app, gui, mouse_pos))
            .response;
        let padding = 2;
        app.hex.ui.hex_iface_rect.x = padding;
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Window size can't exceed i16"
        )]
        {
            app.hex.ui.hex_iface_rect.y = top_re.response.rect.bottom() as ViewportScalar + padding;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Window size can't exceed i16"
        )]
        {
            app.hex.ui.hex_iface_rect.w = right_re.rect.left() as ViewportScalar - padding * 2;
        }
        #[expect(
            clippy::cast_possible_truncation,
            reason = "Window size can't exceed i16"
        )]
        {
            app.hex.ui.hex_iface_rect.h = (bot_re.response.rect.top() as ViewportScalar
                - app.hex.ui.hex_iface_rect.y)
                - padding * 2;
        }
        let mut dialogs: Vec<_> = std::mem::take(&mut gui.dialogs);
        dialogs.retain_mut(|dialog| {
            let mut retain = true;
            Window::new(dialog.title()).show(ctx, |ui| {
                retain = dialog.ui(ui, app);
            });
            retain
        });
        gui.dialogs = dialogs;
    });
}
