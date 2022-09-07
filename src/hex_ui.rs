use slotmap::Key;

use crate::{
    app::interact_mode::InteractMode,
    meta::{LayoutKey, ViewKey},
    timer::Timer,
    view::ViewportRect,
};

/// State related to the hex view ui, different from the egui gui overlay
pub struct HexUi {
    /// "a" point of selection. Could be smaller or larger than "b".
    /// The length of selection is absolute difference between a and b
    pub select_a: Option<usize>,
    /// "b" point of selection. Could be smaller or larger than "a".
    /// The length of selection is absolute difference between a and b
    pub select_b: Option<usize>,
    pub interact_mode: InteractMode,
    pub current_layout: LayoutKey,
    pub focused_view: Option<ViewKey>,
    /// The rectangle area that's available for the hex interface
    pub hex_iface_rect: ViewportRect,
    pub flash_cursor_timer: Timer,
    /// Whether to scissor views when drawing them. Useful to disable when debugging rendering.
    pub scissor_views: bool,
    /// When alt is being held, it shows things like names of views as overlays
    pub show_alt_overlay: bool,
}

impl Default for HexUi {
    fn default() -> Self {
        Self {
            scissor_views: true,
            interact_mode: InteractMode::View,
            focused_view: None,
            select_a: None,
            select_b: None,
            flash_cursor_timer: Timer::default(),
            hex_iface_rect: ViewportRect::default(),
            show_alt_overlay: false,
            current_layout: LayoutKey::null(),
        }
    }
}