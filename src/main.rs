#[derive(Default, Debug, Clone, Eq, PartialEq)]
struct Words {
    // invariant: last word is non-zero
    words: Vec<usize>,
}

#[derive(Debug, Copy, Clone)]
struct WordBitAddr {
    idx_of_word: usize,
    idx_in_word: u32, // invariant: in 0..usize::BITs
}

#[derive(Debug, Copy, Clone)]
struct NotWords<A: WordLookup> {
    a: A,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum BinWordOperator {
    Nand,       // either a or b
    Or,         // either a or b
    Xor,        // either a or b
    And,        // both
    Difference, // first not latter
}

#[derive(Debug, Copy, Clone)]
struct CombinedWords<A: WordLookup, B: WordLookup> {
    a: A,
    b: B,
    op: BinWordOperator,
}

#[derive(Debug, Copy, Clone)]
struct WordIter<A: WordLookup> {
    a: A,
    idx_of_next_word: usize,
}

#[derive(Debug, Copy, Clone)]
struct SetBitIdxIter<A: WordLookup> {
    wi: WordIter<A>,
    cached: usize,
}
///////////////////////////////////////////////////////////////////////
trait WordLookup: Copy {
    fn get_word(self, idx_of_word: usize) -> Option<usize>;
    fn into_set_bit_iter(self) -> SetBitIdxIter<Self> {
        SetBitIdxIter {
            wi: self.into_word_iter(),
            cached: 0,
        }
    }
    fn into_word_iter(self) -> WordIter<Self> {
        WordIter {
            a: self,
            idx_of_next_word: 0,
        }
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
///////////////////////////////////////////////////////////////////////

impl<T: WordLookup> WordLookup for &T {
    fn get_word(self, idx_of_word: usize) -> Option<usize> {
        <T as WordLookup>::get_word(*self, idx_of_word)
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

impl WordBitAddr {
    fn from_bit_idx(bit_idx: usize) -> Self {
        Self {
            idx_of_word: bit_idx / usize::BITS as usize,
            idx_in_word: (bit_idx % usize::BITS as usize) as u32,
        }
    }
    fn to_bit_idx(self) -> usize {
        self.idx_of_word * usize::BITS as usize + self.idx_in_word as usize
    }
}
impl<A: WordLookup> WordLookup for NotWords<A> {
    fn get_word(self, idx_of_word: usize) -> Option<usize> {
        Some(!self.a.get_word(idx_of_word).unwrap_or(0))
    }
}

impl<A: WordLookup, B: WordLookup> WordLookup for &CombinedWords<A, B> {
    fn get_word(self, idx_of_word: usize) -> Option<usize> {
        let wa = self.a.get_word(idx_of_word);
        let wb = self.b.get_word(idx_of_word);
        use BinWordOperator::*;
        if let (None, None, _) | (_, None, And) | (None, _, And | Difference) = (wa, wb, self.op) {
            return None;
        }
        let wa = wa.unwrap_or(0);
        let wb = wb.unwrap_or(0);
        Some(match self.op {
            Nand => !(wa & wb),
            And => wa & wb,
            Or => wa & wb,
            Xor => wa ^ wb,
            Difference => wa & !wb,
        })
    }
}
impl WordLookup for &Words {
    fn get_word(self, idx_of_word: usize) -> Option<usize> {
        self.words.get(idx_of_word).copied()
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
        let wba = WordBitAddr {
            idx_in_word,
            idx_of_word: self.wi.idx_of_next_word - 1,
        };
        Some(wba.to_bit_idx())
    }
}

impl Words {
    fn pop_zero_tail(&mut self) {
        loop {
            match self.words.pop() {
                Some(n) if n > 0 => {
                    self.words.push(n);
                    return;
                }
                _ => {}
            }
        }
    }
    pub fn add_all<A: WordLookup>(&mut self, a: A) {
        let mut it = a.into_word_iter();
        for dest in self.words.iter_mut() {
            match it.next() {
                Some(w) => *dest |= w,
                None => return,
            }
        }
        while let Some(w) = it.next() {
            self.words.push(w);
        }
        self.pop_zero_tail();
    }
    pub fn remove_all<A: WordLookup>(&mut self, a: A) {
        for i in 0.. {
            match (self.words.get_mut(i), a.get_word(i)) {
                (Some(dest), Some(src)) => *dest &= !src,
                _ => return,
            }
        }
    }
    pub fn clear(&mut self) {
        self.words.clear();
    }
    pub fn insert_bit(&mut self, k: usize) -> bool {
        let wba = WordBitAddr::from_bit_idx(k);
        while self.words.len() <= wba.idx_of_word {
            self.words.push(0);
        }
        let word = unsafe { self.words.get_unchecked_mut(wba.idx_of_word) };
        let mask = 1 << wba.idx_in_word as usize;
        let was = *word & mask > 0;
        *word |= mask;
        !was
    }
    pub fn flip_bit(&mut self, k: usize) -> bool {
        let wba = WordBitAddr::from_bit_idx(k);
        if let Some(word) = self.words.get_mut(wba.idx_of_word) {
            let mask = 1 << wba.idx_in_word as usize;
            let was = *word & mask > 0;
            *word |= mask;
            was
        } else {
            self.insert_bit(k)
        }
    }
    pub fn remove_bit(&mut self, k: usize) -> bool {
        let wba = WordBitAddr::from_bit_idx(k);
        if let Some(word) = self.words.get_mut(wba.idx_of_word) {
            let mask = 1 << wba.idx_in_word as usize;
            let was = *word & mask > 0;
            if was {
                *word &= !mask;
                self.pop_zero_tail();
                true
            } else {
                // can't remove. bit is already zero
                false
            }
        } else {
            // can't remove. bit out of bounds
            false
        }
    }
    pub fn from_set_bits<T: IntoIterator<Item = usize>>(iter: T) -> Self {
        let mut me = Self::default();
        for bit_idx in iter {
            me.insert_bit(bit_idx);
        }
        me
    }
}

fn main() {
    let b = Words::from_set_bits([1, 2, 3]);
    let nb = b.not_words();
    for i in nb.into_set_bit_iter().take(20) {
        println!("{:?}", i);
    }
}
