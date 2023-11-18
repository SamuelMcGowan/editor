use std::alloc::{self, Layout};
use std::ptr::NonNull;

const MIN_RESERVE: usize = 8;

pub struct RawBuf {
    ptr: NonNull<u8>,
    cap: usize,
}

impl RawBuf {
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    /// # Panics
    /// Panics if `capacity > isize::MAX`.
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Self::new();
        if capacity > 0 {
            buf.alloc_cap(capacity);
        }
        buf
    }

    /// Only works for vecs that use the global allocator.
    pub fn from_vec(v: Vec<u8>) -> Self {
        let cap = v.capacity();
        let ptr = NonNull::from(v.leak()).cast();
        Self { ptr, cap }
    }

    pub fn capacity(&self) -> usize {
        self.cap
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    /// Resize so that the new capacity >= the required capacity.
    ///
    /// # Panics
    /// Panics if `required_cap > isize::MAX`.
    pub fn resize_to_fit(&mut self, required_cap: usize) {
        if required_cap <= self.cap {
            return;
        }

        // Multiplying cap by 2 can't overflow as cap is at most isize::MAX
        let new_cap = (self.cap * 2)
            .min(isize::MAX as usize)
            .max(MIN_RESERVE)
            .max(required_cap);

        // `new_cap` can't be zero since `required_cap > self.cap``.
        self.alloc_cap(new_cap);
    }

    /// Resize to the given capacity.
    ///
    /// # Panics
    /// Panics if `new_cap == 0` or `new_cap > isize::MAX`.
    pub fn alloc_cap(&mut self, new_cap: usize) {
        assert!(new_cap > 0, "capacity was zero");
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