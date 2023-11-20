use std::alloc::{self, Layout};
use std::ptr::NonNull;

pub struct RawBuf {
    ptr: NonNull<u8>,
    cap: usize,
}

impl RawBuf {
    const MIN_RESERVE: usize = 8;
    const MAX_RESERVE: usize = isize::MAX as usize;

    #[inline]
    pub const fn new() -> Self {
        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    /// # Panics
    /// Panics if `capacity > isize::MAX`.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        let mut buf = Self::new();
        buf.set_capacity(capacity);
        buf
    }

    /// # Panics
    /// Panics if `required_cap > isize::MAX`.
    pub fn resize_to_fit(&mut self, required_cap: usize) {
        if required_cap <= self.cap {
            return;
        }

        // Multiplying cap by 2 can't overflow as cap is at most isize::MAX
        let new_cap = (self.cap * 2)
            .clamp(Self::MIN_RESERVE, Self::MAX_RESERVE)
            .max(required_cap);

        // We allow `set_capacity` to check that the new capacity <= `isize::MAX`.
        self.set_capacity(new_cap);
    }

    /// # Panics
    /// Panics if `new_cap > isize::MAX`.
    pub fn set_capacity(&mut self, new_cap: usize) {
        if self.cap == new_cap {
            return;
        }

        if new_cap == 0 {
            // Previous capacity wasn't zero, so there is an allocation.
            unsafe { alloc::dealloc(self.as_ptr(), self.layout()) };
        } else {
            let new_layout = Layout::array::<u8>(new_cap).unwrap();

            let new_ptr = if self.cap == 0 {
                unsafe { alloc::alloc(new_layout) }
            } else {
                unsafe { alloc::realloc(self.as_ptr(), self.layout(), new_layout.size()) }
            };

            self.ptr =
                NonNull::new(new_ptr).unwrap_or_else(|| alloc::handle_alloc_error(new_layout));
        }

        self.cap = new_cap;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr.as_ptr()
    }

    #[inline]
    fn layout(&self) -> Layout {
        Layout::array::<u8>(self.cap).unwrap()
    }
}

impl From<Vec<u8>> for RawBuf {
    fn from(v: Vec<u8>) -> Self {
        // `Vec` also uses a dangling pointer for an unallocated vector.
        let cap = v.capacity();
        let ptr = NonNull::from(v.leak()).cast();
        Self { ptr, cap }
    }
}

impl Drop for RawBuf {
    #[inline]
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        unsafe { alloc::dealloc(self.as_ptr(), self.layout()) };
    }
}
