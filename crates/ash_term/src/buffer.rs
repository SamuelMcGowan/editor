use std::ops::{Index, IndexMut};

use compact_str::{CompactString, ToCompactString};
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

use crate::style::Style;
use crate::units::OffsetU16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    symbol: CompactString,
    style: Style,
}

impl Cell {
    pub const fn empty() -> Self {
        Cell {
            symbol: CompactString::new_inline(" "),
            style: Style::EMPTY,
        }
    }

    pub fn symbol(&self) -> &str {
        &self.symbol
    }

    pub fn style(&self) -> Style {
        self.style
    }

    pub fn with_symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_compact_string();
        self
    }

    pub fn with_char(mut self, ch: char) -> Self {
        self.symbol = ch.to_compact_string();
        self
    }

    pub fn with_style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Debug)]
pub struct Buffer {
    buf: Vec<Option<Cell>>,
    size: OffsetU16,

    cursor: Option<OffsetU16>,
}

impl Clone for Buffer {
    fn clone(&self) -> Self {
        Self {
            buf: self.buf.clone(),
            size: self.size,
            cursor: self.cursor,
        }
    }

    fn clone_from(&mut self, source: &Self) {
        self.buf.clone_from(&source.buf);
        self.size = source.size;
        self.cursor = source.cursor;
    }
}

impl Buffer {
    pub fn new(size: impl Into<OffsetU16>) -> Self {
        let size: OffsetU16 = size.into();
        let buf = vec![None; size.area()];

        Self {
            buf,
            size,
            cursor: None,
        }
    }

    pub fn size(&self) -> OffsetU16 {
        self.size
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn resize_and_clear(&mut self, size: impl Into<OffsetU16>) {
        let size: OffsetU16 = size.into();

        if size != self.size {
            self.buf.clear();
            self.buf.extend(std::iter::repeat(None).take(size.area()));
            self.size = size;
        } else {
            self.buf.fill(None);
        }

        self.cursor = None;
    }

    pub fn fill(&mut self, cell: Cell) {
        self.buf.fill(Some(cell));
    }

    pub fn write_str(&mut self, pos: impl Into<OffsetU16>, s: &str, style: Style) {
        let pos: OffsetU16 = pos.into();

        let mut x = pos.x;
        for grapheme in s.graphemes(true) {
            let width = grapheme.width() as u16;

            let Some(cell) = self.get_mut([x, pos.y]) else {
                break;
            };

            *cell = Some(Cell {
                symbol: grapheme.into(),
                style,
            });

            x += width;
        }
    }

    pub fn get(&self, index: impl Into<OffsetU16>) -> Option<&Option<Cell>> {
        let index = self.index(index)?;
        self.buf.get(index)
    }

    pub fn get_mut(&mut self, index: impl Into<OffsetU16>) -> Option<&mut Option<Cell>> {
        let index = self.index(index)?;
        self.buf.get_mut(index)
    }

    fn index(&self, pos: impl Into<OffsetU16>) -> Option<usize> {
        let pos = pos.into();

        if pos.cmp_ge(self.size).either() {
            return None;
        }

        let index = pos.y as usize * self.size.x as usize + pos.x as usize;

        Some(index)
    }
}

impl<Idx: Into<OffsetU16>> Index<Idx> for Buffer {
    type Output = Option<Cell>;

    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index).expect("indices out of bounds")
    }
}

impl<Idx: Into<OffsetU16>> IndexMut<Idx> for Buffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index).expect("indices out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Cell};
    use crate::style::Style;

    #[test]
    fn simple() {
        let b = Cell::empty().with_char('b');
        let c = Cell::empty().with_char('c');

        let mut buf = Buffer::new([10, 10]);
        assert_eq!(buf.len(), 10 * 10);

        buf[[0, 0]] = Some(b.clone());
        buf[[9, 9]] = Some(c.clone());

        assert_eq!(buf[[0, 0]], Some(b));
        assert_eq!(buf[[9, 9]], Some(c));
        assert!(buf.get([10, 10]).is_none());
    }

    #[test]
    fn write_str() {
        let mut buf = Buffer::new([10, 10]);
        buf.write_str([0, 0], "🐻‍❄️!", Style::default());

        assert_eq!(buf[[0, 0]].as_ref().unwrap().symbol, "🐻\u{200d}❄️");
        assert_eq!(buf[[1, 0]].as_ref(), None);
        assert_eq!(buf[[2, 0]].as_ref(), None);
        assert_eq!(buf[[3, 0]].as_ref().unwrap().symbol, "!");
    }
}
