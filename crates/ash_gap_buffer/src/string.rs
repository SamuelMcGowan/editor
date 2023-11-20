use crate::GapVec;

pub struct GapString {
    inner: GapVec<u8>,
}

impl GapString {
    #[inline]
    pub const fn new() -> Self {
        Self {
            inner: GapVec::new(),
        }
    }

    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: GapVec::with_capacity(capacity),
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
        self.len() == 0
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

        let new_len = self.front().len() - ch.len_utf8();
        unsafe { self.inner.set_len_front(new_len) };

        Some(ch)
    }

    #[inline]
    pub fn pop_back(&mut self) -> Option<char> {
        let ch = self.back().chars().next()?;

        let new_len = self.back().len() - ch.len_utf8();
        unsafe { self.inner.set_len_back(new_len) };

        Some(ch)
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
    pub fn is_char_boundary(&self, index: usize) -> bool {
        if index <= self.front().len() {
            self.front().is_char_boundary(index)
        } else {
            self.back().is_char_boundary(index - self.front().len())
        }
    }

    #[inline]
    pub fn set_gap(&mut self, index: usize) {
        assert!(index <= self.len(), "index out of bounds");
        assert!(
            self.is_char_boundary(index),
            "index is not on a char boundary"
        );

        self.inner.set_gap(index);
    }

    #[inline]
    fn from_bytes_unchecked(bytes: GapVec<u8>) -> Self {
        Self { inner: bytes }
    }
}

impl From<&str> for GapString {
    fn from(value: &str) -> Self {
        Self::from_bytes_unchecked(value.as_bytes().into())
    }
}

impl From<String> for GapString {
    fn from(value: String) -> Self {
        Self::from_bytes_unchecked(value.into_bytes().into())
    }
}

#[cfg(test)]
mod tests {
    use super::GapString;

    #[test]
    fn push_pop() {
        let mut s = GapString::new();

        s.push('£');
        s.push_str("ab");
        s.push('c');

        assert_eq!(s.len(), 5);

        assert_eq!(s.pop(), Some('c'));
        assert_eq!(s.pop(), Some('b'));
        assert_eq!(s.pop(), Some('a'));
        assert_eq!(s.pop(), Some('£'));

        assert_eq!(s.len(), 0);
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn push_pop_back() {
        let mut s = GapString::new();

        s.push_back('£');
        s.push_str_back("ba");
        s.push_back('c');

        assert_eq!(s.capacity(), 8);
        assert_eq!(s.len(), 5);

        assert_eq!(s.pop_back(), Some('c'));
        assert_eq!(s.pop_back(), Some('b'));
        assert_eq!(s.pop_back(), Some('a'));
        assert_eq!(s.pop_back(), Some('£'));

        assert_eq!(s.len(), 0);
        assert_eq!(s.pop_back(), None);
    }

    #[test]
    fn push_str() {
        let mut s = GapString::new();
        s.push_str("hello");
        s.push_str(" £world");

        assert_eq!(s.capacity(), 16);
        assert_eq!(s.len(), 13);

        assert_eq!(s.front(), "hello £world");
        assert_eq!(s.back(), "");
    }

    #[test]
    fn push_str_back() {
        let mut s = GapString::new();
        s.push_str_back(" £world");
        s.push_str_back("hello");

        assert_eq!(s.capacity(), 16);
        assert_eq!(s.len(), 13);

        assert_eq!(s.front(), "");
        assert_eq!(s.back(), "hello £world");
    }

    #[test]
    fn set_gap() {
        let mut s = GapString::new();
        s.push_str("hello world");
        s.set_gap(0);

        assert_eq!(s.front(), "");
        assert_eq!(s.back(), "hello world");

        s.set_gap(5);

        assert_eq!(s.front(), "hello");
        assert_eq!(s.back(), " world");
    }

    #[test]
    #[should_panic = "index is not on a char boundary"]
    fn set_gap_invalid_front() {
        let mut s = GapString::new();
        s.push_str("£5");
        s.set_gap(1);
    }

    #[test]
    #[should_panic = "index is not on a char boundary"]
    fn set_gap_invalid_back() {
        let mut s = GapString::new();
        s.push_str_back("£5");
        s.set_gap(1);
    }
}
