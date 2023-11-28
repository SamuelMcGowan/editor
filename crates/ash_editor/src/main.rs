mod editor;
mod panic;
mod utils;

use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ash_term::char_buffer::CharBuffer;
use ash_term::draw_char_buffer::draw_diff;
use ash_term::platform::{Events, PlatformTerminal, Terminal, Writer};
use ash_term::units::OffsetU16;
use editor::Editor;

const FRAME_RATE: Duration = Duration::from_millis(17);

fn main() -> Result<()> {
    init_logging()?;

    panic::catch_and_reprint_panic(|| App::new()?.run()).context("panicked")??;

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

struct App {
    terminal: PlatformTerminal,

    char_buf_prev: CharBuffer,
    char_buf: CharBuffer,

    editor: Editor,
}

impl App {
    fn new() -> Result<Self> {
        Ok(Self {
            terminal: PlatformTerminal::init()?,

            char_buf_prev: CharBuffer::new(OffsetU16::ZERO),
            char_buf: CharBuffer::new(OffsetU16::ZERO),

            editor: Editor::default(),
        })
    }

    fn run(mut self) -> Result<()> {
        self.draw()?;

        loop {
            let deadline = Instant::now() + FRAME_RATE;

            if let Some(event) = self.terminal.events().read_with_deadline(deadline)? {
                log::debug!("event: {event:?}");

                if let ControlFlow::Break(res) = self.editor.handle_event(event) {
                    return res;
                }
            }

            self.draw()?;
        }
    }

    fn draw(&mut self) -> Result<()> {
        let size = self.terminal.size()?;

        self.char_buf.resize_and_clear(size, None);
        self.editor.draw(&mut self.char_buf);

        draw_diff(&self.char_buf_prev, &self.char_buf, self.terminal.writer());
        self.terminal.writer().flush()?;

        self.char_buf_prev.clone_from(&self.char_buf);

        Ok(())
    }
}
