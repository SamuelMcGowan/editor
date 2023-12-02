use std::io;
use std::time::Instant;

use crate::event::Event;
use crate::style::{Color, CursorShape, CursorStyle, Style, Weight};
use crate::units::OffsetU16;

mod ansi;
mod ansi_event;
mod input;
pub mod linux;

#[cfg(target_os = "linux")]
pub type PlatformTerminal = linux::LinuxTerminal;

pub trait Terminal: Sized {
    type Writer: Writer;
    type Events: Events;

    fn init() -> io::Result<Self>;
    fn size(&self) -> io::Result<OffsetU16>;

    fn writer(&mut self) -> &mut Self::Writer;
    fn events(&mut self) -> &mut Self::Events;
}

pub trait Writer {
    fn flush(&mut self) -> io::Result<()>;

    fn clear_all(&mut self);

    fn set_cursor_home(&mut self);
    fn next_line(&mut self);

    fn set_cursor_pos(&mut self, poss: impl Into<OffsetU16>);
    fn set_cursor_vis(&mut self, vis: bool);

    fn set_cursor_shape(&mut self, shape: CursorShape);
    fn set_cursor_blinking(&mut self, blinking: bool);

    fn set_fg_color(&mut self, c: Color);
    fn set_bg_color(&mut self, c: Color);

    fn set_weight(&mut self, weight: Weight);
    fn set_underline(&mut self, underline: bool);

    fn write_char(&mut self, ch: char) {
        if !ch.is_control() {
            self.write_str_raw(ch.encode_utf8(&mut [0; 4]));
        }
    }

    #[inline]
    fn write_str(&mut self, s: &str) {
        for part in s.split(char::is_control) {
            self.write_str_raw(part);
        }
    }

    fn write_str_raw(&mut self, s: &str);

    #[inline]
    fn write_style(&mut self, style: Style) {
        self.set_fg_color(style.fg);
        self.set_bg_color(style.bg);
        self.set_weight(style.weight);
        self.set_underline(style.underline);
    }

    #[inline]
    fn write_cursor_style(&mut self, style: CursorStyle) {
        self.set_cursor_shape(style.shape);
        self.set_cursor_blinking(style.blinking);
    }
}

pub trait Events {
    fn read_with_deadline(&mut self, deadline: Instant) -> io::Result<Option<Event>>;
}
