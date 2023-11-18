use std::fmt::Write;

use super::style::Style;
use crate::draw::style::Weight;

#[derive(Debug, Clone)]
pub struct AnsiBuilder {
    s: String,
    style: Style,

    cursor_visible: bool,
}

impl Default for AnsiBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AnsiBuilder {
    pub fn new() -> Self {
        let mut ansi_builder = AnsiBuilder {
            s: String::new(),
            style: Style::default(),

            // so that the following call works
            cursor_visible: true,
        };

        ansi_builder.set_cursor_position(0, 0);
        ansi_builder.show_cursor(false);

        ansi_builder
    }

    pub fn write_str(&mut self, s: &str) {
        // remove the control characters without having to
        // push each character separately
        for part in s.split(|c: char| c.is_control()) {
            self.s.push_str(part);
        }
    }

    pub fn write_char(&mut self, c: char) {
        if !c.is_control() {
            self.s.push(c);
        }
    }

    pub fn write_newline(&mut self) {
        self.s.push_str("\r\n");
    }

    pub fn write_style(&mut self, style: Style) {
        macro_rules! sgr {
            ($($arg:tt)+) => {{
                write!(self.s, "\x1b[{}m", format_args!($($arg)*)).unwrap();
            }};
        }

        if style.fg != self.style.fg {
            sgr!("3{}", style.fg as u8);
        }

        if style.bg != self.style.bg {
            sgr!("4{}", style.bg as u8);
        }

        if style.weight != self.style.weight {
            match style.weight {
                Weight::Normal => sgr!("22"),
                Weight::Bold => sgr!("1"),
                Weight::Dim => sgr!("2"),
            }
        }

        if style.underline != self.style.underline {
            match style.underline {
                true => sgr!("4"),
                false => sgr!("24"),
            }
        }

        self.style = style;
    }

    pub fn clear_screen(&mut self) {
        self.s.push_str("\x1b[2J");
        self.s.push_str("\x1b[3J");
    }

    pub fn set_cursor_position(&mut self, x: usize, y: usize) {
        let row = y.saturating_add(1);
        let col = x.saturating_add(1);
        write!(self.s, "\x1b[{row};{col}H").unwrap();
    }

    pub fn show_cursor(&mut self, vis: bool) {
        if vis == self.cursor_visible {
            return;
        }

        match vis {
            true => write!(self.s, "\x1b[?25h").unwrap(),
            false => write!(self.s, "\x1b[?25l").unwrap(),
        }

        self.cursor_visible = vis;
    }

    pub fn finish(mut self) -> String {
        self.write_style(Style::default());
        self.s
    }
}

#[cfg(test)]
mod tests {
    use super::AnsiBuilder;
    use crate::draw::style::{Color, Style, Weight};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn my_test() {
        let mut ansi = AnsiBuilder::default();
        ansi.clear_screen();

        ansi.write_str("hello");
        ansi.write_newline();

        ansi.write_style(Style {
            fg: Color::Magenta,
            ..Default::default()
        });
        ansi.write_str("world\r\n");
        ansi.write_newline();

        ansi.write_style(Style {
            fg: Color::Blue,
            weight: Weight::Bold,
            ..Default::default()
        });
        ansi.write_str("boo!\r\n");
        ansi.write_newline();

        let expected = "\x1b[1;1H\x1b[?25l\x1b[2J\x1b[3Jhello\r\n\x1b[35mworld\r\n\x1b[34m\x1b[1mboo!\r\n\x1b[39m\x1b[22m";

        assert_eq!(ansi.finish(), expected);
    }
}
