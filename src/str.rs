use crate::buffer::GapBuffer;

pub struct GapString {
    inner: GapBuffer,
}

impl GapString {
    pub fn front(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.inner.front()) }
    }

    pub fn back(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.inner.back()) }
    }

    pub fn front_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.inner.front_mut()) }
    }

    pub fn back_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.inner.back_mut()) }
    }

    pub fn front_and_back_mut(&mut self) -> (&mut str, &mut str) {
        let (front, back) = self.inner.front_and_back_mut();
        unsafe {
            let front = std::str::from_utf8_unchecked_mut(front);
            let back = std::str::from_utf8_unchecked_mut(back);
            (front, back)
        }
    }
}
