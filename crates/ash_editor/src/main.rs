mod action;
mod document;
mod editor;
mod panic;
mod utils;

use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ash_term::buffer::Buffer;
use ash_term::draw_buffer::draw_diff;
use ash_term::platform::{Events, PlatformTerminal, Terminal, Writer};
use ash_term::units::OffsetU16;
use clap::Parser;
use document::Document;
use editor::Editor;

const FRAME_RATE: Duration = Duration::from_millis(17);

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Mode {
    #[default]
    Normal,
    Insert,
}

#[derive(Parser)]
struct Args {
    path: Option<PathBuf>,
}

fn main() -> Result<()> {
    init_logging()?;

    let args = Args::parse();

    panic::catch_and_reprint_panic(|| App::new(args)?.run()).context("panicked")??;

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
        .chain(fern::log_file("logs/editor.log")?)
        .apply()?;

    Ok(())
}

struct App {
    terminal: PlatformTerminal,

    char_buf_prev: Buffer,
    char_buf: Buffer,

    editor: Editor,
}

impl App {
    fn new(args: Args) -> Result<Self> {
        let document = Document::new(args.path)?;

        Ok(Self {
            terminal: PlatformTerminal::init()?,

            char_buf_prev: Buffer::new(OffsetU16::ZERO),
            char_buf: Buffer::new(OffsetU16::ZERO),

            editor: Editor::new(document),
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

        self.char_buf.resize_and_clear(size);
        self.editor.draw(&mut self.char_buf.view(true));

        draw_diff(
            &self.char_buf_prev.view(false),
            &self.char_buf.view(false),
            self.terminal.writer(),
        );

        self.terminal.writer().flush()?;

        self.char_buf_prev.clone_from(&self.char_buf);

        Ok(())
    }
}
