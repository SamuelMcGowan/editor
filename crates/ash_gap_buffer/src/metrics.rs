pub trait TextMetrics {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn char_to_byte(&self, offset: usize) -> Option<usize>;
    fn byte_to_char(&self, offset: usize) -> Option<usize>;
    fn byte_to_grapheme(&self, offset: usize) -> Option<usize>;

    fn byte_to_line(&self, offset: usize) -> Option<Line>;

    fn byte_to_line_col(&self, offset: usize) -> Option<(usize, usize)> {
        let line = self.byte_to_line(offset)?;

        let graphemes_start = self.byte_to_grapheme(line.byte_offset)?;
        let graphemes_end = self.byte_to_grapheme(offset)?;

        let col = graphemes_end - graphemes_start;

        Some((line.line, col))
    }
}

pub struct Line {
    pub line: usize,
    pub byte_offset: usize,
}
