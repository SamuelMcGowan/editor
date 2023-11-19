use std::time::{Duration, Instant};

use anyhow::Result;
use ash_term::char_buffer::{Cell, CharBuffer};
use ash_term::draw_char_buffer::draw_diff;
use ash_term::event::{Event, KeyCode, KeyEvent, Modifiers};
use ash_term::platform::{Events, PlatformTerminal, Terminal, Writer};
use ash_term::style::{Color, Style};
use ash_term::units::Offset;

const FRAME_RATE: Duration = Duration::from_millis(17);

fn main() -> Result<()> {
    Editor::new()?.run()
}

pub struct Editor {
    terminal: PlatformTerminal,

    char_buf_prev: CharBuffer,
    char_buf: CharBuffer,
}

impl Editor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: PlatformTerminal::init()?,
            char_buf_prev: CharBuffer::new(Offset::ZERO),
            char_buf: CharBuffer::new(Offset::ZERO),
        })
    }

    pub fn run(mut self) -> Result<()> {
        self.draw()?;

        loop {
            let deadline = Instant::now() + FRAME_RATE;

            #[allow(clippy::collapsible_match)]
            if let Some(event) = self.terminal.events().read_with_deadline(deadline)? {
                if let Event::Key(KeyEvent {
                    key_code: KeyCode::Char('Q'),
                    modifiers: Modifiers::CTRL,
                }) = event
                {
                    return Ok(());
                }
            }

            self.draw()?;
        }
    }

    fn draw(&mut self) -> Result<()> {
        let fill_style = Style {
            bg: Color::Green,
            ..Default::default()
        };
        let fill_cell = Cell::new(' ', fill_style);

        let size = self.terminal.size()?;

        self.char_buf.resize_and_clear(size, Some(fill_cell));

        draw_diff(&self.char_buf_prev, &self.char_buf, self.terminal.writer());
        self.terminal.writer().flush()?;

        self.char_buf_prev.clone_from(&self.char_buf);

        Ok(())
    }
}
