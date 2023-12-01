use std::ops::{Index, IndexMut};

use compact_str::{CompactString, ToCompactString};

// use unicode_segmentation::UnicodeSegmentation;
// use unicode_width::UnicodeWidthStr;
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

impl Default for &Cell {
    fn default() -> Self {
        const DEFAULT_CELL: &Cell = &Cell::empty();
        DEFAULT_CELL
    }
}

#[derive(Debug)]
pub struct Buffer {
    buf: Vec<Option<Cell>>,
    size: OffsetU16,

    pub cursor: Option<OffsetU16>,
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

    pub fn view(&mut self, set_cursor: bool) -> BufferView {
        BufferView {
            start: OffsetU16::ZERO,
            end: self.size,
            buf: self,
            set_cursor,
        }
    }
}

pub struct BufferView<'a> {
    buf: &'a mut Buffer,

    start: OffsetU16,
    end: OffsetU16,

    set_cursor: bool,
}

impl<'a> BufferView<'a> {
    pub fn size(&self) -> OffsetU16 {
        self.end - self.start
    }

    pub fn get(&self, index: impl Into<OffsetU16>) -> Option<&Option<Cell>> {
        self.buf.buf.get(self.index(index)?)
    }

    pub fn get_mut(&mut self, index: impl Into<OffsetU16>) -> Option<&mut Option<Cell>> {
        let index = self.index(index)?;
        self.buf.buf.get_mut(index)
    }

    pub fn set_cursor(&mut self, cursor: Option<impl Into<OffsetU16>>) {
        if !self.set_cursor {
            return;
        }

        let cursor = match cursor {
            Some(cursor) => {
                let cursor: OffsetU16 = cursor.into();
                let cursor = cursor.saturating_add(self.start);

                if cursor.cmp_lt(self.end).both() {
                    Some(cursor)
                } else {
                    None
                }
            }

            None => None,
        };

        self.buf.cursor = cursor;
    }

    pub fn cursor(&self) -> Option<OffsetU16> {
        self.buf.cursor
    }

    fn index(&self, index: impl Into<OffsetU16>) -> Option<usize> {
        let index = self.start.saturating_add(index.into());

        if index.cmp_gt(self.end).either() {
            return None;
        }

        Some(index.y as usize * self.buf.size.x as usize + index.x as usize)
    }
}

impl<I: Into<OffsetU16>> Index<I> for BufferView<'_> {
    type Output = Option<Cell>;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index).expect("out of bounds")
    }
}

impl<I: Into<OffsetU16>> IndexMut<I> for BufferView<'_> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.get_mut(index).expect("out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Cell};
    // use crate::style::Style;

    #[test]
    fn simple() {
        let b = Cell::empty().with_char('b');
        let c = Cell::empty().with_char('c');

        let mut buff = Buffer::new([10, 10]);
        let mut buf = buff.view(true);

        buf[[0, 0]] = Some(b.clone());
        buf[[9, 9]] = Some(c.clone());

        assert_eq!(buf[[0, 0]], Some(b));
        assert_eq!(buf[[9, 9]], Some(c));
        assert!(buf.get([10, 10]).is_none());
    }

    // #[test]
    // fn write_str() {
    //     let mut buff = Buffer::new([10, 10]);
    //     let mut buf = buff.view(true);

    //     buf.write_str([0, 0], "🐻‍❄️!", Style::default());

    //     assert_eq!(buf[[0, 0]].as_ref().unwrap().symbol, "🐻\u{200d}❄️");
    //     assert_eq!(buf[[1, 0]].as_ref(), None);
    //     assert_eq!(buf[[2, 0]].as_ref(), None);
    //     assert_eq!(buf[[3, 0]].as_ref().unwrap().symbol, "!");
    // }
}
