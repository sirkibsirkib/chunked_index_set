use super::WordLookup;

#[derive(Debug, Copy, Clone)]
pub struct NotWords<A: WordLookup> {
    pub a: A,
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
pub struct CombinedWords<A: WordLookup, B: WordLookup> {
    pub a: A,
    pub b: B,
    pub op: BinWordOperator,
}

///////////////////////////
impl BinWordOperator {
    #[inline]
    pub fn word_op_fn(self) -> fn(usize, usize) -> usize {
        use BinWordOperator::*;
        match self {
            Nand => |a, b| !(a & b),
            And => core::ops::BitAnd::bitand,
            Or => core::ops::BitOr::bitor,
            Xor => core::ops::BitXor::bitxor,
            Difference => |a, b| a & !b,
        }
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
        Some((self.op.word_op_fn())(wa.unwrap_or(0), wb.unwrap_or(0)))
    }
}
