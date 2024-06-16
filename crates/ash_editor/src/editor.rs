use std::ops::{ControlFlow, Range};

use crate::action::{Action, KeyMap};
use crate::document::{Document, RopeExt};
use anyhow::Result;
use ash_term::buffer::{BufferView, Cell};
use ash_term::event::Event;
use ash_term::style::{CursorShape, CursorStyle, Style, Weight};
use ash_term::units::{OffsetU16, OffsetUsize};
use crop::Rope;
use unicode_width::UnicodeWidthStr;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Default)]
pub struct Editor {
    document: Document,
    mode: Mode,
    keymap: KeyMap,
}

impl Editor {
    pub fn new(document: Document) -> Self {
        Self {
            document,
            ..Default::default()
        }
    }

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

            Action::InsertChar(ch) => self.document.insert_char(ch),
            Action::InsertCharAfter(ch) => self.document.insert_char_after(ch),

            Action::InsertString(s) => self.document.insert_str(&s),
            Action::InsertStringAfter(s) => self.document.insert_str_after(&s),

            Action::Backspace => self.document.backspace(),
            Action::Delete => self.document.delete(),

            Action::MoveLeft => self.document.move_left(),
            Action::MoveRight => self.document.move_right(),
            Action::MoveUp => self.document.move_up(),
            Action::MoveDown => self.document.move_down(),

            Action::MoveHome => self.document.move_home(),
            Action::MoveEnd => self.document.move_end(),

            Action::SetMode(mode) => self.mode = mode,

            Action::Save => self.document.save_file(),
            Action::Quit => return ControlFlow::Break(Ok(())),
        }

        ControlFlow::Continue(())
    }
}

impl Editor {
    pub fn draw(&mut self, buffer: &mut BufferView) {
        self.document.scroll_to_show_cursor(buffer.size().into());

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

        let gutters = Gutters::new(self.document.rope(), "", "  ", "~");
        let max_width = gutters.max_width();

        for (y, gutter) in gutters
            .skip(self.document.scroll_offset().y)
            .take(buffer.size().y as usize)
            .enumerate()
        {
            buffer.draw_text(OffsetU16::new(0, y as u16), &gutter, GUTTER_STYLE);
        }

        max_width
    }

    fn draw_text(&self, buffer: &mut BufferView) {
        let size: OffsetUsize = buffer.size().into();
        let scroll_offset = self.document.scroll_offset();

        for (y, line) in self
            .document
            .rope()
            .lines()
            .skip(scroll_offset.y)
            .take(size.y)
            .enumerate()
        {
            let mut x = 0;
            for grapheme in line.graphemes() {
                if x >= scroll_offset.x {
                    let column = x - scroll_offset.x;

                    if column >= size.x {
                        break;
                    }

                    buffer[[column as u16, y as u16]] =
                        Some(Cell::empty().with_grapheme(&grapheme));
                }

                x += grapheme.width();
            }
        }
    }

    fn draw_cursor(&self, buffer: &mut BufferView) {
        // If we support cursors being offscreen, we can't use saturating sub.
        let cursor = self
            .document
            .cursor_offset()
            .saturating_sub(self.document.scroll_offset());

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

        let max_width = (len.checked_ilog10().unwrap_or_default() as usize + 1).max(blank.width());

        Self {
            lines: 0..len,
            emit_blank: rope.has_trailing_newline(),

            max_width,

            prefix,
            postfix,
            blank,
        }
    }

    fn max_width(&self) -> usize {
        self.max_width + self.prefix.width() + self.postfix.width()
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
