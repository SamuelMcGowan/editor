use crate::char_buffer::Buffer;
use crate::platform::Writer;
use crate::style::Style;
use crate::units::Offset;

pub fn draw_diff(old: &Buffer, new: &Buffer, w: &mut impl Writer) {
    if old.size() != new.size() {
        draw_no_diff(new, w);
        return;
    }

    w.set_cursor_home();
    w.set_cursor_vis(false);

    let mut cursor_pos = Offset::ZERO;
    let mut style = Style::default();

    w.write_style(style);

    for y in 0..new.size().y {
        for x in 0..new.size().x {
            let old_cell = old[[x, y]];
            let new_cell = new[[x, y]];

            if old_cell == new_cell {
                continue;
            }

            let cell = new_cell.unwrap_or_default();

            draw_style_diff(style, cell.style, w);
            style = cell.style;

            let cell_pos = Offset::new(x, y);
            if cell_pos != cursor_pos {
                w.set_cursor_pos(cell_pos);
                cursor_pos = cell_pos;
            }

            cursor_pos.x = cursor_pos.x.saturating_add(1);

            w.write_char(cell.c);
        }
    }

    if let Some(pos) = new.cursor {
        w.set_cursor_pos(pos);
        w.set_cursor_vis(true);
    }
}

fn draw_no_diff(buf: &Buffer, w: &mut impl Writer) {
    w.clear_all();

    w.set_cursor_home();
    w.set_cursor_vis(false);

    let mut style = Style::default();
    w.write_style(style);

    let mut pos_dirty = false;

    for y in 0..buf.size().y {
        for x in 0..buf.size().x {
            let Some(cell) = buf[[x, y]] else {
                pos_dirty = true;
                continue;
            };

            if pos_dirty {
                w.set_cursor_pos([x, y]);
            }

            draw_style_diff(style, cell.style, w);
            style = cell.style;

            w.write_char(cell.c);
        }

        pos_dirty = true;
    }

    if let Some(pos) = buf.cursor {
        w.set_cursor_pos(pos);
        w.set_cursor_vis(true);
    }
}

fn draw_style_diff(old: Style, new: Style, w: &mut impl Writer) {
    if new.fg != old.fg {
        w.set_fg_color(new.fg);
    }

    if new.bg != old.bg {
        w.set_bg_color(new.bg);
    }

    if new.weight != old.weight {
        w.set_weight(new.weight);
    }

    if new.underline != old.underline {
        w.set_underline(new.underline);
    }
}
