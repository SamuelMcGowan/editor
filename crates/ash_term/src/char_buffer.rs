use std::ops::{Index, IndexMut};

use super::style::Style;
use crate::units::Offset;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Cell {
    fn new(c: char, style: Style) -> Self {
        Self { c, style }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::new(' ', Style::default())
    }
}

pub struct Buffer {
    data: Vec<Cell>,

    size: Offset,

    pub cursor: Option<Offset>,
}

impl Buffer {
    pub fn new(size: impl Into<Offset>) -> Self {
        let size = size.into();
        let data = vec![Cell::default(); size.area()];

        Self {
            data,
            size,
            cursor: None,
        }
    }

    pub fn size(&self) -> Offset {
        self.size
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn as_slice(&self) -> &[Cell] {
        &self.data
    }

    pub fn get(&self, index: impl Into<Offset>) -> Option<&Cell> {
        let index = self.index(index)?;
        self.data.get(index)
    }

    pub fn get_mut(&mut self, index: impl Into<Offset>) -> Option<&mut Cell> {
        let index = self.index(index)?;
        self.data.get_mut(index)
    }

    fn index(&self, pos: impl Into<Offset>) -> Option<usize> {
        let pos = pos.into();

        if pos.ge(self.size).either() {
            return None;
        }

        let index = pos.y as usize * self.size.x as usize + pos.x as usize;

        Some(index)
    }
}

impl<Idx: Into<Offset>> Index<Idx> for Buffer {
    type Output = Cell;

    fn index(&self, index: Idx) -> &Self::Output {
        self.get(index).expect("indices out of bounds")
    }
}

impl<Idx: Into<Offset>> IndexMut<Idx> for Buffer {
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        self.get_mut(index).expect("indices out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Cell};
    use crate::style::Style;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn simple() {
        let b = Cell::new('b', Style::default());
        let c = Cell::new('c', Style::default());

        let mut arr = Buffer::new([10, 10]);
        assert_eq!(arr.len(), 10 * 10);

        arr[[0, 0]] = b;
        arr[[9, 9]] = c;

        assert_eq!(arr[[0, 0]].c, 'b');
        assert_eq!(arr[[9, 9]].c, 'c');
        assert!(arr.get([10, 10]).is_none());
    }
}
