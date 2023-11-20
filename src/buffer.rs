use std::ptr;

use crate::raw::RawBuf;

pub struct GapBuffer {
    inner: RawBuf,

    front_len: usize,
    back_len: usize,
}

impl GapBuffer {
    #[inline]
    pub fn len(&self) -> usize {
        self.front_len + self.back_len
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Panics if new_cap > isize::MAX.
    fn grow_for_push(&mut self, additional: usize) {
        let new_cap = calc_new_capacity(self.len(), additional);

        if new_cap > self.inner.capacity() {
            let prev_back_offset = self.inner.capacity() - self.back_len;

            self.inner.set_capacity(new_cap);

            // Use offset to get previous back pointer because the buffer could have moved.
            let back_ptr_prev = unsafe { self.front_ptr().add(prev_back_offset) };
            let back_ptr = self.back_ptr();

            unsafe { ptr::copy(back_ptr_prev, back_ptr, self.back_len) };
        }
    }

    fn front_ptr(&self) -> *mut u8 {
        self.inner.as_ptr()
    }

    fn back_ptr(&self) -> *mut u8 {
        let back_offset = self.inner.capacity() - self.back_len;
        unsafe { self.front_ptr().add(back_offset) }
    }
}

#[inline]
fn calc_new_capacity(len: usize, additional: usize) -> usize {
    if additional == 0 {
        return len;
    }

    let min_gap_size = (len / 16).max(64);
    let new_gap_size = additional.max(min_gap_size);

    len.checked_add(new_gap_size)
        .expect("required capacity too large")
}

#[cfg(test)]
mod tests {
    use crate::buffer::calc_new_capacity;

    #[test]
    fn calc_capacity() {
        assert_eq!(calc_new_capacity(0, 0), 0);
        assert_eq!(calc_new_capacity(0, 1), 64);
        assert_eq!(calc_new_capacity(64, 1), 128);
    }
}
