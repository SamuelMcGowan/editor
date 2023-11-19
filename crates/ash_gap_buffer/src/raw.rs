use std::alloc::{self, Layout};
use std::ptr::NonNull;

pub struct RawVec<T> {
    ptr: NonNull<T>,
    cap: usize,
}

impl<T> RawVec<T> {}

impl<T> RawVec<T> {
    const MIN_RESERVE: usize = 8;
    const MAX_RESERVE: usize = (isize::MAX as usize) / std::mem::size_of::<T>();

    /// # Panics
    /// Panics if `T` is zero-sized.
    #[inline]
    pub const fn new() -> Self {
        check_not_zero_sized::<T>();

        Self {
            ptr: NonNull::dangling(),
            cap: 0,
        }
    }

    /// # Panics
    /// Panics if the required capacity in bytes > `isize::MAX` or if `T` is
    /// zero-sized.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        check_not_zero_sized::<T>();

        let mut buf = Self::new();
        if capacity > 0 {
            buf.alloc_cap(capacity);
        }
        buf
    }

    /// Only works for vecs that use the global allocator.
    ///
    /// # Panics
    /// Panics if `T` is zero-sized.
    #[inline]
    pub fn from_vec(v: Vec<T>) -> Self {
        check_not_zero_sized::<T>();

        let cap = v.capacity();
        let ptr = NonNull::from(v.leak()).cast();
        Self { ptr, cap }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.cap
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut T {
        self.ptr.as_ptr()
    }

    /// Resize so that the new capacity >= the required capacity.
    ///
    /// # Panics
    /// Panics if the required capacity in bytes > `isize::MAX`.
    pub fn resize_to_fit(&mut self, required_cap: usize) {
        if required_cap <= self.cap {
            return;
        }

        // Multiplying cap by 2 can't overflow as cap is at most isize::MAX
        let new_cap = (self.cap * 2)
            .clamp(Self::MIN_RESERVE, Self::MAX_RESERVE)
            .max(required_cap);

        // `new_cap` can't be zero since `required_cap > self.cap``.
        self.alloc_cap(new_cap);
    }

    /// Resize to the given capacity.
    ///
    /// # Panics
    /// Panics if `new_cap == 0` or the required capacity in bytes >
    /// `isize::MAX`.
    pub fn alloc_cap(&mut self, new_cap: usize) {
        assert!(new_cap > 0, "capacity was zero");

        let new_layout = Layout::array::<T>(new_cap).expect("capacity too large");

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<T>(self.cap).unwrap();
            unsafe { alloc::realloc(self.ptr.as_ptr() as *mut u8, old_layout, new_layout.size()) }
        };

        self.ptr = NonNull::new(new_ptr)
            .unwrap_or_else(|| alloc::handle_alloc_error(new_layout))
            .cast();
        self.cap = new_cap;
    }
}

impl<T> Drop for RawVec<T> {
    fn drop(&mut self) {
        if self.cap == 0 {
            return;
        }

        let old_layout = Layout::array::<T>(self.cap).unwrap();
        unsafe { alloc::dealloc(self.ptr.as_ptr() as *mut u8, old_layout) }
    }
}

const fn check_not_zero_sized<T>() {
    assert!(
        std::mem::size_of::<T>() > 0,
        "zero-sized types not supported"
    );
}
