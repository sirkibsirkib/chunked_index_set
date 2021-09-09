mod word_lookup;
pub use word_lookup::WordLookup;

mod words;
pub use words::{WordDrain, Words};

pub mod combinators;
use combinators::{BinWordOperator, CombinedWords, NotWords};

pub mod iterators;
use iterators::{SetBitIdxIter, WordIter};

#[cfg(test)]
mod tests;

/////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
struct WordBitAddr {
    idx_of_word: usize,
    idx_in_word: u32, // invariant: in 0..usize::BITs
}
///////////////////////////////////////////////////////////////////////

impl WordBitAddr {
    fn from_bit_idx(bit_idx: usize) -> Self {
        Self {
            idx_of_word: bit_idx / usize::BITS as usize,
            idx_in_word: (bit_idx % usize::BITS as usize) as u32,
        }
    }
    const fn word_mask(&self) -> usize {
        1 << self.idx_in_word
    }
    fn to_bit_idx(self) -> usize {
        self.idx_of_word * usize::BITS as usize + self.idx_in_word as usize
    }
}
