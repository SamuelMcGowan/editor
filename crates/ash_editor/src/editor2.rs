use std::borrow::Cow;
use std::ops::ControlFlow;

use anyhow::Result;
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::units::OffsetUsize;
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

pub struct Editor {
    rope: Rope,

    /// Cursor position, as a byte index.
    cursor_index: usize,

    /// Column to try to move to when moving (in cells).
    target_column: Option<usize>,

    /// Scroll offset, in cells.
    scroll_offset: OffsetUsize,
}

impl Editor {
    pub fn handle_event(&mut self, event: Event) -> ControlFlow<Result<()>> {
        match event {
            Event::Key(KeyEvent {
                key_code: KeyCode::Char('Q'),
                modifiers: Modifiers::CTRL,
            }) => return ControlFlow::Break(Ok(())),

            Event::Key(KeyEvent {
                key_code,
                modifiers: Modifiers::EMPTY,
            }) => match key_code {
                KeyCode::Char(ch) => self.insert_char(ch),
                KeyCode::Return => self.insert_char('\n'),

                KeyCode::Backspace => self.backspace(),
                KeyCode::Delete => self.delete(),

                KeyCode::Left => self.move_left(),
                KeyCode::Right => self.move_right(),
                KeyCode::Up => self.move_up(),
                KeyCode::Down => self.move_down(),

                KeyCode::Home => self.move_home(),
                KeyCode::End => self.move_end(),
                _ => {}
            },

            Event::Paste(s) => self.insert_str(&s),

            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn insert_str(&mut self, s: &str) {
        self.rope.insert(self.cursor_index, s);
        self.target_column = None;
    }

    fn insert_char(&mut self, ch: char) {
        self.insert_str(ch.encode_utf8(&mut [0; 4]));
    }

    fn backspace(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            self.rope
                .delete((self.cursor_index - prev.len())..self.cursor_index);
        }
        self.target_column = None;
    }

    fn delete(&mut self) {
        if let Some(next) = self.grapheme_after_cursor() {
            self.rope
                .delete(self.cursor_index..(self.cursor_index + next.len()));
        }
        self.target_column = None;
    }

    fn move_left(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            self.cursor_index -= prev.len();
        }
        self.target_column = None;
    }

    fn move_right(&mut self) {
        if let Some(next) = self.grapheme_after_cursor() {
            self.cursor_index += next.len();
        }
        self.target_column = None;
    }

    fn move_up(&mut self) {
        self.move_vertical(-1);
    }

    fn move_down(&mut self) {
        self.move_vertical(1);
    }

    fn move_home(&mut self) {
        self.go_to_offset(OffsetUsize::new(0, self.cursor_offset().y));
    }

    fn move_end(&mut self) {
        let (_, line) = self.current_line();
        let line_width = line.chunks().map(|chunk| chunk.width()).sum();
        self.go_to_offset(OffsetUsize::new(line_width, self.cursor_offset().y))
    }

    fn move_vertical(&mut self, n: isize) {
        let cursor_offset = self.cursor_offset();

        // Doesn't matter if this is greater than the number of lines, `go_to_offset`
        // handles it.
        let new_offset_y = cursor_offset.y.saturating_add_signed(n);

        let new_offset_x = match self.target_column {
            Some(col) => col,
            None => {
                let col = cursor_offset.x;
                self.target_column = Some(col);
                col
            }
        };

        self.go_to_offset(OffsetUsize::new(new_offset_x, new_offset_y));
    }

    fn go_to_offset(&mut self, offset: OffsetUsize) {
        if offset.y >= self.rope.line_len() {
            self.cursor_index = self.rope.byte_len();
            self.target_column = None;
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

    fn current_line(&self) -> (usize, RopeSlice) {
        let line_num = self.rope.line_of_byte(self.cursor_index);

        let slice = if line_num == self.rope.line_len() {
            self.rope.byte_slice(self.cursor_index..)
        } else {
            self.rope.line(line_num)
        };

        (line_num, slice)
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
        match self.target_column {
            Some(col) => col,
            None => {
                let col = self.cursor_offset().x;
                self.target_column = Some(col);
                col
            }
        }
    }
}
