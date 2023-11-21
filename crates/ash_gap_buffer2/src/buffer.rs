use std::cmp::Ordering;
use std::{ptr, slice};

use crate::iter::SkipGapIter;
use crate::raw::RawBuf;

pub struct GapBuffer {
    inner: RawBuf,

    front_len: usize,
    back_len: usize,
}

impl Default for GapBuffer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl GapBuffer {
    pub const fn new() -> Self {
        Self {
            inner: RawBuf::new(),
            front_len: 0,
            back_len: 0,
        }
    }

    /// # Panics
    /// Panics if `capacity > isize::MAX`.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RawBuf::with_capacity(capacity),
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

    /// # Panics
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

    pub fn set_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "index out of bounds");

        // If capacity is zero, `index` will equal `self.front_len`, so no capacity
        // check needed.

        match index.cmp(&self.front_len) {
            Ordering::Less => {
                let src_ptr = unsafe { self.front_ptr().add(index) };
                let len = self.front_len - index;

                self.front_len = index;
                self.back_len += len;

                unsafe { ptr::copy(src_ptr, self.back_ptr().cast_mut(), len) };
            }

            Ordering::Equal => {}

            Ordering::Greater => {
                let src_ptr = self.back_ptr();
                let dest_ptr = self.gap_ptr().cast_mut();
                let len = index - self.front_len;

                self.front_len = index;
                self.back_len -= len;

                unsafe { ptr::copy(src_ptr, dest_ptr, len) };
            }
        }
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
    pub fn front_and_back_mut(&mut self) -> (&mut [u8], &mut [u8]) {
        unsafe {
            let front = slice::from_raw_parts_mut(self.front_ptr().cast_mut(), self.front_len);
            let back = slice::from_raw_parts_mut(self.back_ptr().cast_mut(), self.back_len);

            (front, back)
        }
    }

    #[inline]
    pub fn get(&self, index: usize) -> Option<&u8> {
        self.index_to_ptr(index).map(|ptr| unsafe { &*ptr })
    }

    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        self.index_to_ptr(index)
            .map(|ptr| unsafe { &mut *ptr.cast_mut() })
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

    /// # Panics
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
            let prev_back_ptr = unsafe { self.front_ptr().add(prev_back_offset) };
            let back_ptr = self.back_ptr().cast_mut();

            unsafe { ptr::copy(prev_back_ptr, back_ptr, self.back_len) };
        }
    }

    pub fn shrink_to_fit(&mut self) {
        self.shrink_to(self.len());
    }

    /// # Panics
    /// Panics if `capacity` is smaller than the current length.
    pub fn shrink_to(&mut self, capacity: usize) {
        assert!(capacity >= self.len(), "capacity smaller than length");

        let new_back_offset = capacity - self.back_len;
        let new_back_ptr = unsafe { self.front_ptr().cast_mut().add(new_back_offset) };

        unsafe { ptr::copy(self.back_ptr(), new_back_ptr, self.back_len) };

        self.inner.set_capacity(capacity);
    }

    #[inline]
    pub fn iter(&self) -> SkipGapIter<slice::Iter<'_, u8>> {
        SkipGapIter::new(self.front().iter(), self.back().iter())
    }

    #[inline]
    pub fn iter_mut(&mut self) -> SkipGapIter<slice::IterMut<'_, u8>> {
        let (front, back) = self.front_and_back_mut();
        SkipGapIter::new(front.iter_mut(), back.iter_mut())
    }

    #[inline]
    pub fn into_vec(mut self) -> Vec<u8> {
        // `Vec` should handle this case (dangling pointer) fine, but the invariants of
        // `Vec::from_raw_parts` don't mention it so we'll avoid it.
        if self.capacity() == 0 {
            return vec![];
        }

        self.shrink_to_fit();

        // Safety: all invariants upheld by data structure and above `shrink_to_fit`
        // call.
        let v = unsafe {
            Vec::from_raw_parts(self.front_ptr().cast_mut(), self.len(), self.capacity())
        };

        std::mem::forget(self);

        v
    }

    #[inline]
    pub(crate) fn front_len(&self) -> usize {
        self.front_len
    }

    #[inline]
    pub(crate) fn back_len(&self) -> usize {
        self.back_len
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

    #[inline]
    fn index_to_ptr(&self, index: usize) -> Option<*const u8> {
        if index < self.front_len {
            Some(unsafe { self.front_ptr().add(index) })
        } else {
            let index = index - self.front_len;
            if index < self.back_len {
                Some(unsafe { self.back_ptr().add(index) })
            } else {
                None
            }
        }
    }
}

impl From<Vec<u8>> for GapBuffer {
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        let len = v.len();
        Self {
            inner: v.into(),
            front_len: len,
            back_len: 0,
        }
    }
}

impl From<&[u8]> for GapBuffer {
    #[inline]
    fn from(slice: &[u8]) -> Self {
        let mut buf = Self::new();
        buf.push_slice(slice);
        buf
    }
}

impl<const N: usize> From<&[u8; N]> for GapBuffer {
    #[inline]
    fn from(slice: &[u8; N]) -> Self {
        let mut buf = Self::new();
        buf.push_slice(slice.as_slice());
        buf
    }
}

impl From<GapBuffer> for Vec<u8> {
    #[inline]
    fn from(buf: GapBuffer) -> Self {
        buf.into_vec()
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
    fn from_vec() {
        let mut buf = GapBuffer::from(b"hello world".to_vec());
        assert_eq!(buf.front(), b"hello world");
        assert_eq!(buf.back(), b"");

        buf.push_slice_back(b"-wide-web");
        assert_eq!(buf.front(), b"hello world");
        assert_eq!(buf.back(), b"-wide-web");
    }

    #[test]
    fn into_vec() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.push_slice_back(b" world");
        assert!(buf.len() < buf.capacity());

        let v = buf.into_vec();
        assert_eq!(v.as_slice(), b"hello world");
    }

    #[test]
    fn grow() {
        let mut buf = GapBuffer::new();

        buf.reserve(1);
        assert_eq!(buf.capacity(), 64);
    }

    #[test]
    fn grow_again() {
        let mut buf = GapBuffer::with_capacity(10);

        buf.push(1);
        buf.push_back(2);
        assert_eq!(buf.capacity(), 10);

        buf.reserve(10);
        assert_eq!(buf.capacity(), 74);

        assert_eq!(buf.front(), &[1]);
        assert_eq!(buf.back(), &[2]);
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
    fn push_empty_slices() {
        let mut buf = GapBuffer::new();

        buf.push_slice(b"");
        assert_eq!(buf.front(), b"");

        buf.push_slice_back(b"");
        assert_eq!(buf.back(), b"");

        buf.push(1);
        buf.push_back(2);

        assert_eq!(buf.front(), &[1]);
        assert_eq!(buf.back(), &[2]);
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
    fn set_gap() {
        let mut buf = GapBuffer::new();
        for i in 0..10 {
            buf.push(i);
        }

        assert_eq!(buf.front(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(buf.back(), &[]);

        buf.set_gap(0);
        assert_eq!(buf.front_len, 0);
        assert_eq!(buf.back_len, 10);
        assert_eq!(buf.front(), &[]);
        assert_eq!(buf.back(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        buf.set_gap(5);
        assert_eq!(buf.front_len, 5);
        assert_eq!(buf.back_len, 5);
        assert_eq!(buf.front(), &[0, 1, 2, 3, 4]);
        assert_eq!(buf.back(), &[5, 6, 7, 8, 9]);

        buf.set_gap(10);
        assert_eq!(buf.front_len, 10);
        assert_eq!(buf.back_len, 0);
        assert_eq!(buf.front(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(buf.back(), &[]);
    }

    #[test]
    fn set_gap_empty() {
        let mut buf = GapBuffer::new();
        buf.set_gap(0);
    }

    #[test]
    #[should_panic = "index out of bounds"]
    fn set_gap_out_of_bounds() {
        let mut buf = GapBuffer::new();
        buf.set_gap(1);
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
    fn get_both_mut_slices() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.push_slice_back(b"world");

        let (front, back) = buf.front_and_back_mut();
        front[0] = b'y';
        back[0] = b'q';

        assert_eq!(buf.front(), b"yello");
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

    #[test]
    fn get() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.push_slice_back(b" world");

        for (i, &b) in b"hello world".iter().enumerate() {
            let mut byte = b;
            assert_eq!(buf.get(i), Some(&byte));
            assert_eq!(buf.get_mut(i), Some(&mut byte));
        }

        assert_eq!(buf.get(11), None);
    }

    #[test]
    fn iterators() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.push_slice_back(b" world");

        let mut bytes = buf.iter();
        for b in b"hello world".iter() {
            assert_eq!(bytes.next(), Some(b));
        }

        assert_eq!(bytes.next(), None);

        let mut bytes_mut = buf.iter_mut();
        for &b in b"hello world".iter() {
            let mut byte = b;
            assert_eq!(bytes_mut.next(), Some(&mut byte));
        }
    }

    #[test]
    fn shrink_to() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.push_slice_back(b" world");

        assert_eq!(buf.capacity(), 64);

        buf.shrink_to_fit();
        assert_eq!(buf.capacity(), 11);

        assert_eq!(buf.front(), b"hello");
        assert_eq!(buf.back(), b" world");
    }

    #[test]
    #[should_panic = "capacity smaller than length"]
    fn shrink_too_much() {
        let mut buf = GapBuffer::from(b"hello");
        buf.shrink_to(4);
    }
}
