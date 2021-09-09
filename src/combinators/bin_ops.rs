use crate::{BinChunkOp, Chunk};

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Nand;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Or;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Xor;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct And;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Diff;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Fst;
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Snd;

//////////

#[inline]
fn z(chunk: Option<Chunk>) -> Chunk {
    chunk.unwrap_or(0)
}

impl BinChunkOp for Nand {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        Some(!(z(a) & z(b)))
    }
}
impl BinChunkOp for Or {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if a.is_none() && b.is_none() {
            None
        } else {
            Some(z(a) | z(b))
        }
    }
}
impl BinChunkOp for Xor {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        Some(z(a) ^ z(b))
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
}
impl BinChunkOp for Diff {
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        if let Some(a) = a {
            Some(a & !(z(b)))
        } else {
            None
        }
    }
}
impl BinChunkOp for Fst {
    fn combine_chunks(self, a: Option<Chunk>, _: Option<Chunk>) -> Option<Chunk> {
        a
    }
}
impl BinChunkOp for Snd {
    fn combine_chunks(self, _: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk> {
        b
    }
}
