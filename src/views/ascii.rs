use gamedebug_core::imm_msg;
use sfml::graphics::{Color, Font, Rect, RectangleShape, RenderTarget, RenderWindow, Shape};

use crate::{
    app::App,
    views::{draw_cursor, draw_glyph},
    EditTarget, InteractMode,
};

pub fn ascii(app: &mut App, view_idx_off_y: usize, window: &mut RenderWindow, font: &Font) {
    // The offset for the ascii display imposed by the view
    let ascii_display_x_offset = app.ascii_display_x_offset();
    imm_msg!(ascii_display_x_offset);
    let view_idx_off_x: usize = app
        .view_x
        .saturating_sub(ascii_display_x_offset)
        .try_into()
        .unwrap_or(0)
        / app.col_width as usize;
    //let view_idx_off_y: usize = app.view_y.try_into().unwrap_or(0) / app.row_height as usize;
    let view_idx_off = view_idx_off_y * app.view.cols + view_idx_off_x;
    imm_msg!("ascii");
    imm_msg!(view_idx_off_x);
    //imm_msg!(view_idx_off_y);
    imm_msg!(view_idx_off);
    let mut ascii_rows_rendered: u32 = 0;
    let mut ascii_cols_rendered: u32 = 0;
    let mut idx = app.view.start_offset + view_idx_off;
    imm_msg!(idx);
    'asciidisplay: for y in 0..app.view.rows {
        for x in 0..app.view.cols {
            if x == app.max_visible_cols * 2 || x >= app.view.cols.saturating_sub(view_idx_off_x) {
                idx += app.view.cols - x;
                break;
            }
            if idx >= app.data.len() {
                break 'asciidisplay;
            }
            let pix_x = (x + app.view.cols * 2 + 1) as f32 * f32::from(app.col_width / 2)
                - app.view_x as f32;
            //let pix_y = y as f32 * f32::from(app.row_height) - app.view_y as f32;
            let pix_y = (y + view_idx_off_y) as f32 * f32::from(app.row_height) - app.view_y as f32;
            let byte = app.data[idx];
            let c = app.color_method.byte_color(byte);
            let selected = match app.selection {
                Some(sel) => (sel.begin..=sel.end).contains(&idx),
                None => false,
            };
            if selected || (app.find_dialog.open && app.find_dialog.result_offsets.contains(&idx)) {
                let mut rs = RectangleShape::from_rect(Rect::new(
                    pix_x,
                    pix_y,
                    (app.col_width / 2) as f32,
                    app.row_height as f32,
                ));
                rs.set_fill_color(Color::rgb(150, 150, 150));
                if app.cursor == idx {
                    rs.set_outline_color(Color::WHITE);
                    rs.set_outline_thickness(-2.0);
                }
                window.draw(&rs);
            }
            if idx == app.cursor {
                draw_cursor(
                    pix_x,
                    pix_y,
                    window,
                    app.edit_target == EditTarget::Text && app.interact_mode == InteractMode::Edit,
                );
            }
            draw_glyph(
                font,
                app.font_size,
                &mut app.vertices,
                pix_x,
                pix_y,
                byte as u32,
                c,
            );
            idx += 1;
            ascii_cols_rendered += 1;
        }
        ascii_rows_rendered += 1;
    }
    imm_msg!(ascii_rows_rendered);
    ascii_cols_rendered = ascii_cols_rendered
        .checked_div(ascii_rows_rendered)
        .unwrap_or(0);
    imm_msg!(ascii_cols_rendered);
}