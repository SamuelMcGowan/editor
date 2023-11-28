use std::ops::{Index, IndexMut};

use super::style::Style;
use crate::units::Vec2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Cell {
    pub fn new(c: char, style: Style) -> Self {
        Self { c, style }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', Style::default())
    }
}

#[derive(Debug)]
pub struct CharBuffer {
    buf: Vec<Option<Cell>>,

    size: Vec2<u16>,

    pub cursor: Option<Vec2<u16>>,
}

impl Clone for CharBuffer {
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

impl CharBuffer {
    pub fn new(size: impl Into<Vec2<u16>>) -> Self {
        let size = size.into();
        let data = vec![None; size.area::<usize>()];

        Self {
            buf: data,
            size,
            cursor: None,
        }
    }

    pub fn resize_and_clear(&mut self, size: impl Into<Vec2<u16>>, cell: Option<Cell>) {
        let size: Vec2<u16> = size.into();

        if size != self.size {
            self.buf.clear();
            self.buf
                .extend(std::iter::repeat(cell).take(size.area::<usize>()));
            self.size = size;
        } else {
            self.buf.fill(cell);
        }

        self.cursor = None;
    }

    pub fn size(&self) -> Vec2<u16> {
        self.size
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[Option<Cell>] {
        &self.buf
    }

    pub fn get(&self, index: impl Into<Vec2<u16>>) -> Option<&Option<Cell>> {
        let index = self.index(index)?;
        self.buf.get(index)
    }

    pub fn get_mut(&mut self, index: impl Into<Vec2<u16>>) -> Option<&mut Option<Cell>> {
        let index = self.index(index)?;
        self.buf.get_mut(index)
    }

    fn index(&self, pos: impl Into<Vec2<u16>>) -> Option<usize> {
        let pos = pos.into();

        if pos.ge(self.size).either() {
            return None;
        }

        let index = pos.y as usize * self.size.x as usize + pos.x as usize;

        Some(index)
    }
}

impl<Idx: Into<Vec2<u16>>> Index<Idx> for CharBuffer {
    type Output = Option<Cell>;

    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index).expect("indices out of bounds")
    }
}

impl<Idx: Into<Vec2<u16>>> IndexMut<Idx> for CharBuffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index).expect("indices out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{Cell, CharBuffer};
    use crate::style::Style;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn simple() {
        let b = Cell::new('b', Style::default());
        let c = Cell::new('c', Style::default());

        let mut arr = CharBuffer::new([10, 10]);
        assert_eq!(arr.len(), 10 * 10);

        arr[[0, 0]] = Some(b);
        arr[[9, 9]] = Some(c);

        assert_eq!(arr[[0, 0]], Some(b));
        assert_eq!(arr[[9, 9]], Some(c));
        assert!(arr.get([10, 10]).is_none());
    }
}
