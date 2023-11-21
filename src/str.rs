use std::str::Utf8Error;

use crate::buffer::GapBuffer;

#[derive(Default)]
pub struct GapString {
    inner: GapBuffer,
}

impl GapString {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: GapBuffer::new(),
        }
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.inner.capacity()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push(ch as u8),
            _ => self
                .inner
                .push_slice(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    #[inline]
    pub fn push_back(&mut self, ch: char) {
        match ch.len_utf8() {
            1 => self.inner.push_back(ch as u8),
            _ => self
                .inner
                .push_slice_back(ch.encode_utf8(&mut [0; 4]).as_bytes()),
        }
    }

    #[inline]
    pub fn push_str(&mut self, s: &str) {
        self.inner.push_slice(s.as_bytes());
    }

    #[inline]
    pub fn push_str_back(&mut self, s: &str) {
        self.inner.push_slice_back(s.as_bytes());
    }

    #[inline]
    pub fn pop(&mut self) -> Option<char> {
        let ch = self.front().chars().next_back()?;
        let new_len = self.inner.front_len() - ch.len_utf8();
        self.inner.truncate_front(new_len);
        Some(ch)
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<char> {
        let ch = self.back().chars().next()?;
        let new_len = self.inner.back_len() - ch.len_utf8();
        self.inner.truncate_back(new_len);
        Some(ch)
    }

    #[inline]
    pub fn set_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "index out of bounds");
        assert!(self.is_char_boundary(index), "index not on char boundary");
        self.inner.set_gap(index);
    }

    #[inline]
    pub fn is_char_boundary(&self, index: usize) -> bool {
        match self.inner.get(index) {
            None => index == self.len(),
            Some(&b) => is_utf8_char_boundary(b),
        }
    }

    #[inline]
    pub fn front(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.inner.front()) }
    }

    #[inline]
    pub fn back(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(self.inner.back()) }
    }

    #[inline]
    pub fn front_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.inner.front_mut()) }
    }

    #[inline]
    pub fn back_mut(&mut self) -> &mut str {
        unsafe { std::str::from_utf8_unchecked_mut(self.inner.back_mut()) }
    }

    #[inline]
    pub fn front_and_back_mut(&mut self) -> (&mut str, &mut str) {
        let (front, back) = self.inner.front_and_back_mut();
        unsafe {
            let front = std::str::from_utf8_unchecked_mut(front);
            let back = std::str::from_utf8_unchecked_mut(back);
            (front, back)
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn truncate_front(&mut self, len: usize) {
        assert!(
            self.front().is_char_boundary(len),
            "len not on char boundary"
        );

        self.inner.truncate_front(len);
    }

    #[inline]
    pub fn truncate_back(&mut self, len: usize) {
        let len = self.inner.back_len().min(len);
        let index = self.inner.back_len() - len;

        assert!(
            self.back().is_char_boundary(index),
            "len not on char boundary"
        );

        self.inner.truncate_back(len);
    }

    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }

    #[inline]
    pub fn shrink_to(&mut self, capacity: usize) {
        self.inner.shrink_to(capacity);
    }

    #[inline]
    fn from_buffer_unchecked(inner: GapBuffer) -> Self {
        Self { inner }
    }
}

impl From<String> for GapString {
    #[inline]
    fn from(value: String) -> Self {
        Self::from_buffer_unchecked(value.into_bytes().into())
    }
}

impl From<&str> for GapString {
    #[inline]
    fn from(value: &str) -> Self {
        Self::from_buffer_unchecked(value.as_bytes().into())
    }
}

impl TryFrom<GapBuffer> for GapString {
    type Error = Utf8Error;

    #[inline]
    fn try_from(buffer: GapBuffer) -> Result<Self, Self::Error> {
        let _ = std::str::from_utf8(buffer.front())?;
        let _ = std::str::from_utf8(buffer.back())?;

        Ok(Self::from_buffer_unchecked(buffer))
    }
}

impl TryFrom<Vec<u8>> for GapString {
    type Error = Utf8Error;

    #[inline]
    fn try_from(bytes: Vec<u8>) -> Result<Self, Self::Error> {
        let _ = std::str::from_utf8(&bytes)?;
        Ok(Self::from_buffer_unchecked(bytes.into()))
    }
}

#[inline]
fn is_utf8_char_boundary(byte: u8) -> bool {
    // Taken from std::is_char_boundary
    (byte as i8) >= -0x40
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

    #[test]
    fn set_gap() {
        let mut s = GapString::from("that will be £5 please");

        s.set_gap(15);
        assert_eq!(s.front(), "that will be £");
        assert_eq!(s.back(), "5 please");

        s.set_gap(23);
        assert_eq!(s.front(), "that will be £5 please");
        assert_eq!(s.back(), "");
    }

    #[test]
    #[should_panic = "index not on char boundary"]
    fn set_gap_in_char() {
        let mut s = GapString::from("that will be £5 please");
        s.set_gap(14);
    }

    #[test]
    fn set_gap_empty() {
        let mut s = GapString::new();
        s.set_gap(0);
    }

    #[test]
    #[should_panic = "index out of bounds"]
    fn set_gap_out_of_bounds() {
        let mut s = GapString::new();
        s.set_gap(1);
    }

    #[test]
    fn truncate_front() {
        let mut s = GapString::from("that will be £5 please");

        s.truncate_front(23);
        assert_eq!(s.front(), "that will be £5 please");

        s.truncate_front(15);
        assert_eq!(s.front(), "that will be £");
    }

    #[test]
    #[should_panic = "len not on char boundary"]
    fn truncate_front_invalid() {
        let mut s = GapString::from("that will be £5 please");
        s.truncate_front(14);
    }

    // TODO: test truncate back
}
