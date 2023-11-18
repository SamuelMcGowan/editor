use std::io;
use std::time::Instant;

use crate::event::Event;
use crate::style::{Color, Style, Weight};

mod ansi;
pub mod linux;

pub trait Terminal: Sized {
    type Writer: Writer;

    fn init() -> io::Result<Self>;
    fn size(&self) -> io::Result<(u16, u16)>;

    fn writer(&mut self) -> &mut Self::Writer;
}

pub trait Writer {
    fn flush(&mut self) -> io::Result<()>;

    fn clear_all(&mut self);

    fn set_cursor_home(&mut self);
    fn next_line(&mut self);

    fn set_cursor_pos(&mut self, x: u16, y: u16);
    fn set_cursor_vis(&mut self, vis: bool);

    fn set_fg_color(&mut self, c: Color);
    fn set_bg_color(&mut self, c: Color);

    fn set_weight(&mut self, weight: Weight);
    fn set_underline(&mut self, underline: bool);

    fn write_char(&mut self, c: char);
    fn write_str(&mut self, s: &str);

    fn write_style(&mut self, style: Style) {
        self.set_fg_color(style.fg);
        self.set_bg_color(style.bg);
        self.set_weight(style.weight);
        self.set_underline(style.underline);
    }
}

pub trait Events {
    fn read_with_deadline(&mut self, deadline: Instant) -> io::Result<Option<Event>>;
}
