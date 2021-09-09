use super::*;

#[derive(Debug, Copy, Clone)]
pub struct WordIter<A: WordLookup> {
    a: A,
    idx_of_next_word: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct SetBitIdxIter<A: WordLookup> {
    pub(crate) wi: WordIter<A>,
    pub(crate) cached: usize,
}

impl<A: WordLookup> WordIter<A> {
    pub fn new(a: A) -> Self {
        Self { a, idx_of_next_word: 0 }
    }
}

impl<A: WordLookup> Iterator for WordIter<A> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let next = self.a.get_word(self.idx_of_next_word)?;
        self.idx_of_next_word += 1;
        Some(next)
    }
}

impl<A: WordLookup> SetBitIdxIter<A> {
    pub fn new(a: A) -> Self {
        Self { wi: a.iter_words(), cached: 0 }
    }
}
impl<A: WordLookup> Iterator for SetBitIdxIter<A> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.cached == 0 {
            self.cached = self.wi.next()?;
        }
        // self.cached is NONZERO
        let idx_in_word = self.cached.trailing_zeros();
        self.cached &= !(1 << idx_in_word);
        let wba = WordBitAddr { idx_in_word, idx_of_word: self.wi.idx_of_next_word - 1 };
        Some(wba.to_bit_idx())
    }
}
