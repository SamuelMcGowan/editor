use std::ptr::NonNull;

struct Buffer {
    buffer_start: *mut u8,
    buffer_end: *mut u8,

    gap_start: *mut u8,
    gap_end: *mut u8,
}

impl Buffer {
    pub fn new() -> Self {
        Buffer {
            buffer_start: std::ptr::null_mut(),
            buffer_end: std::ptr::null_mut(),
            gap_start: std::ptr::null_mut(),
            gap_end: std::ptr::null_mut(),
        }
    }

    fn len(&self) -> usize {
        self.capacity() - self.gap_len()
    }

    fn gap_len(&self) -> usize {
        (self.gap_end as usize) - (self.gap_start as usize)
    }

    fn capacity(&self) -> usize {
        (self.buffer_end as usize) - (self.buffer_start as usize)
    }
}
