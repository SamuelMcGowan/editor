use std::ops::ControlFlow;

use anyhow::Result;
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::style::Style;
use ash_term::units::Offset;
use crop::Rope;

use crate::utils::{LineSegment, LineSegments};

#[derive(Default)]
pub struct Editor {
    rope: Rope,
    cursor: usize,
}

impl Editor {
    pub fn handle_event(&mut self, event: Event) -> ControlFlow<Result<()>> {
        match event {
            Event::Key(KeyEvent {
                key_code: KeyCode::Char('Q'),
                modifiers: Modifiers::CTRL,
            }) => return ControlFlow::Break(Ok(())),

            Event::Key(KeyEvent {
                key_code: KeyCode::Char(ch),
                modifiers: Modifiers::EMPTY,
            }) => self.insert_char(ch),

            Event::Key(KeyEvent {
                key_code: KeyCode::Return,
                modifiers: Modifiers::EMPTY,
            }) => self.insert_char('\n'),

            Event::Key(KeyEvent {
                key_code: KeyCode::Left,
                modifiers: Modifiers::EMPTY,
            }) => self.move_left(),

            Event::Key(KeyEvent {
                key_code: KeyCode::Right,
                modifiers: Modifiers::EMPTY,
            }) => self.move_right(),

            Event::Paste(s) => self.insert_str(&s),

            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn insert_char(&mut self, ch: char) {
        if let '\n' | '\r' = ch {
            self.rope.insert(self.cursor, "\n");
            self.cursor += 1;
        } else if !ch.is_control() {
            self.rope.insert(self.cursor, ch.encode_utf8(&mut [0; 4]));
            self.cursor += ch.len_utf8();
        } else {
            return;
        };
    }

    fn insert_str(&mut self, s: &str) {
        for segment in LineSegments::new(s) {
            log::debug!("segment {segment:?}");
            match segment {
                LineSegment::Line(s) => {
                    for part in s.split(|ch: char| ch.is_control()) {
                        self.rope.insert(self.cursor, part);
                        self.cursor += part.len();
                    }
                }

                LineSegment::LineBreak => {
                    self.rope.insert(self.cursor, "\n");
                    self.cursor += 1;
                }
            }
        }
    }

    fn move_left(&mut self) {
        let before_cursor = self.rope.byte_slice(..self.cursor);
        if let Some(prev_char) = before_cursor.graphemes().next_back() {
            self.cursor -= prev_char.len();
        }
    }

    fn move_right(&mut self) {
        let after_cursor = self.rope.byte_slice(self.cursor..);
        if let Some(next_char) = after_cursor.graphemes().next() {
            self.cursor += next_char.len();
        }
    }

    fn cursor_offset(&self) -> Offset {
        let line_num = self.rope.line_of_byte(self.cursor);

        let line_start = if line_num > self.rope.line_len() {
            self.rope.byte_len()
        } else {
            self.rope.byte_of_line(line_num)
        };

        let col_num = self
            .rope
            .byte_slice(line_start..self.cursor)
            .graphemes()
            .count();

        Offset::new(col_num as u16, line_num as u16)
    }

    pub fn draw(&self, buffer: &mut CharBuffer) {
        let mut col = 0;
        let mut line = 0;

        for ch in self.rope.chars() {
            match ch {
                '\n' => {
                    col = 0;
                    line += 1;
                }

                ch if !ch.is_control() => {
                    let Some(cell) = buffer.get_mut([col, line]) else {
                        break;
                    };

                    *cell = Some(Cell::new(ch, Style::default()));
                    col += 1;
                }

                _ => {}
            }
        }

        buffer.cursor = Some(self.cursor_offset());
    }
}
