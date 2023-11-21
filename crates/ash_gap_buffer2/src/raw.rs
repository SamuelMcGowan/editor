use std::alloc::{self, Layout};
use std::ptr::NonNull;

pub struct RawBuf {
    ptr: NonNull<u8>,
    cap: usize,
}

impl RawBuf {
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
    /// Panics if `new_cap > isize::MAX`.
    pub fn set_capacity(&mut self, new_cap: usize) {
        assert!(
            new_cap <= isize::MAX as usize,
            "capacity overflows `isize::MAX`"
        );

        if self.cap == new_cap {
            return;
        }

        if new_cap == 0 {
            // Previous capacity wasn't zero, so there is an allocation.
            unsafe { alloc::dealloc(self.as_ptr_mut(), self.layout()) };
            // Pointer is already dangling so no need to set.
        } else {
            let new_layout = Layout::array::<u8>(new_cap).unwrap();

            let new_ptr = if self.cap == 0 {
                unsafe { alloc::alloc(new_layout) }
            } else {
                unsafe { alloc::realloc(self.as_ptr_mut(), self.layout(), new_layout.size()) }
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
    pub fn as_ptr(&self) -> *const u8 {
        self.ptr.as_ptr()
    }

    #[inline]
    pub fn as_ptr_mut(&mut self) -> *mut u8 {
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

        unsafe { alloc::dealloc(self.as_ptr_mut(), self.layout()) };
    }
}

#[cfg(test)]
mod tests {
    use super::RawBuf;

    #[test]
    fn reallocate() {
        let mut buf = RawBuf::new();

        // do nothing
        buf.set_capacity(0);
        assert_eq!(buf.capacity(), 0);

        // allocate
        buf.set_capacity(5);
        assert_eq!(buf.capacity(), 5);

        // do nothing
        buf.set_capacity(5);
        assert_eq!(buf.capacity(), 5);

        // reallocate
        buf.set_capacity(10);
        assert_eq!(buf.capacity(), 10);

        // deallocate
        buf.set_capacity(0);
        assert_eq!(buf.capacity(), 0);
    }

    #[test]
    fn drop_deallocate() {
        RawBuf::with_capacity(10);
    }

    #[test]
    #[should_panic = "capacity overflows `isize::MAX`"]
    fn cap_too_large() {
        RawBuf::with_capacity(isize::MAX as usize + 1);
    }
}
