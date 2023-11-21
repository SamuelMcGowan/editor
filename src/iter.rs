use std::{iter, slice};

pub struct Bytes<'a> {
    pub(crate) inner: iter::Copied<iter::Chain<slice::Iter<'a, u8>, slice::Iter<'a, u8>>>,
}

impl Iterator for Bytes<'_> {
    type Item = u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for Bytes<'_> {}

impl DoubleEndedIterator for Bytes<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

pub struct BytesMut<'a> {
    pub(crate) inner: iter::Chain<slice::IterMut<'a, u8>, slice::IterMut<'a, u8>>,
}

impl<'a> Iterator for BytesMut<'a> {
    type Item = &'a mut u8;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl ExactSizeIterator for BytesMut<'_> {}

impl DoubleEndedIterator for BytesMut<'_> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}
