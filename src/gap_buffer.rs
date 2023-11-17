use std::alloc::{self, Layout};
use std::ptr;

const GROW_BY: usize = 10;

#[derive(Debug)]
pub struct Buffer {
    left: *mut u8,
    left_len: usize,

    right: *mut u8,
    right_len: usize,

    cap: usize,
}

impl Default for Buffer {
    fn default() -> Self {
        Self::new()
    }
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            left: ptr::null_mut(),
            left_len: 0,

            right: ptr::null_mut(),
            right_len: 0,

            cap: 0,
        }
    }

    pub fn push(&mut self, byte: u8) {
        if self.gap_len() == 0 {
            self.grow_gap();
        }

        unsafe {
            let gap_ptr = self.left.add(self.left_len);
            ptr::write(gap_ptr, byte);
        }

        self.left_len += 1;
    }

    pub fn move_gap(&mut self, index: usize) {
        if index < self.left_len {
            let src_ptr = unsafe { self.left.add(index) };
            let len = self.left_len - index;

            self.left_len -= len;
            self.right = unsafe { self.right.sub(len) };
            self.right_len += len;

            unsafe { ptr::copy(src_ptr, self.right, len) }
        } else {
            todo!("cannot move pointer right");
        }
    }

    pub fn len(&self) -> usize {
        self.left_len + self.right_len
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn gap_len(&self) -> usize {
        self.capacity() - self.len()
    }

    fn grow_gap(&mut self) {
        let new_cap = self.cap + GROW_BY;
        let new_layout = Layout::array::<u8>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe { alloc::realloc(self.left, old_layout, new_layout.size()) }
        };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        let right_old = unsafe { new_ptr.add(self.cap - self.right_len) };

        self.left = new_ptr;
        self.right = unsafe { new_ptr.add(new_cap - self.right_len) };

        self.cap = new_cap;

        unsafe { ptr::copy(right_old, self.right, self.right_len) };
    }

    pub fn left(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.left, self.left_len) }
    }

    pub fn right(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.right, self.right_len) }
    }
}

impl Drop for Buffer {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        let old_layout = Layout::array::<u8>(self.cap).unwrap();
        unsafe { alloc::dealloc(self.left, old_layout) };
    }
}

#[cfg(test)]
mod tests {
    use super::Buffer;

    #[test]
    fn simple() {
        let mut buffer = Buffer::new();
        buffer.push(12);

        assert_eq!(buffer.left_len, 1);
        assert_eq!(buffer.right_len, 0);
        assert_eq!(buffer.cap, 10);

        assert_eq!(buffer.right as usize - buffer.left as usize, 10);
        assert_eq!(buffer.left(), &[12]);
        assert_eq!(buffer.right(), &[]);

        for i in 1..=10 {
            buffer.push(i);
        }

        assert_eq!(buffer.left_len, 11);
        assert_eq!(buffer.right_len, 0);
        assert_eq!(buffer.cap, 20);
        assert_eq!(buffer.right as usize - buffer.left as usize, 20);

        buffer.move_gap(4);

        assert_eq!(buffer.right as usize - buffer.left as usize, 13);
        assert_eq!(buffer.left_len, 4);
        assert_eq!(buffer.right_len, 7);
        assert_eq!(buffer.left(), &[12, 1, 2, 3]);
        assert_eq!(buffer.right(), &[4, 5, 6, 7, 8, 9, 10]);
    }
}
