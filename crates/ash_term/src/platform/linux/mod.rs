use self::raw_term::RawTerm;
use super::ansi::AnsiWriter;
use super::ansi_event::AnsiEvents;
use super::{Terminal, Writer};
use crate::style::Style;

mod raw_term;

pub struct LinuxTerminal {
    ansi_raw_term: AnsiWriter<RawTerm>,
    ansi_events: AnsiEvents,
}

impl Terminal for LinuxTerminal {
    type Writer = AnsiWriter<RawTerm>;
    type Events = AnsiEvents;

    #[inline]
    fn init() -> std::io::Result<Self> {
        let mut term = Self {
            ansi_raw_term: AnsiWriter::new(RawTerm::new()?),
            ansi_events: AnsiEvents::default(),
        };

        term.writer().clear_all();
        term.writer().flush()?;

        Ok(term)
    }

    fn size(&self) -> std::io::Result<(u16, u16)> {
        self.ansi_raw_term.inner().size()
    }

    #[inline]
    fn writer(&mut self) -> &mut Self::Writer {
        &mut self.ansi_raw_term
    }

    #[inline]
    fn events(&mut self) -> &mut Self::Events {
        &mut self.ansi_events
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
