use std::ops::ControlFlow;

use anyhow::Result;
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::style::Style;
use ash_term::units::Offset;
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

use crate::utils::{LineSegment, LineSegments};

#[derive(Default)]
pub struct Editor {
    rope: Rope,

    cursor_x: usize,
    cursor_y: usize,
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

            Event::Key(KeyEvent {
                key_code: KeyCode::Up,
                modifiers: Modifiers::EMPTY,
            }) => self.move_up(),

            Event::Key(KeyEvent {
                key_code: KeyCode::Down,
                modifiers: Modifiers::EMPTY,
            }) => self.move_down(),

            Event::Paste(s) => self.insert_str(&s),

            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn insert_char(&mut self, ch: char) {
        if let '\n' | '\r' = ch {
            self.rope.insert(self.cursor_idx(), "\n");
            self.cursor_x = 0;
            self.cursor_y += 1;
        } else if !ch.is_control() {
            self.rope
                .insert(self.cursor_idx(), ch.encode_utf8(&mut [0; 4]));
            self.cursor_x += 1;
        } else {
            return;
        };
    }

    fn insert_str(&mut self, s: &str) {
        let mut idx = self.cursor_idx();

        for segment in LineSegments::new(s) {
            match segment {
                LineSegment::Line(s) => {
                    for part in s.split(|ch: char| ch.is_control()) {
                        self.rope.insert(idx, part);
                        idx += part.len();
                    }
                }

                LineSegment::LineBreak => {
                    self.rope.insert(idx, "\n");
                    idx += 1;

                    self.cursor_x = 0;
                    self.cursor_y += 1;
                }
            }
        }

        if let Some(LineSegment::Line(last_line)) = LineSegments::new(s).next_back() {
            self.cursor_x += last_line.width();
        }
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.current_line().width();
        }
    }

    fn move_right(&mut self) {
        if self.cursor_x < self.current_line().width() {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.rope.line_len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
    }

    fn move_up(&mut self) {
        if self.cursor_y == 0 {
            self.cursor_x = 0;
        } else {
            self.cursor_y -= 1;
            self.cursor_x = self.cursor_x.min(self.current_line().width());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.rope.line_len() {
            self.cursor_y += 1;
            self.cursor_x = self.cursor_x.min(self.current_line().width());
        } else {
            self.cursor_x = self.current_line().width();
        }
    }

    fn current_line(&self) -> RopeSlice {
        if self.cursor_y >= self.rope.line_len() {
            self.rope.byte_slice(self.rope.byte_len()..)
        } else {
            self.rope.line(self.cursor_y)
        }
    }

    fn cursor_idx(&self) -> usize {
        if self.cursor_y >= self.rope.line_len() {
            self.rope.byte_len()
        } else {
            let line_start = self.rope.byte_of_line(self.cursor_y);
            let column_len: usize = self
                .rope
                .line(self.cursor_y)
                .graphemes()
                .take(self.cursor_x)
                .map(|g| g.len())
                .sum();

            line_start + column_len
        }
    }

    fn cursor_view_offset(&self) -> Offset {
        Offset::new(self.cursor_x as u16, self.cursor_y as u16)
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

        buffer.cursor = Some(self.cursor_view_offset());
    }
}

trait RopeSliceExt {
    fn width(&self) -> usize;
}

impl RopeSliceExt for RopeSlice<'_> {
    fn width(&self) -> usize {
        self.graphemes().map(|g| g.len()).sum()
    }
}
