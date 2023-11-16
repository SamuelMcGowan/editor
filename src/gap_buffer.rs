use std::alloc::{self, Layout};

const GROW_BY: usize = 1024;

struct Buffer {
    left: *mut u8,
    left_len: usize,

    right: *mut u8,
    right_len: usize,

    cap: usize,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            left: std::ptr::null_mut(),
            left_len: 0,

            right: std::ptr::null_mut(),
            right_len: 0,

            cap: 0,
        }
    }

    fn grow_gap(&mut self) {
        let new_cap = self.cap + GROW_BY;
        let new_layout = Layout::array::<u8>(new_cap).unwrap();

        let new_ptr = if self.cap == 0 {
            unsafe { alloc::alloc(new_layout) }
        } else {
            let old_layout = Layout::array::<u8>(self.cap).unwrap();
            unsafe { alloc::realloc(self.left, old_layout, new_layout.size()) }
        };

        if new_ptr.is_null() {
            alloc::handle_alloc_error(new_layout);
        }

        let right_old = self.right;

        self.left = new_ptr;
        self.right = unsafe { self.left.add(self.cap - self.right_len) };

        self.cap = new_cap;

        unsafe { std::ptr::copy(right_old, self.right, self.right_len) };
    }
}
