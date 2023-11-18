use self::raw_term::RawTerm;
use super::ansi::AnsiWriter;
use super::{Terminal, Writer};
use crate::style::Style;

mod raw_term;

pub struct LinuxTerminal {
    ansi_raw_term: AnsiWriter<RawTerm>,
}

impl Terminal for LinuxTerminal {
    type Writer = AnsiWriter<RawTerm>;

    fn init() -> std::io::Result<Self> {
        let mut term = Self {
            ansi_raw_term: AnsiWriter::new(RawTerm::new()?),
        };

        term.writer().clear_all();
        term.writer().flush()?;

        Ok(term)
    }

    fn size(&self) -> std::io::Result<(u16, u16)> {
        self.ansi_raw_term.inner().size()
    }

    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.ansi_raw_term
    }
}

impl Drop for LinuxTerminal {
    fn drop(&mut self) {
        self.writer().clear_all();
        self.writer().set_cursor_home();
        self.writer().set_cursor_vis(true);
        self.writer().write_style(Style::default());

        let _ = self.writer().flush();
    }
}
