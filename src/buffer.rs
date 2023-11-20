use std::ptr;

use crate::raw::RawBuf;

pub struct GapBuffer {
    inner: RawBuf,

    front_len: usize,
    back_len: usize,
}

impl GapBuffer {
    pub const fn new() -> Self {
        Self {
            inner: RawBuf::new(),
            front_len: 0,
            back_len: 0,
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.front_len + self.back_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Panics if `new_cap > isize::MAX`.
    #[inline]
    pub fn push(&mut self, byte: u8) {
        self.grow_for_push(1);
        unsafe { ptr::write(self.gap_ptr(), byte) };
        self.front_len += 1;
    }

    #[inline]
    pub fn push_back(&mut self, byte: u8) {
        self.grow_for_push(1);
        self.back_len += 1;
        unsafe { ptr::write(self.back_ptr(), byte) };
    }

    #[inline]
    pub fn pop(&mut self) -> Option<u8> {
        if self.front_len > 0 {
            self.front_len -= 1;
            Some(unsafe { ptr::read(self.gap_ptr()) })
        } else {
            None
        }
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<u8> {
        if self.back_len > 0 {
            let byte = unsafe { ptr::read(self.back_ptr()) };
            self.back_len -= 1;
            Some(byte)
        } else {
            None
        }
    }

    /// Panics if `new_cap > isize::MAX`.
    fn grow_for_push(&mut self, additional: usize) {
        let required = self
            .len()
            .checked_add(additional)
            .expect("capacity overflow");

        if let Some(new_cap) = calc_new_capacity(self.capacity(), required) {
            let prev_back_offset = self.inner.capacity() - self.back_len;

            self.inner.set_capacity(new_cap);

            // Use offset to get previous back pointer because the buffer could have moved.
            let back_ptr_prev = unsafe { self.front_ptr().add(prev_back_offset) };
            let back_ptr = self.back_ptr();

            unsafe { ptr::copy(back_ptr_prev, back_ptr, self.back_len) };
        }
    }

    #[inline]
    fn front_ptr(&mut self) -> *mut u8 {
        self.inner.as_ptr()
    }

    fn gap_ptr(&mut self) -> *mut u8 {
        unsafe { self.front_ptr().add(self.front_len) }
    }

    #[inline]
    fn back_ptr(&mut self) -> *mut u8 {
        let back_offset = self.inner.capacity() - self.back_len;
        unsafe { self.front_ptr().add(back_offset) }
    }
}

#[inline]
fn calc_new_capacity(cap: usize, required: usize) -> Option<usize> {
    if required <= cap {
        None
    } else {
        // Can't overflow as `cap < isize::MAX`.
        let min_cap = cap + (cap / 16).max(64);
        Some(required.max(min_cap))
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;
    use crate::buffer::calc_new_capacity;

    #[test]
    #[cfg_attr(miri, ignore)]
    fn calc_capacity() {
        assert_eq!(calc_new_capacity(0, 0), None);
        assert_eq!(calc_new_capacity(0, 1), Some(64));
        assert_eq!(calc_new_capacity(64, 2), None);
        assert_eq!(calc_new_capacity(64, 64), None);
        assert_eq!(calc_new_capacity(64, 65), Some(128));
        assert_eq!(calc_new_capacity(0, 123), Some(123));
        assert_eq!(calc_new_capacity(1600, 1601), Some(1700));
    }

    #[test]
    fn grow() {
        let mut buf = GapBuffer::new();

        buf.grow_for_push(1);
        assert_eq!(buf.capacity(), 64);
    }

    #[test]
    fn push_pop() {
        let mut buf = GapBuffer::new();

        buf.push(10);
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.len(), 1);

        buf.push(20);
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.len(), 2);

        assert_eq!(buf.pop(), Some(20));
        assert_eq!(buf.pop(), Some(10));
        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn push_pop_back() {
        let mut buf = GapBuffer::new();

        buf.push_back(10);
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.len(), 1);

        buf.push_back(20);
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.len(), 2);

        assert_eq!(buf.pop_back(), Some(20));
        assert_eq!(buf.pop_back(), Some(10));
        assert_eq!(buf.pop_back(), None);
    }
}
