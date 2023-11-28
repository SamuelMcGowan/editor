/// An iterator over lines and linebreaks, handling newlines and carriage
/// returns.
pub struct LineSegments<'a> {
    s: &'a str,
}

impl<'a> LineSegments<'a> {
    pub fn new(s: &'a str) -> Self {
        Self { s }
    }
}

impl<'a> Iterator for LineSegments<'a> {
    type Item = LineSegment<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        if let Some(rest) = self.s.strip_prefix("\r\n") {
            self.s = rest;
            return Some(LineSegment::LineBreak);
        }

        if let Some(rest) = self.s.strip_prefix(is_line_break) {
            self.s = rest;
            return Some(LineSegment::LineBreak);
        }

        let next_index = self.s.find(is_line_break).unwrap_or(self.s.len());
        let (line, rest) = self.s.split_at(next_index);
        self.s = rest;

        Some(LineSegment::Line(line))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, Some(self.s.len()))
    }
}

impl<'a> DoubleEndedIterator for LineSegments<'a> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.s.is_empty() {
            return None;
        }

        if let Some(rest) = self.s.strip_suffix("\r\n") {
            self.s = rest;
            return Some(LineSegment::LineBreak);
        }

        if let Some(rest) = self.s.strip_suffix(is_line_break) {
            self.s = rest;
            return Some(LineSegment::LineBreak);
        }

        let prev_index = self.s.rfind(is_line_break).unwrap_or(0);
        let (line, rest) = self.s.split_at(prev_index + 1);
        self.s = rest;

        Some(LineSegment::Line(line))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LineSegment<'a> {
    Line(&'a str),
    LineBreak,
}

fn is_line_break(ch: char) -> bool {
    matches!(ch, '\r' | '\n')
}
