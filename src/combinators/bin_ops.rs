use crate::{BinChunkOp, Chunk};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Or;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Xor;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct And;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Without;

//////////

#[inline]
fn z(chunk: Option<Chunk>) -> Chunk {
    chunk.unwrap_or(0)
}
impl BinChunkOp for Or {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if a.is_none() && b.is_none() {
            None
        } else {
            Some(z(a) | z(b))
        }
    }
    fn combine_zero_chunks_from_conservative(self, ncfa: usize, ncfb: usize) -> usize {
        ncfa.max(ncfb)
    }
}
impl BinChunkOp for Xor {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if a.is_none() && b.is_none() {
            None
        } else {
            Some(z(a) ^ z(b))
        }
    }
    fn combine_zero_chunks_from_conservative(self, ncfa: usize, ncfb: usize) -> usize {
        ncfa.max(ncfb)
    }
}
impl BinChunkOp for And {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if let [Some(a), Some(b)] = [a, b] {
            Some(a & b)
        } else {
            None
        }
    }
    fn combine_zero_chunks_from_conservative(self, ncfa: usize, ncfb: usize) -> usize {
        ncfa.min(ncfb)
    }
}
impl BinChunkOp for Without {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if let Some(a) = a {
            Some(a & !(z(b)))
        } else {
            None
        }
    }
    fn combine_zero_chunks_from_conservative(self, ncfa: usize, _ncfb: usize) -> usize {
        ncfa
    }
}
