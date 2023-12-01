use std::borrow::Cow;
use std::ops::ControlFlow;

use anyhow::Result;
use ash_term::buffer::{BufferView, Cell};
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::units::{OffsetU16, OffsetUsize};
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

#[derive(Default)]
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
        self.cursor_index += s.len();
        self.target_column = None;
    }

    fn insert_char(&mut self, ch: char) {
        self.insert_str(ch.encode_utf8(&mut [0; 4]));
    }

    fn backspace(&mut self) {
        if let Some(prev) = self.grapheme_before_cursor() {
            let prev_len = prev.len();
            self.rope
                .delete((self.cursor_index - prev_len)..self.cursor_index);
            self.cursor_index -= prev_len;
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
        let prev_cursor_index = self.cursor_index;

        'main: {
            let cursor_offset = self.cursor_offset();

            let Some(new_offset_y) = cursor_offset.y.checked_add_signed(n) else {
                self.cursor_index = 0;
                break 'main;
            };

            if new_offset_y >= self.rope.line_len() {
                self.cursor_index = self.rope.byte_len();
                break 'main;
            }

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

        if self.cursor_index == prev_cursor_index {
            self.target_column = None;
        }
    }

    fn go_to_offset(&mut self, offset: OffsetUsize) {
        if offset.y >= self.rope.line_len() {
            self.cursor_index = self.rope.byte_len();
            return;
        };

        let line = self.rope.line(offset.y);
        let line_start = self.rope.byte_of_line(offset.y);

        let new_column = line.graphemes().try_fold(0, |acc, grapheme| {
            let end = acc + grapheme.width();
            if offset.x >= end {
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

    fn scroll_to_show_cursor(&mut self, size: OffsetUsize) {
        let cursor_offset = self.cursor_offset();

        if cursor_offset.x < self.scroll_offset.x {
            self.scroll_offset.x = cursor_offset.x;
        } else if cursor_offset.x >= self.scroll_offset.x + size.x {
            self.scroll_offset.x = cursor_offset.x - size.x + 1;
        }

        if cursor_offset.y < self.scroll_offset.y {
            self.scroll_offset.y = cursor_offset.y;
        } else if cursor_offset.y >= self.scroll_offset.y + size.y {
            self.scroll_offset.y = cursor_offset.y - size.y + 1;
        }
    }
}

impl Editor {
    pub fn draw(&mut self, buffer: &mut BufferView) {
        self.scroll_to_show_cursor(buffer.size().into());

        // ignoring gutter for now
        self.draw_text(buffer);
        self.draw_cursor(buffer);
    }

    fn draw_text(&self, buffer: &mut BufferView) {
        let size: OffsetUsize = buffer.size().into();

        for (y, line) in self
            .rope
            .lines()
            .skip(self.scroll_offset.y)
            .take(size.y)
            .enumerate()
        {
            let mut x = 0;
            for grapheme in line.graphemes() {
                if x >= self.scroll_offset.x {
                    let column = x - self.scroll_offset.x;

                    if column >= size.x {
                        break;
                    }

                    buffer[[column as u16, y as u16]] = Some(Cell::empty().with_symbol(&grapheme));
                }

                x += grapheme.width();
            }
        }
    }

    fn draw_cursor(&self, buffer: &mut BufferView) {
        // If we support cursors being offscreen, we can't use saturating sub.
        let cursor = self.cursor_offset().saturating_sub(self.scroll_offset);

        if cursor.cmp_lt(buffer.size().into()).both() {
            buffer.set_cursor(Some(OffsetU16::from(cursor)));
        }
    }
}
