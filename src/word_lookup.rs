use super::*;

pub trait WordLookup: Copy {
    fn get_word(self, idx_of_word: usize) -> Option<usize>;
    fn into_set_bit_iter(self) -> SetBitIdxIter<Self> {
        SetBitIdxIter::new(self)
    }
    fn into_word_iter(self) -> WordIter<Self> {
        WordIter::new(self)
    }
    fn count_bits(&self) -> usize {
        self.into_word_iter().map(|x| x.count_ones() as usize).sum()
    }
    fn buffer_words_into(self, buf: &mut Vec<usize>) {
        for word in self.into_word_iter() {
            buf.push(word)
        }
    }
    fn get_bit(self, bit_idx: usize) -> bool {
        let wba = WordBitAddr::from_bit_idx(bit_idx);
        match self.get_word(wba.idx_of_word) {
            None => false,
            Some(word) => word & (1 << wba.idx_in_word) > 0,
        }
    }
    fn not_words(self) -> NotWords<Self> {
        NotWords { a: self }
    }
    fn combine_words<B: WordLookup>(self, b: B, op: BinWordOperator) -> CombinedWords<Self, B> {
        CombinedWords { a: self, b, op }
    }
}
impl WordLookup for &Words {
    fn get_word(self, idx_of_word: usize) -> Option<usize> {
        self.words.get(idx_of_word).copied()
    }
}
