use std::borrow::Cow;
use std::ops::{ControlFlow, Range};

use anyhow::Result;
use ash_term::buffer::{BufferView, Cell};
use ash_term::event::Event;
use ash_term::style::{CursorShape, CursorStyle, Style, Weight};
use ash_term::units::{OffsetU16, OffsetUsize};
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

use crate::action::{Action, KeyMap};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Default)]
pub struct Editor {
    rope: Rope,

    /// Cursor position, as a byte index.
    cursor_index: usize,

    /// Column to try to move to when moving (in cells).
    target_column: Option<usize>,

    /// Scroll offset, in cells.
    scroll_offset: OffsetUsize,

    mode: Mode,

    keymap: KeyMap,
}

impl Editor {
    pub fn handle_event(&mut self, event: Event) -> ControlFlow<Result<()>> {
        if let Some(action) = self.keymap.get_action(self.mode, event) {
            self.handle_action(action)
        } else {
            ControlFlow::Continue(())
        }
    }

    fn handle_action(&mut self, action: Action) -> ControlFlow<Result<()>> {
        match action {
            Action::Combo(actions) => {
                for action in actions {
                    self.handle_action(action)?;
                }
            }

            Action::InsertChar(ch) => self.insert_char(ch),
            Action::InsertString(s) => self.insert_str(&s),

            Action::Backspace => self.backspace(),
            Action::Delete => self.delete(),

            Action::MoveLeft => self.move_left(),
            Action::MoveRight => self.move_right(),
            Action::MoveUp => self.move_up(),
            Action::MoveDown => self.move_down(),

            Action::MoveHome => self.move_home(),
            Action::MoveEnd => self.move_end(),

            Action::SetMode(mode) => self.mode = mode,

            Action::Quit => return ControlFlow::Break(Ok(())),
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

        let gutter_width = self.draw_gutter(buffer);

        let mut edit_view = buffer.view(gutter_width as u16.., .., true);
        self.draw_text(&mut edit_view);
        self.draw_cursor(&mut edit_view);
    }

    fn draw_gutter(&self, buffer: &mut BufferView) -> usize {
        const GUTTER_STYLE: Style = Style {
            weight: Weight::Dim,
            ..Style::EMPTY
        };

        let gutters = Gutters::new(&self.rope, "", "  ", "~");
        let max_width = gutters.max_width();

        for (y, gutter) in gutters
            .skip(self.scroll_offset.y)
            .take(buffer.size().y as usize)
            .enumerate()
        {
            // TODO: use graphemes
            for (x, ch) in gutter.chars().enumerate() {
                buffer[[x as u16, y as u16]] =
                    Some(Cell::empty().with_char(ch).with_style(GUTTER_STYLE))
            }
        }

        max_width
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

        let style = match self.mode {
            Mode::Normal => CursorStyle {
                shape: CursorShape::Block,
                blinking: false,
            },
            Mode::Insert => CursorStyle {
                shape: CursorShape::Bar,
                blinking: true,
            },
        };

        buffer.set_cursor_style(style);
    }
}

struct Gutters<'a> {
    lines: Range<usize>,
    emit_blank: bool,

    max_width: usize,

    prefix: &'a str,
    postfix: &'a str,
    blank: &'a str,
}

impl<'a> Gutters<'a> {
    fn new(rope: &Rope, prefix: &'a str, postfix: &'a str, blank: &'a str) -> Self {
        let len = rope.line_len();

        let trailing_newline = match rope.chunks().last() {
            Some(chunk) => chunk.ends_with('\n'),
            None => true,
        };

        let max_width = (len.checked_ilog10().unwrap_or_default() as usize + 1).max(blank.len());

        Self {
            lines: 0..len,
            emit_blank: trailing_newline,

            max_width,

            prefix,
            postfix,
            blank,
        }
    }

    fn max_width(&self) -> usize {
        self.max_width + self.prefix.len() + self.postfix.len()
    }

    fn next_with(&mut self, f: impl Fn(&mut Self) -> Option<usize>) -> Option<String> {
        if let Some(line) = f(self) {
            return Some(format!(
                "{}{:>w$}{}",
                self.prefix,
                line + 1,
                self.postfix,
                w = self.max_width
            ));
        }

        if self.emit_blank {
            self.emit_blank = false;
            Some(format!(
                "{}{:>w$}{}",
                self.prefix,
                self.blank,
                self.postfix,
                w = self.max_width
            ))
        } else {
            None
        }
    }
}

impl Iterator for Gutters<'_> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_with(|s| s.lines.next())
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.next_with(|s| s.lines.nth(n))
    }
}
