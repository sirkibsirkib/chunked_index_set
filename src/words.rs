use super::{WordBitAddr, WordLookup, Words};

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
