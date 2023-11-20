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
    pub fn grow(&mut self, required_cap: usize) {
        if let Some(new_cap) = self.grow_cap(required_cap) {
            // We allow `set_capacity` to check that the new capacity <= `isize::MAX`.
            self.set_capacity(new_cap);
        }
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

    #[inline]
    #[must_use]
    fn grow_cap(&self, required_cap: usize) -> Option<usize> {
        // Vec::push(&mut self, value)
        if required_cap <= self.cap {
            None
        } else {
            // Multiplying current cap by 2 can't overflow as it is at most isize::MAX
            let new_cap = (self.cap * 2)
                .clamp(Self::MIN_RESERVE, Self::MAX_RESERVE)
                .max(required_cap);

            Some(new_cap)
        }
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

    #[test]
    fn grow() {
        let mut buf = RawBuf::new();

        assert_eq!(buf.grow_cap(0), None);
        assert_eq!(buf.grow_cap(1), Some(RawBuf::MIN_RESERVE));
        assert_eq!(
            buf.grow_cap(RawBuf::MIN_RESERVE + 1),
            Some(RawBuf::MIN_RESERVE + 1)
        );

        buf.set_capacity(5);
        assert_eq!(buf.grow_cap(6), Some(10));
    }
}
