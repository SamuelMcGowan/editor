use std::borrow::Cow;
use std::ops::ControlFlow;

use ash_term::units::OffsetUsize;
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

pub struct Editor {
    rope: Rope,

    /// Cursor position, as a byte index.
    cursor_index: usize,

    /// Column to try to move to when moving (in cells).
    start_column: Option<usize>,

    /// Scroll offset, in cells.
    scroll_offset: OffsetUsize,
}

impl Editor {
    fn move_left(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            self.cursor_index -= prev.len();
        }
        self.start_column = None;
    }

    fn move_right(&mut self) {
        if let Some(next) = self.grapheme_after_cursor() {
            self.cursor_index += next.len();
        }
        self.start_column = None;
    }

    fn move_vertical(&mut self, n: isize) {
        let cursor_offset = self.cursor_offset();

        // Doesn't matter if this is greater than the number of lines, `go_to_offset`
        // handles it.
        let new_offset_y = cursor_offset.y.saturating_add_signed(n);

        let new_offset_x = match self.start_column {
            Some(col) => col,
            None => {
                let col = cursor_offset.x;
                self.start_column = Some(col);
                col
            }
        };

        self.go_to_offset(OffsetUsize::new(new_offset_x, new_offset_y));
    }

    fn go_to_offset(&mut self, offset: OffsetUsize) {
        if offset.y >= self.rope.line_len() {
            self.cursor_index = self.rope.byte_len();
            self.start_column = None;
            return;
        };

        let line = self.rope.line(offset.y);
        let line_start = self.rope.byte_of_line(offset.y);

        let new_column = line.graphemes().try_fold(0, |acc, grapheme| {
            let end = acc + grapheme.width();
            if offset.x > end {
                ControlFlow::Continue(end)
            } else {
                ControlFlow::Break(acc)
            }
        });

        let new_column = match new_column {
            ControlFlow::Break(start) => start,
            ControlFlow::Continue(start) => start,
        };

        self.cursor_index = line_start + new_column;
    }

    fn grapheme_before_cursor(&self) -> Option<Cow<str>> {
        self.rope_before_cursor().graphemes().next_back()
    }

    fn grapheme_after_cursor(&self) -> Option<Cow<str>> {
        self.rope_after_cursor().graphemes().next()
    }

    fn rope_before_cursor(&self) -> RopeSlice {
        self.rope.byte_slice(..self.cursor_index)
    }

    fn rope_after_cursor(&self) -> RopeSlice {
        self.rope.byte_slice(self.cursor_index..)
    }

    /// The cursor offset, in cells.
    fn cursor_offset(&self) -> OffsetUsize {
        let line = self.rope.line_of_byte(self.cursor_index);
        let line_start = self.rope.byte_of_line(line);

        // Fine to sum up the widths of each chunk - the `width` implementation just
        // sums the character widths, so it seems there's nothing contextual
        // that is lost by splitting up a string.
        let column: usize = self
            .rope
            .byte_slice(line_start..self.cursor_index)
            .chunks()
            .map(|s| s.width())
            .sum();

        OffsetUsize::new(column, line)
    }

    fn get_or_set_start_column(&mut self) -> usize {
        match self.start_column {
            Some(col) => col,
            None => {
                let col = self.cursor_offset().x;
                self.start_column = Some(col);
                col
            }
        }
    }
}
