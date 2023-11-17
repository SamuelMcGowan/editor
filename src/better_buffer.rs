use std::alloc::{self, Layout};
use std::cmp::Ordering;
use std::ptr::{self, NonNull};

const MIN_RESERVE: usize = 8;

struct RawBuf {
    ptr: NonNull<u8>,
    cap: usize,
}

impl RawBuf {
    const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        let mut buf = Self::new();
        buf.alloc_cap(capacity);
        buf
    }

    /// Resize so that the new capacity >= the required capacity.
    fn resize_to_fit(&mut self, required_cap: usize) {
        if required_cap <= self.cap {
            return;
        }

        // Multiplying cap by 2 can't overflow as cap is at most isize::MAX
        let new_cap = (self.cap * 2).max(required_cap).max(MIN_RESERVE);

        self.alloc_cap(new_cap);
    }

    /// Resize to the given capacity.
    fn alloc_cap(&mut self, new_cap: usize) {
        assert!(new_cap > 0);
        assert!(
            new_cap <= isize::MAX as usize,
            "capacity too large (greater than isize::MAX)"
        );

        let new_layout = Layout::array::<u8>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe { alloc::realloc(self.ptr.as_ptr(), old_layout, new_layout.size()) }
        };

        self.ptr = match NonNull::new(new_ptr) {
            Some(ptr) => ptr,
            None => alloc::handle_alloc_error(new_layout),
        };
        self.cap = new_cap;
    }
}

impl Drop for RawBuf {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        let old_layout = Layout::array::<u8>(self.cap).unwrap();
        unsafe { alloc::dealloc(self.ptr.as_ptr(), old_layout) }
    }
}

pub struct GapBuffer {
    inner: RawBuf,
    len_start: usize,
    len_end: usize,
}

impl GapBuffer {
    pub const fn new() -> Self {
        Self {
            inner: RawBuf::new(),
            len_start: 0,
            len_end: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: RawBuf::with_capacity(capacity),
            len_start: 0,
            len_end: 0,
        }
    }

    pub fn capacity(&self) -> usize {
        self.inner.cap
    }

    pub fn len_start(&self) -> usize {
        self.len_start
    }

    pub fn len_end(&self) -> usize {
        self.len_end
    }

    pub fn len(&self) -> usize {
        self.len_start + self.len_end
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn push(&mut self, byte: u8) {
        self.make_space(1);

        unsafe { ptr::write(self.gap_ptr(), byte) };

        self.len_start += 1; // FIXME: handle overflow
    }

    pub fn pop(&mut self) -> Option<u8> {
        if self.len_start == 0 {
            return None;
        }

        self.len_start -= 1;

        Some(unsafe { ptr::read(self.gap_ptr()) })
    }

    pub fn slice_start(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.start_ptr(), self.len_start) }
    }

    pub fn slice_end(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.end_ptr(), self.len_end) }
    }

    pub fn set_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "index out of bounds");

        if self.capacity() == 0 {
            return;
        }

        match index.cmp(&self.len_start) {
            Ordering::Less => {
                let src_ptr = unsafe { self.start_ptr().add(index) };
                let len = self.len_start - index;

                self.len_start = index;
                self.len_end += len;

                unsafe { ptr::copy(src_ptr, self.end_ptr(), len) };
            }

            Ordering::Equal => {}

            Ordering::Greater => {
                let src_ptr = self.end_ptr();
                let dest_ptr = self.gap_ptr();
                let len = index - self.len_start;

                self.len_start = index;
                self.len_end -= len;

                unsafe { ptr::copy(src_ptr, dest_ptr, len) };
            }
        }
    }

    /// Ensure that there are at least `additional` bytes in the gap.
    fn make_space(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }

        let required_len = self
            .len()
            .checked_add(additional)
            .expect("length overflowed");

        let prev_end_len = self.len_end;

        self.inner.resize_to_fit(required_len);

        // Use offset to get end pointer because the buffer could have moved.
        let prev_end_ptr = unsafe { self.start_ptr().add(prev_end_len) };
        let end_ptr = self.end_ptr();

        if !ptr::eq(end_ptr, prev_end_ptr) {
            unsafe { ptr::copy(prev_end_ptr, end_ptr, self.len_end()) };
        }
    }

    fn start_ptr(&self) -> *mut u8 {
        self.inner.ptr.as_ptr()
    }

    fn gap_ptr(&self) -> *mut u8 {
        // Safety: ptr + len_start is within the allocation
        unsafe { self.start_ptr().add(self.len_start) }
    }

    fn end_ptr(&self) -> *mut u8 {
        let end_offset = self.capacity() - self.len_end();

        // Safety: ptr + end_offset is within the allocation
        unsafe { self.start_ptr().add(end_offset) }
    }
}

#[cfg(test)]
mod tests {
    use super::GapBuffer;

    #[test]
    fn push_pop() {
        let mut buf = GapBuffer::new();

        for i in 0..10 {
            buf.push(i);
        }

        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.len_start(), 10);
        assert_eq!(buf.len_end(), 0);
        assert_eq!(ptr_diff(buf.end_ptr(), buf.start_ptr()), 16);

        assert_eq!(buf.slice_start(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);

        for i in (0..10).rev() {
            assert_eq!(buf.pop(), Some(i));
        }

        assert_eq!(buf.pop(), None);
    }

    #[test]
    fn set_gap() {
        let mut buf = GapBuffer::new();
        for i in 0..10 {
            buf.push(i);
        }

        assert_eq!(buf.slice_start(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
        assert_eq!(buf.slice_end(), &[]);

        buf.set_gap(0);
        assert_eq!(buf.capacity(), 16);
        assert_eq!(buf.len(), 10);
        assert_eq!(buf.len_start(), 0);
        assert_eq!(buf.len_end(), 10);
        assert_eq!(ptr_diff(buf.end_ptr(), buf.start_ptr()), 6);

        assert_eq!(buf.slice_start(), &[]);
        assert_eq!(buf.slice_end(), &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
    }

    #[test]
    #[should_panic]
    fn set_gap_out_of_bounds() {
        let mut buf = GapBuffer::new();
        buf.set_gap(1);
    }

    fn ptr_diff(a: *const u8, b: *const u8) -> usize {
        a as usize - b as usize
    }
}
