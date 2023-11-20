use crate::buffer::GapBuffer;

#[derive(Default)]
pub struct GapString {
    inner: GapBuffer,
}

impl GapString {
    pub const fn new() -> Self {
        Self {
            inner: GapBuffer::new(),
        }
    }

    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push(ch as u8),
            _ => self
                .inner
                .push_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    pub fn push_back(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push_back(ch as u8),
            _ => self
                .inner
                .push_slice_back(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    pub fn push_str(&mut self, s: &str) {
        self.inner.push_slice(s.as_bytes());
    }

    pub fn push_str_back(&mut self, s: &str) {
        self.inner.push_slice_back(s.as_bytes());
    }

    pub fn pop(&mut self) -> Option<char> {
        let ch = self.front().chars().next_back()?;
        let new_len = self.inner.front_len() - ch.len_utf8();
        self.inner.truncate_front(new_len);
        Some(ch)
    }

    pub fn pop_back(&mut self) -> Option<char> {
        let ch = self.back().chars().next()?;
        let new_len = self.inner.back_len() - ch.len_utf8();
        self.inner.truncate_back(new_len);
        Some(ch)
    }

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

    fn from_buffer_unchecked(inner: GapBuffer) -> Self {
        Self { inner }
    }
}

impl From<String> for GapString {
    fn from(value: String) -> Self {
        Self::from_buffer_unchecked(value.into_bytes().into())
    }
}

impl From<&str> for GapString {
    fn from(value: &str) -> Self {
        Self::from_buffer_unchecked(value.as_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use super::GapString;

    #[test]
    fn push_pop() {
        let mut s = GapString::new();

        s.push('a');
        s.push('£');
        s.push('b');

        assert_eq!(s.pop(), Some('b'));
        assert_eq!(s.pop(), Some('£'));
        assert_eq!(s.pop(), Some('a'));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn push_pop_back() {
        let mut s = GapString::new();

        s.push_back('a');
        s.push_back('£');
        s.push_back('b');

        assert_eq!(s.pop_back(), Some('b'));
        assert_eq!(s.pop_back(), Some('£'));
        assert_eq!(s.pop_back(), Some('a'));
        assert_eq!(s.pop_back(), None);
    }
}
