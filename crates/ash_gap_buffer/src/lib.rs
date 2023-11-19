mod raw;

use std::cmp::Ordering;
use std::ops::{Index, IndexMut};
use std::{ptr, slice};

use self::raw::RawVec;

pub struct GapVec<T> {
    inner: RawVec<T>,
    front_len: usize,
    back_len: usize,
}

impl<T> GapVec<T> {
    /// Create a new, empty gap buffer (without allocating).
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: RawVec::new(),
            front_len: 0,
            back_len: 0,
        }
    }

    /// Create a new gap buffer with the given capacity.
    ///
    /// # Panics
    /// Panics if the required capacity in bytes overflows `isize::MAX` or if
    /// `T` is a zero-sized type.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RawVec::with_capacity(capacity),
            front_len: 0,
            back_len: 0,
        }
    }

    /// Create from an existing vec, retaining excess capacity.
    ///
    /// Only works for vecs that use the global allocator (we have to deallocate
    /// its contents afterwards!)
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub fn from_vec(v: Vec<T>) -> Self {
        let len = v.len();
        let inner = RawVec::from_vec(v);

        Self {
            inner,
            front_len: len,
            back_len: 0,
        }
    }

    /// Create from a slice.
    ///
    /// # Panics
    /// Panics if `T` is a zero-sized type.
    #[inline]
    pub fn from_slice(slice: &[T]) -> Self {
        let mut buf = Self::new();
        buf.push_slice(slice);
        buf
    }

    /// The total capacity of the gap buffer.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    /// The total number of elements in the gap buffer (not including the gap).
    #[inline]
    pub fn len(&self) -> usize {
        self.front_len + self.back_len
    }

    /// Whether the gap buffer is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Push a element to the elements before the gap.
    ///
    /// Panics if the required capacity in bytes overflows `isize::MAX`
    #[inline]
    pub fn push(&mut self, element: T) {
        self.reserve(1);

        unsafe { ptr::write(self.gap_ptr(), element) };

        self.front_len += 1;
    }

    /// Push a element to the elements after the gap.
    ///
    /// Panics if the required capacity in bytes overflows `isize::MAX`
    #[inline]
    pub fn push_back(&mut self, element: T) {
        self.reserve(1);

        self.back_len += 1;

        unsafe { ptr::write(self.back_ptr(), element) }
    }

    /// Push a slice to the elements before the gap.
    ///
    /// Panics if the required capacity in bytes overflows `isize::MAX`
    #[inline]
    pub fn push_slice(&mut self, slice: &[T]) {
        self.reserve(slice.len());

        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), self.gap_ptr(), slice.len()) };

        self.front_len += slice.len();
    }

    /// Push a slice to the elements after the gap.
    ///
    /// Panics if the required capacity in bytes overflows `isize::MAX`
    #[inline]
    pub fn push_slice_back(&mut self, slice: &[T]) {
        self.reserve(slice.len());
        self.back_len += slice.len();

        unsafe { ptr::copy_nonoverlapping(slice.as_ptr(), self.back_ptr(), slice.len()) }
    }

    /// Pop a value from the elements before the gap.
    #[inline]
    pub fn pop(&mut self) -> Option<T> {
        if self.front_len == 0 {
            return None;
        }

        self.front_len -= 1;

        Some(unsafe { ptr::read(self.gap_ptr()) })
    }

    /// Pop a value from the elements after the gap.
    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        if self.back_len == 0 {
            return None;
        }

        let element = unsafe { ptr::read(self.back_ptr()) };
        self.back_len -= 1;

        Some(element)
    }

    /// Get a reference to the element at `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get(&self, index: usize) -> Option<&T> {
        let p = self.index_to_ptr(index)?;

        // Safety: pointer is valid for returned lifetime.
        Some(unsafe { &*p })
    }

    /// Get a mutable reference to the element at `index`.
    ///
    /// Returns `None` if the index is out of bounds.
    #[inline]
    pub fn get_mut(&mut self, index: usize) -> Option<&mut T> {
        let p = self.index_to_ptr(index)?;

        // Safety: pointer is valid for returned lifetime.
        Some(unsafe { &mut *p })
    }

    /// The elements before the gap.
    #[inline]
    pub fn front(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.front_ptr(), self.front_len) }
    }

    /// The elements before the gap, mutably.
    #[inline]
    pub fn front_mut(&mut self) -> &mut [T] {
        unsafe { slice::from_raw_parts_mut(self.front_ptr(), self.front_len) }
    }

    /// The elements after the gap.
    #[inline]
    pub fn back(&self) -> &[T] {
        unsafe { slice::from_raw_parts(self.back_ptr(), self.back_len) }
    }

    /// The elements after the gap, mutably.
    #[inline]
    pub fn back_mut(&mut self) -> &mut [T] {
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

    /// Ensure that there are at least `additional` spaces available in
    /// the gap, allocating if necessary.
    ///
    /// Will invalidate any pointers into the buffer if it reallocates!
    ///
    /// # Panics
    /// Panics if the length overflows or the required capacity in bytes >
    /// `isize::MAX`.
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
    fn front_ptr(&self) -> *mut T {
        self.inner.as_ptr()
    }

    #[inline]
    fn gap_ptr(&self) -> *mut T {
        // Safety: resulting pointer is within the allocation
        unsafe { self.front_ptr().add(self.front_len) }
    }

    #[inline]
    fn back_ptr(&self) -> *mut T {
        let back_offset = self.capacity() - self.back_len;

        // Safety: resulting pointer is within the allocation
        unsafe { self.front_ptr().add(back_offset) }
    }

    #[inline]
    fn index_to_ptr(&self, index: usize) -> Option<*mut T> {
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

impl<T> From<Vec<T>> for GapVec<T> {
    #[inline]
    fn from(v: Vec<T>) -> Self {
        Self::from_vec(v)
    }
}

impl<T> From<&[T]> for GapVec<T> {
    #[inline]
    fn from(slice: &[T]) -> Self {
        Self::from_slice(slice)
    }
}

impl<T> Index<usize> for GapVec<T> {
    type Output = T;

    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.get(index).expect("index out of bounds")
    }
}

impl<T> IndexMut<usize> for GapVec<T> {
    #[inline]
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index).expect("index out of bounds")
    }
}

#[cfg(test)]
mod tests {
    // Ensure we can't infer the wrong type.
    type GapBuffer = super::GapVec<u16>;

    #[test]
    fn from_vec() {
        let v = vec![0, 1, 2, 3, 4];
        let cap = v.capacity();

        let buf = GapBuffer::from(v);

        assert_eq!(buf.capacity(), cap);
        assert_eq!(buf.len(), 5);
        assert_eq!(buf.front_len, 5);
        assert_eq!(buf.back_len, 0);
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), cap);

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
        buf.push_slice(&[0, 1, 2, 3]);
        buf.push_slice_back(&[4, 5, 6, 7]);

        assert_eq!(buf.capacity(), 8);
        assert_eq!(buf.front(), &[0, 1, 2, 3]);
        assert_eq!(buf.back(), &[4, 5, 6, 7]);

        buf.push_back(100);

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.front(), &[0, 1, 2, 3]);
        assert_eq!(buf.back(), &[100, 4, 5, 6, 7]);
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
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), 16);

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
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), 6);

        assert_eq!(buf.back(), &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

        for i in (0..10).rev() {
            assert_eq!(buf.pop_back(), Some(i));
        }

        assert_eq!(buf.pop_back(), None);
    }

    #[test]
    fn push_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(&[0, 1, 2, 3, 4, 5]);
        buf.push_slice(&[6, 7, 8, 9, 10]);

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.front_len, 11);
        assert_eq!(buf.back_len, 0);
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), 16);

        assert_eq!(buf.front(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        assert_eq!(buf.back(), &[]);
    }

    #[test]
    fn push_slice_back() {
        let mut buf = GapBuffer::new();
        buf.push_slice_back(&[6, 7, 8, 9, 10]);
        buf.push_slice_back(&[0, 1, 2, 3, 4, 5]);

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 11);
        assert_eq!(buf.front_len, 0);
        assert_eq!(buf.back_len, 11);
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), 5);

        assert_eq!(buf.front(), &[]);
        assert_eq!(buf.back(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
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
        assert_eq!(elements_diff(buf.back_ptr(), buf.front_ptr()), 6);

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

        buf.push_slice(&[0, 1, 2, 3, 4]);
        buf.set_gap(1);

        assert_eq!(buf.front(), &[0]);
        assert_eq!(buf.back(), &[1, 2, 3, 4]);

        for (i, mut element) in [0, 1, 2, 3, 4].iter().copied().enumerate() {
            assert_eq!(&buf[i], &element);
            assert_eq!(&mut buf[i], &mut element);
        }

        assert_eq!(buf.get(5), None);
    }

    #[test]
    fn mutable_slice() {
        let mut buf = GapBuffer::new();
        buf.push_slice(&[0, 1, 2, 3, 4]);
        buf.front_mut()[0] = 100;
        assert_eq!(buf.front(), &[100, 1, 2, 3, 4]);
    }

    fn elements_diff<T>(a: *const T, b: *const T) -> usize {
        let byte_diff = a as usize - b as usize;
        assert_eq!(byte_diff % std::mem::size_of::<T>(), 0);
        byte_diff / std::mem::size_of::<T>()
    }
}
