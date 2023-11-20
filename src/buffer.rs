use std::{ptr, slice};

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
        self.reserve(1);
        unsafe { ptr::write(self.gap_ptr().cast_mut(), byte) };
        self.front_len += 1;
    }

    #[inline]
    pub fn push_back(&mut self, byte: u8) {
        self.reserve(1);
        self.back_len += 1;
        unsafe { ptr::write(self.back_ptr().cast_mut(), byte) };
    }

    #[inline]
    pub fn push_slice(&mut self, slice: &[u8]) {
        self.reserve(slice.len());

        // slice cannot alias self
        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), self.gap_ptr().cast_mut(), slice.len()) };

        self.front_len += slice.len();
    }

    #[inline]
    pub fn push_slice_back(&mut self, slice: &[u8]) {
        self.reserve(slice.len());

        self.back_len += slice.len();

        // slice cannot alias self
        unsafe {
            ptr::copy_nonoverlapping(slice.as_ptr(), self.back_ptr().cast_mut(), slice.len())
        };
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

    #[inline]
    #[must_use = "must handle how many bytes were written"]
    pub fn pop_slice(&mut self, slice: &mut [u8]) -> usize {
        let len = slice.len().min(self.front_len);

        self.front_len -= len;

        // slice cannot alias self
        unsafe { ptr::copy_nonoverlapping(self.gap_ptr(), slice.as_mut_ptr(), len) };

        len
    }

    #[inline]
    #[must_use = "must handle how many bytes were written"]
    pub fn pop_slice_back(&mut self, slice: &mut [u8]) -> usize {
        let len = slice.len().min(self.back_len);

        // slice cannot alias self
        unsafe { ptr::copy_nonoverlapping(self.back_ptr(), slice.as_mut_ptr(), len) };

        self.back_len -= len;

        len
    }

    #[inline]
    pub fn front(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.front_ptr(), self.front_len) }
    }

    #[inline]
    pub fn back(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.back_ptr(), self.back_len) }
    }

    #[inline]
    pub fn front_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.front_ptr().cast_mut(), self.front_len) }
    }

    #[inline]
    pub fn back_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.back_ptr().cast_mut(), self.back_len) }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.front_len = 0;
        self.back_len = 0;
    }

    #[inline]
    pub fn truncate_front(&mut self, len: usize) {
        self.front_len = self.front_len.min(len);
    }

    #[inline]
    pub fn truncate_back(&mut self, len: usize) {
        self.back_len = self.back_len.min(len);
    }

    /// Panics if `new_cap > isize::MAX`.
    pub fn reserve(&mut self, additional: usize) {
        let required = self
            .len()
            .checked_add(additional)
            .expect("capacity overflow");

        if let Some(new_cap) = calc_new_capacity(self.capacity(), required) {
            let prev_back_offset = self.inner.capacity() - self.back_len;

            self.inner.set_capacity(new_cap);

            // Use offset to get previous back pointer because the buffer could have moved.
            let back_ptr_prev = unsafe { self.front_ptr().add(prev_back_offset) };
            let back_ptr = self.back_ptr().cast_mut();

            unsafe { ptr::copy(back_ptr_prev, back_ptr, self.back_len) };
        }
    }

    #[inline]
    fn front_ptr(&self) -> *const u8 {
        self.inner.as_ptr()
    }

    #[inline]
    fn gap_ptr(&self) -> *const u8 {
        unsafe { self.front_ptr().add(self.front_len) }
    }

    #[inline]
    fn back_ptr(&self) -> *const u8 {
        let back_offset = self.inner.capacity() - self.back_len;
        unsafe { self.front_ptr().add(back_offset) }
    }
}

/// `cap` should be less than or equal to `isize::MAX` to avoid overflow.
#[inline]
fn calc_new_capacity(cap: usize, required: usize) -> Option<usize> {
    if required <= cap {
        None
    } else {
        // Can't overflow as `cap <= isize::MAX`.
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

        buf.reserve(1);
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

    #[test]
    fn push_and_get_slices() {
        let mut buf = GapBuffer::new();

        buf.push_slice(b"hello");
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.front_len, 5);

        buf.push_slice_back(b" world");
        assert_eq!(buf.capacity(), 64);
        assert_eq!(buf.back_len, 6);

        assert_eq!(buf.front(), b"hello");
        assert_eq!(buf.back(), b" world");
    }

    #[test]
    fn get_mut_slices() {
        let mut buf = GapBuffer::new();

        buf.push_slice(b"hello");
        buf.front_mut()[0] = b'y';
        assert_eq!(buf.front(), b"yello");

        buf.push_slice_back(b"world");
        buf.back_mut()[0] = b'q';
        assert_eq!(buf.back(), b"qorld");
    }

    #[test]
    fn pop_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");

        let mut dest = [0; 2];
        assert_eq!(buf.pop_slice(&mut dest), 2);

        assert_eq!(buf.front(), b"hel");
        assert_eq!(&dest, b"lo");
    }

    #[test]
    fn pop_slice_back() {
        let mut buf = GapBuffer::new();
        buf.push_slice_back(b"hello");

        let mut dest = [0; 2];
        assert_eq!(buf.pop_slice_back(&mut dest), 2);

        assert_eq!(buf.back(), b"llo");
        assert_eq!(&dest, b"he");
    }

    #[test]
    fn pop_too_much_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");

        let mut dest = [0; 7];
        assert_eq!(buf.pop_slice(&mut dest), 5);

        assert_eq!(&dest, b"hello\0\0");
        assert_eq!(buf.front(), b"");

        buf.push_slice_back(b"hello");

        let mut dest = [0; 7];
        assert_eq!(buf.pop_slice_back(&mut dest), 5);

        assert_eq!(&dest, b"hello\0\0");
        assert_eq!(buf.back(), b"");
    }
}
