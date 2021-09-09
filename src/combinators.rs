use super::{Chunk, ChunkRead};

#[derive(Debug, Copy, Clone)]
pub struct NotChunks<A: ChunkRead> {
    pub a: A,
}

pub trait BinChunkOp: Copy {
    /// NONE means zero chunk AND all subsequent chunks are zero
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk>;
}

pub mod bin_ops {
    use crate::{BinChunkOp, Chunk};

    #[inline]
    fn z(chunk: Option<Chunk>) -> Chunk {
        chunk.unwrap_or(0)
    }

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
}

#[derive(Debug, Copy, Clone)]
pub struct CombinedChunks<A: ChunkRead, B: ChunkRead, O: BinChunkOp> {
    pub a: A,
    pub b: B,
    pub op: O,
}

impl<A: ChunkRead> ChunkRead for NotChunks<A> {
    fn get_chunk(self, idx_of_chunk: usize) -> Option<Chunk> {
        Some(!self.a.get_chunk(idx_of_chunk).unwrap_or(0))
    }
}

impl<A: ChunkRead, B: ChunkRead, O: BinChunkOp> ChunkRead for &CombinedChunks<A, B, O> {
    fn get_chunk(self, idx_of_chunk: usize) -> Option<Chunk> {
        self.op.combine_chunks(self.a.get_chunk(idx_of_chunk), self.b.get_chunk(idx_of_chunk))
    }
}
