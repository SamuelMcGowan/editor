mod panic;
mod utils;

use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::draw_char_buffer::draw_diff;
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::platform::{Events, PlatformTerminal, Terminal, Writer};
use ash_term::style::Style;
use ash_term::units::Offset;
use crop::Rope;
use utils::{LineSegment, LineSegments};

const FRAME_RATE: Duration = Duration::from_millis(17);

fn main() -> Result<()> {
    init_logging()?;

    panic::catch_and_reprint_panic(|| Editor::new()?.run()).context("panicked")??;

    Ok(())
}

fn init_logging() -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            let now = chrono::Local::now();

            out.finish(format_args!(
                "[{} {} {}] {}",
                now.format("%Y/%m/%d %H:%M:%S"),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .chain(fern::log_file("output.log")?)
        .apply()?;

    Ok(())
}

pub struct Editor {
    terminal: PlatformTerminal,

    char_buf_prev: CharBuffer,
    char_buf: CharBuffer,

    rope: Rope,

    cursor_byte: usize,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: PlatformTerminal::init()?,

            char_buf_prev: CharBuffer::new(Offset::ZERO),
            char_buf: CharBuffer::new(Offset::ZERO),

            rope: Rope::new(),

            cursor_byte: 0,
        })
    }

    pub fn run(mut self) -> Result<()> {
        self.draw_to_terminal()?;

        loop {
            let deadline = Instant::now() + FRAME_RATE;

            #[allow(clippy::collapsible_match)]
            if let Some(event) = self.terminal.events().read_with_deadline(deadline)? {
                log::debug!("event: {event:?}");

                match event {
                    Event::Key(KeyEvent {
                        key_code: KeyCode::Char('Q'),
                        modifiers: Modifiers::CTRL,
                    }) => return Ok(()),

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
                    }) => {
                        let before_cursor = self.rope.byte_slice(..self.cursor_byte);
                        if let Some(prev_char) = before_cursor.graphemes().next_back() {
                            self.cursor_byte -= prev_char.len();
                        }
                    }

                    Event::Key(KeyEvent {
                        key_code: KeyCode::Right,
                        modifiers: Modifiers::EMPTY,
                    }) => {
                        let after_cursor = self.rope.byte_slice(self.cursor_byte..);
                        if let Some(next_char) = after_cursor.graphemes().next() {
                            self.cursor_byte += next_char.len();
                        }
                    }

                    Event::Paste(s) => self.insert_str(&s),

                    _ => (),
                }
            }

            self.draw_to_buf();
            self.draw_to_terminal()?;
        }
    }

    fn insert_char(&mut self, ch: char) {
        if let '\n' | '\r' = ch {
            self.rope.insert(self.cursor_byte, "\n");
            self.cursor_byte += 1;
        } else if !ch.is_control() {
            self.rope
                .insert(self.cursor_byte, ch.encode_utf8(&mut [0; 4]));
            self.cursor_byte += ch.len_utf8();
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
                        self.rope.insert(self.cursor_byte, part);
                        self.cursor_byte += part.len();
                    }
                }

                LineSegment::LineBreak => {
                    self.rope.insert(self.cursor_byte, "\n");
                    self.cursor_byte += 1;
                }
            }
        }
    }

    fn cursor_pos(&self) -> Offset {
        let line_num = self.rope.line_of_byte(self.cursor_byte);

        let line_start = if line_num > self.rope.line_len() {
            self.rope.byte_len()
        } else {
            self.rope.byte_of_line(line_num)
        };

        let col_num = self
            .rope
            .byte_slice(line_start..self.cursor_byte)
            .graphemes()
            .count();

        Offset::new(col_num as u16, line_num as u16)
    }

    fn draw_to_buf(&mut self) {
        let mut col = 0;
        let mut line = 0;

        for ch in self.rope.chars() {
            match ch {
                '\n' => {
                    col = 0;
                    line += 1;
                }

                ch if !ch.is_control() => {
                    let Some(cell) = self.char_buf.get_mut([col, line]) else {
                        break;
                    };

                    *cell = Some(Cell::new(ch, Style::default()));
                    col += 1;
                }

                _ => {}
            }
        }

        self.char_buf.cursor = Some(self.cursor_pos());
    }

    fn draw_to_terminal(&mut self) -> Result<()> {
        let size = self.terminal.size()?;

        draw_diff(&self.char_buf_prev, &self.char_buf, self.terminal.writer());
        self.terminal.writer().flush()?;

        self.char_buf_prev.clone_from(&self.char_buf);
        self.char_buf.resize_and_clear(size, None);

        Ok(())
    }
}
