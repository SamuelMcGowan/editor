use compact_str::CompactString;

use crate::style::Style;
use crate::units::OffsetU16;

#[derive(Debug, Clone, PartialEq, Eq)]
struct Cell {
    symbol: CompactString,
    style: Style,
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

    pub fn size(&self) -> OffsetU16 {
        self.size
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn get(&self, index: impl Into<OffsetU16>) -> Option<&Option<Cell>> {
        let index = self.index(index)?;
        self.buf.get(index)
    }

    fn get_mut(&mut self, index: impl Into<OffsetU16>) -> Option<&mut Option<Cell>> {
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
