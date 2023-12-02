use std::fmt::Write as _;
use std::io::{self, Write};

use super::Writer;
use crate::style::{Color, CursorShape, Weight};
use crate::units::OffsetU16;

const CSI: &str = "\x1b[";

pub struct AnsiWriter<W: Write> {
    buf: String,
    writer: W,
}

impl<W: Write> AnsiWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            buf: String::new(),
            writer,
        }
    }

    pub fn inner(&self) -> &W {
        &self.writer
    }
}

impl<W: Write> Writer for AnsiWriter<W> {
    #[inline]
    fn flush(&mut self) -> io::Result<()> {
        self.writer.write_all(self.buf.as_bytes())?;
        self.buf.clear();

        self.writer.flush()
    }

    #[inline]
    fn clear_all(&mut self) {
        write!(self.buf, "{CSI}2J{CSI}3J").unwrap();
    }

    #[inline]
    fn set_cursor_home(&mut self) {
        write!(self.buf, "{CSI}H").unwrap();
    }

    #[inline]
    fn set_cursor_pos(&mut self, pos: impl Into<OffsetU16>) {
        let pos = pos.into();

        let row = pos.y.saturating_add(1);
        let col = pos.x.saturating_add(1);

        write!(self.buf, "{CSI}{row};{col}H").unwrap();
    }

    #[inline]
    fn set_cursor_vis(&mut self, vis: bool) {
        match vis {
            true => write!(self.buf, "{CSI}?25h").unwrap(),
            false => write!(self.buf, "{CSI}?25l").unwrap(),
        }
    }

    #[inline]
    fn set_cursor_shape(&mut self, shape: CursorShape) {
        match shape {
            CursorShape::Block => write!(self.buf, "{CSI}2 q").unwrap(),
            CursorShape::Underscore => write!(self.buf, "{CSI}4 q").unwrap(),
            CursorShape::Bar => write!(self.buf, "{CSI}6 q").unwrap(),
        }
    }

    #[inline]
    fn set_cursor_blinking(&mut self, blinking: bool) {
        match blinking {
            true => write!(self.buf, "{CSI}?12h").unwrap(),
            false => write!(self.buf, "{CSI}?12l").unwrap(),
        }
    }

    #[inline]
    fn next_line(&mut self) {
        self.buf.push('\n');
    }

    #[inline]
    fn set_fg_color(&mut self, c: Color) {
        write!(self.buf, "{CSI}3{}m", c as u8).unwrap();
    }

    #[inline]
    fn set_bg_color(&mut self, c: Color) {
        write!(self.buf, "{CSI}4{}m", c as u8).unwrap();
    }

    #[inline]
    fn set_weight(&mut self, weight: Weight) {
        match weight {
            Weight::Normal => write!(self.buf, "{CSI}22m").unwrap(),
            Weight::Bold => write!(self.buf, "{CSI}1m").unwrap(),
            Weight::Dim => write!(self.buf, "{CSI}2m").unwrap(),
        }
    }

    #[inline]
    fn set_underline(&mut self, underline: bool) {
        match underline {
            true => write!(self.buf, "{CSI}4m").unwrap(),
            false => write!(self.buf, "{CSI}24m").unwrap(),
        }
    }

    #[inline]
    fn write_str_raw(&mut self, s: &str) {
        write!(self.buf, "{s}").unwrap();
    }
}
