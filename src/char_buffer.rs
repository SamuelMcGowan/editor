use std::ops::{Index, IndexMut};

use crate::style::Style;

#[derive(Debug, Clone, Copy)]
pub struct Cell {
    pub c: char,
    pub style: Style,
}

impl Cell {
    fn char(c: char) -> Self {
        Self {
            c,
            style: Style::default(),
        }
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::char(' ')
    }
}

pub struct Buffer {
    data: Vec<Cell>,

    width: u16,
    height: u16,

    pub cursor: Option<(u16, u16)>,
}

impl Buffer {
    pub fn new(width: u16, height: u16) -> Self {
        let len = width as usize * height as usize;
        let data = vec![Cell::default(); len];

        Self {
            data,
            width,
            height,
            cursor: None,
        }
    }

    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
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

    pub fn get(&self, x: u16, y: u16) -> Option<&Cell> {
        let index = self.index(x, y)?;
        self.data.get(index)
    }

    pub fn get_mut(&mut self, x: u16, y: u16) -> Option<&mut Cell> {
        let index = self.index(x, y)?;
        self.data.get_mut(index)
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.width || y > self.height {
            return None;
        }

        let index = y as usize * self.width as usize + x as usize;

        Some(index)
    }
}

impl Index<[u16; 2]> for Buffer {
    type Output = Cell;

    fn index(&self, index: [u16; 2]) -> &Self::Output {
        self.get(index[0], index[1]).expect("indices out of bounds")
    }
}

impl IndexMut<[u16; 2]> for Buffer {
    fn index_mut(&mut self, index: [u16; 2]) -> &mut Self::Output {
        self.get_mut(index[0], index[1])
            .expect("indices out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::{Buffer, Cell};

    #[test]
    #[cfg_attr(miri, ignore)]
    fn simple() {
        let b = Cell::char('b');
        let c = Cell::char('c');

        let mut arr = Buffer::new(10, 10);
        assert_eq!(arr.len(), 10 * 10);

        arr[[0, 0]] = b;
        arr[[9, 9]] = c;

        assert_eq!(arr[[0, 0]].c, 'b');
        assert_eq!(arr[[9, 9]].c, 'c');
        assert!(arr.get(10, 10).is_none());
    }
}
