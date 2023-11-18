use std::io;

pub mod ansi_buffer;
mod sys;

pub struct Term {
    raw_term: sys::RawTerm,
    raw_stdout: sys::RawStdout,
}

impl Term {
    pub fn new() -> io::Result<Self> {
        Ok(Self {
            raw_term: sys::RawTerm::new()?,
            raw_stdout: sys::RawStdout,
        })
    }

    pub fn size(&self) -> io::Result<(u16, u16)> {
        self.raw_term.size()
    }
}

#[cfg(test)]
mod tests {
    use super::Term;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn get_term_size() {
        let term = Term::new().unwrap();
        let _ = term.size().unwrap();
    }
}
