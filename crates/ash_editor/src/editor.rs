use std::ops::ControlFlow;

use anyhow::Result;
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::style::{Style, Weight};
use ash_term::units::Offset;
use crop::{Rope, RopeSlice};
use unicode_width::UnicodeWidthStr;

use crate::utils::{LineSegment, LineSegments};

const GUTTER: &str = "# ";

const GUTTER_STYLE: Style = Style {
    weight: Weight::Dim,
    ..Style::EMPTY
};

#[derive(Default)]
pub struct Editor {
    rope: Rope,

    cursor_x: usize,
    cursor_x_ghost: usize,
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

        self.cursor_x_ghost = self.cursor_x;
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

        self.cursor_x_ghost = self.cursor_x;
    }

    fn backspace(&mut self) {
        let idx = self.cursor_idx();
        let before = self.rope.byte_slice(..idx);

        let Some(prev) = before.graphemes().next_back() else {
            self.cursor_x_ghost = self.cursor_x;
            return;
        };

        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_x = self.rope.line(self.cursor_y - 1).width();
            self.cursor_y -= 1;
        }
        self.cursor_x_ghost = self.cursor_x;

        self.rope.delete(idx - prev.len()..idx);
    }

    fn delete(&mut self) {
        let idx = self.cursor_idx();
        let after = self.rope.byte_slice(idx..);

        let Some(next) = after.graphemes().next() else {
            self.cursor_x_ghost = self.cursor_x;
            return;
        };

        self.rope.delete(idx..idx + next.len());
    }

    fn move_left(&mut self) {
        if self.cursor_x > 0 {
            self.cursor_x -= 1;
        } else if self.cursor_y > 0 {
            self.cursor_y -= 1;
            self.cursor_x = self.current_line().width();
        }
        self.cursor_x_ghost = self.cursor_x;
    }

    fn move_right(&mut self) {
        if self.cursor_x < self.current_line().width() {
            self.cursor_x += 1;
        } else if self.cursor_y + 1 < self.rope.line_len() {
            self.cursor_y += 1;
            self.cursor_x = 0;
        }
        self.cursor_x_ghost = self.cursor_x;
    }

    fn move_up(&mut self) {
        if self.cursor_y == 0 {
            self.move_home()
        } else {
            self.cursor_y -= 1;
            self.cursor_x = self.cursor_x_ghost.min(self.current_line().width());
        }
    }

    fn move_down(&mut self) {
        if self.cursor_y + 1 < self.rope.line_len() {
            self.cursor_y += 1;
            self.cursor_x = self.cursor_x_ghost.min(self.current_line().width());
        } else {
            self.move_end();
        }
    }

    fn move_home(&mut self) {
        self.cursor_x = 0;
        self.cursor_x_ghost = 0;
    }

    fn move_end(&mut self) {
        self.cursor_x = self.current_line().width();
        self.cursor_x_ghost = self.cursor_x;
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

    pub fn draw(&self, buffer: &mut CharBuffer) {
        self.draw_gutter(buffer);
        self.draw_text(buffer);
        self.draw_cursor(buffer);
    }

    fn draw_text(&self, buffer: &mut CharBuffer) {
        let size = buffer.size();
        let width = (size.x as usize).saturating_sub(GUTTER.len());

        for (y, mut line) in self.rope.lines().take(size.y as usize).enumerate() {
            if line.byte_len() > width {
                line = line.byte_slice(..width);
            }

            for (ch, x) in line.chars().zip(GUTTER.len()..) {
                buffer[[x as u16, y as u16]] = Some(Cell::new(ch, Style::default()));
            }
        }
    }

    fn draw_gutter(&self, buffer: &mut CharBuffer) {
        for y in 0..buffer.size().y {
            for (x, ch) in GUTTER.chars().enumerate() {
                buffer[[x as u16, y]] = Some(Cell::new(ch, GUTTER_STYLE));
            }
        }
    }

    fn draw_cursor(&self, buffer: &mut CharBuffer) {
        let size = buffer.size();

        let width = (size.x as usize).saturating_sub(GUTTER.len());
        let height = size.y as usize;

        if self.cursor_x >= width || self.cursor_y >= height {
            return;
        }

        buffer.cursor = Some(Offset::new(
            (GUTTER.len() + self.cursor_x) as u16,
            self.cursor_y as u16,
        ));
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
