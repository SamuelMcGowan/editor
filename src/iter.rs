use std::iter::{Chain, FusedIterator};

/// An iterator that skips over the gap.
pub struct SkipGapIter<I> {
    inner: Chain<I, I>,
}

impl<I: Iterator> SkipGapIter<I> {
    pub(crate) fn new(front: I, back: I) -> Self {
        Self {
            inner: front.chain(back),
        }
    }
}

impl<I: Iterator> Iterator for SkipGapIter<I> {
    type Item = I::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }
}

impl<I: DoubleEndedIterator> DoubleEndedIterator for SkipGapIter<I> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<I: ExactSizeIterator> ExactSizeIterator for SkipGapIter<I> {}
impl<I: FusedIterator> FusedIterator for SkipGapIter<I> {}

/// This is an implementation detail - a reimplementation of
/// [`std::str::CharIndices`], required internally to set the character offsets
/// correctly for the back of the buffer.
pub struct CharIndices<'a> {
    front_offset: usize,
    iter: std::str::Chars<'a>,
}

impl<'a> CharIndices<'a> {
    #[inline]
    pub(crate) fn new(s: &'a str, offset: usize) -> Self {
        Self {
            front_offset: offset,
            iter: s.chars(),
        }
    }

    #[inline]
    fn bytes_left(&self) -> usize {
        self.iter.as_str().len()
    }
}

impl<'a> Iterator for CharIndices<'a> {
    type Item = (usize, char);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        let pre_len = self.bytes_left();
        match self.iter.next() {
            None => None,
            Some(ch) => {
                let index = self.front_offset;
                let len = self.bytes_left();
                self.front_offset += pre_len - len;
                Some((index, ch))
            }
        }
    }

    #[inline]
    fn count(self) -> usize {
        self.iter.count()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    #[inline]
    fn last(mut self) -> Option<(usize, char)> {
        self.next_back()
    }
}

impl<'a> DoubleEndedIterator for CharIndices<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<(usize, char)> {
        self.iter.next_back().map(|ch| {
            let index = self.front_offset + self.bytes_left();
            (index, ch)
        })
    }
}

impl FusedIterator for CharIndices<'_> {}
