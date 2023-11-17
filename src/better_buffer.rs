use std::alloc::{self, Layout};
use std::ptr::NonNull;

const MIN_RESERVE: usize = 8;

pub struct RawBuf {
    bytes: NonNull<u8>,
    cap: usize,
}

impl RawBuf {
    pub const fn new() -> Self {
        Self {
            bytes: NonNull::dangling(),
            cap: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Self::new();
        buf.reserve(capacity);
        buf
    }

    pub fn reserve(&mut self, additional: usize) {
        if additional == 0 {
            return;
        }

        let new_cap = grow_cap(self.cap, additional);
        let new_layout = Layout::array::<u8>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe { alloc::realloc(self.bytes.as_ptr(), old_layout, new_layout.size()) }
        };

        self.bytes = match NonNull::new(new_ptr) {
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
        unsafe { alloc::dealloc(self.bytes.as_ptr(), old_layout) }
    }
}

fn grow_cap(cap: usize, additional: usize) -> usize {
    debug_assert!(cap <= isize::MAX as usize);
    debug_assert!(additional > 0);

    let required_cap = cap.checked_add(additional).expect("capacity overflow");

    // Multiplying cap by 2 can't overflow as cap is at most isize::MAX
    (cap * 2).max(required_cap).max(MIN_RESERVE)
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
}
