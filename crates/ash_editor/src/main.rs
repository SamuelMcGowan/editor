use std::time::{Duration, Instant};

use anyhow::Result;
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::draw_char_buffer::draw_diff;
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::platform::{Events, PlatformTerminal, Terminal, Writer};
use ash_term::style::Style;
use ash_term::units::Offset;
use crop::Rope;

const FRAME_RATE: Duration = Duration::from_millis(17);

fn main() -> Result<()> {
    init_logging()?;

    Editor::new()?.run()
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
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: PlatformTerminal::init()?,

            char_buf_prev: CharBuffer::new(Offset::ZERO),
            char_buf: CharBuffer::new(Offset::ZERO),

            rope: Rope::new(),
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
                    }) => self
                        .rope
                        .insert(self.cursor_offset(), ch.encode_utf8(&mut [0; 4])),

                    Event::Key(KeyEvent {
                        key_code: KeyCode::Return,
                        modifiers: Modifiers::EMPTY,
                    }) => self.rope.insert(self.cursor_offset(), "\n"),

                    _ => (),
                }
            }

            self.draw_to_buf();
            self.draw_to_terminal()?;
        }
    }

    fn cursor_offset(&self) -> usize {
        self.rope.byte_len()
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

        self.char_buf.cursor = Some(Offset::new(col, line));
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
