mod raw;

use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
use std::{ptr, slice};

use self::raw::RawBuf;

pub struct GapBuffer {
    inner: RawBuf,
    front_len: usize,
    back_len: usize,
}

impl GapBuffer {
    /// Create a new, empty gap buffer (without allocating).
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: RawBuf::new(),
            front_len: 0,
            back_len: 0,
        }
    }

    /// Create a new gap buffer with the given capacity.
    ///
    /// # Panics
    /// Panics if the capacity overflows `isize::MAX`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RawBuf::with_capacity(capacity),
            front_len: 0,
            back_len: 0,
        }
    }

    /// Create from an existing vec, retaining excess capacity.
    ///
    /// Only works for vecs that use the global allocator (we have to deallocate
    /// its contents afterwards!)
    #[inline]
    pub fn from_vec(v: Vec<u8>) -> Self {
        let len = v.len();
        let inner = RawBuf::from_vec(v);

        Self {
            inner,
            front_len: len,
            back_len: 0,
        }
    }

    /// Create from a slice.
    #[inline]
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut buf = Self::new();
        buf.push_slice(slice);
        buf
    }

    /// The total capacity of the gap buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// The total number of bytes in the gap buffer (not including the gap).
    #[inline]
    pub fn len(&self) -> usize {
        self.front_len + self.back_len
    }

    /// Whether the gap buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a byte to the bytes before the gap.
    ///
    /// Panics if the new length overflows `isize::MAX`;
    #[inline]
    pub fn push(&mut self, byte: u8) {
        self.reserve(1);

        unsafe { ptr::write(self.gap_ptr(), byte) };

        self.front_len += 1;
    }

    /// Push a byte to the bytes after the gap.
    #[inline]
    pub fn push_back(&mut self, byte: u8) {
        self.reserve(1);

        self.back_len += 1;

        unsafe { ptr::write(self.back_ptr(), byte) }
    }

    /// Push a slice to the bytes before the gap.
    #[inline]
    pub fn push_slice(&mut self, slice: &[u8]) {
        self.reserve(slice.len());

        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), self.gap_ptr(), slice.len()) };

        self.front_len += slice.len();
    }

    /// Push a slice to the bytes after the gap.
    #[inline]
    pub fn push_slice_back(&mut self, slice: &[u8]) {
        self.reserve(slice.len());
        self.back_len += slice.len();

        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), self.back_ptr(), slice.len()) }
    }

    /// Pop a value from the bytes before the gap.
    #[inline]
    pub fn pop(&mut self) -> Option<u8> {
        if self.front_len == 0 {
            return None;
        }

        self.front_len -= 1;

        Some(unsafe { ptr::read(self.gap_ptr()) })
    }

    /// Pop a value from the bytes after the gap.
    #[inline]
    pub fn pop_back(&mut self) -> Option<u8> {
        if self.back_len == 0 {
            return None;
        }

        let byte = unsafe { ptr::read(self.back_ptr()) };
        self.back_len -= 1;

        Some(byte)
    }

    /// Get a reference to the byte at `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&u8> {
        let p = self.index_to_ptr(index)?;

        // Safety: pointer is valid for returned lifetime.
        Some(unsafe { &*p })
    }

    /// Get a mutable reference to the byte at `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut u8> {
        let p = self.index_to_ptr(index)?;

        // Safety: pointer is valid for returned lifetime.
        Some(unsafe { &mut *p })
    }

    /// The bytes before the gap.
    #[inline]
    pub fn front(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.front_ptr(), self.front_len) }
    }

    /// The bytes before the gap, mutably.
    #[inline]
    pub fn front_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.front_ptr(), self.front_len) }
    }

    /// The bytes after the gap.
    #[inline]
    pub fn back(&self) -> &[u8] {
        unsafe { slice::from_raw_parts(self.back_ptr(), self.back_len) }
    }

    /// The bytes after the gap, mutably.
    #[inline]
    pub fn back_mut(&mut self) -> &mut [u8] {
        unsafe { slice::from_raw_parts_mut(self.back_ptr(), self.back_len) }
    }

    /// Set the position of the gap.
    ///
    /// This may be an expensive operation if the position is moved far.
    ///
    /// # Panics
    /// Panics if the index is out of bounds.
    pub fn set_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "index out of bounds");

        if self.capacity() == 0 {
            return;
        }

        match index.cmp(&self.front_len) {
            Ordering::Less => {
                let src_ptr = unsafe { self.front_ptr().add(index) };
                let len = self.front_len - index;

                self.front_len = index;
                self.back_len += len;

                unsafe { ptr::copy(src_ptr, self.back_ptr(), len) };
            }

            Ordering::Equal => {}

            Ordering::Greater => {
                let src_ptr = self.back_ptr();
                let dest_ptr = self.gap_ptr();
                let len = index - self.front_len;

                self.front_len = index;
                self.back_len -= len;

                unsafe { ptr::copy(src_ptr, dest_ptr, len) };
            }
        }
    }

    /// Ensure that there are at least `additional` bytes of space available in
    /// the gap, allocating if necessary.
    ///
    /// Will invalidate any pointers into the buffer if it reallocates!
    ///
    /// # Panics
    /// Panics if the length overflows.
    pub fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }

        let required_len = self
            .len()
            .checked_add(additional)
            .expect("length overflowed");

        let prev_back_offset = self.capacity() - self.back_len;

        self.inner.resize_to_fit(required_len);

        // Use offset to get back pointer because the buffer could have moved.
        // `prev_back_len` must be <= capacity so can't overflow (new capacity can't
        // have shrunk!)
        let prev_back_ptr = unsafe { self.front_ptr().add(prev_back_offset) };
        let back_ptr = self.back_ptr();

        if !ptr::eq(back_ptr, prev_back_ptr) {
            unsafe { ptr::copy(prev_back_ptr, back_ptr, self.back_len) };
        }
    }

    #[inline]
    fn front_ptr(&self) -> *mut u8 {
        self.inner.as_ptr()
    }

    #[inline]
    fn gap_ptr(&self) -> *mut u8 {
        // Safety: resulting pointer is within the allocation
        unsafe { self.front_ptr().add(self.front_len) }
    }

    #[inline]
    fn back_ptr(&self) -> *mut u8 {
        let back_offset = self.capacity() - self.back_len;

        // Safety: resulting pointer is within the allocation
        unsafe { self.front_ptr().add(back_offset) }
    }

    #[inline]
    fn index_to_ptr(&self, index: usize) -> Option<*mut u8> {
        if index >= self.len() {
            return None;
        }

        let index = if index > self.front_len {
            index + self.gap_len()
        } else {
            index
        };

        Some(unsafe { self.front_ptr().add(index) })
    }

    #[inline]
    fn gap_len(&self) -> usize {
        self.capacity() - self.len()
    }
}

impl From<Vec<u8>> for GapBuffer {
    #[inline]
    fn from(v: Vec<u8>) -> Self {
        Self::from_vec(v)
    }
}

impl From<&[u8]> for GapBuffer {
    #[inline]
    fn from(slice: &[u8]) -> Self {
        Self::from_slice(slice)
    }
}

impl Index<usize> for GapBuffer {
    type Output = u8;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl IndexMut<usize> for GapBuffer {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;

    #[test]
    fn from_vec() {
        let v = vec![0, 1, 2, 3, 4];
        let cap = v.capacity();

        let buf = GapBuffer::from(v);

        assert_eq!(buf.capacity(), cap);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.front_len, 5);
        assert_eq!(buf.back_len, 0);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), cap);

        assert_eq!(buf.front(), &[0, 1, 2, 3, 4]);
        assert_eq!(buf.back(), &[]);
    }

    #[test]
    fn zero_capacity() {
        let buf = GapBuffer::with_capacity(0);
        assert_eq!(buf.capacity(), 0);
    }

    #[test]
    fn resize() {
        let mut buf = GapBuffer::with_capacity(8);
        buf.push_slice(b"hell");
        buf.push_slice_back(b"yeah");

        assert_eq!(buf.capacity(), 8);
        assert_eq!(buf.front(), b"hell");
        assert_eq!(buf.back(), b"yeah");

        buf.push_back(b' ');

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.front(), b"hell");
        assert_eq!(buf.back(), b" yeah");
    }

    #[test]
    fn push_pop() {
        let mut buf = GapBuffer::new();

        for i in 0..10 {
            buf.push(i);
        }

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.front_len, 10);
        assert_eq!(buf.back_len, 0);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), 16);

        assert_eq!(buf.front(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        for i in (0..10).rev() {
            assert_eq!(buf.pop(), Some(i));
        }

        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn push_pop_back() {
        let mut buf = GapBuffer::new();

        for i in 0..10 {
            buf.push_back(i);
        }

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.front_len, 0);
        assert_eq!(buf.back_len, 10);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), 6);

        assert_eq!(buf.back(), &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

        for i in (0..10).rev() {
            assert_eq!(buf.pop_back(), Some(i));
        }

        assert_eq!(buf.pop_back(), None);
    }

    #[test]
    fn push_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello ");
        buf.push_slice(b"world");

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.front_len, 11);
        assert_eq!(buf.back_len, 0);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), 16);

        assert_eq!(buf.front(), b"hello world");
        assert_eq!(buf.back(), b"");
    }

    #[test]
    fn push_slice_back() {
        let mut buf = GapBuffer::new();
        buf.push_slice_back(b"world");
        buf.push_slice_back(b"hello ");

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.front_len, 0);
        assert_eq!(buf.back_len, 11);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), 5);

        assert_eq!(buf.front(), b"");
        assert_eq!(buf.back(), b"hello world");
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
        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.front_len, 0);
        assert_eq!(buf.back_len, 10);
        assert_eq!(ptr_diff(buf.back_ptr(), buf.front_ptr()), 6);

        assert_eq!(buf.front(), &[]);
        assert_eq!(buf.back(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    #[should_panic = "index out of bounds"]
    fn set_gap_out_of_bounds() {
        let mut buf = GapBuffer::new();
        buf.set_gap(1);
    }

    #[test]
    fn get() {
        let mut buf = GapBuffer::new();

        buf.push_slice(b"hello");
        buf.set_gap(1);

        assert_eq!(buf.front(), b"h");
        assert_eq!(buf.back(), b"ello");

        for (i, mut byte) in b"hello".iter().copied().enumerate() {
            assert_eq!(&buf[i], &byte);
            assert_eq!(&mut buf[i], &mut byte);
        }

        assert_eq!(buf.get(5), None);
    }

    #[test]
    fn mutable_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(b"hello");
        buf.front_mut()[0] = b'f';
        assert_eq!(buf.front(), b"fello");
    }

    fn ptr_diff(a: *const u8, b: *const u8) -> usize {
        a as usize - b as usize
    }
}
