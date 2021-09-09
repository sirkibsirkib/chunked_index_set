use super::{Chunk, ChunkRead};

pub mod bin_ops;

///////////////////////////////

#[derive(Debug, Copy, Clone)]
pub struct NotChunks<'a, A: ChunkRead> {
    pub a: &'a A,
}

pub trait BinChunkOp: Copy {
    /// NONE means zero chunk AND all subsequent chunks are zero
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk>;
}

#[derive(Debug, Copy, Clone)]
pub struct CombinedChunks<'a, A: ChunkRead, B: ChunkRead, O: BinChunkOp> {
    pub a: &'a A,
    pub b: &'a B,
    pub op: O,
}
///////////////////////////////
impl<A: ChunkRead> ChunkRead for NotChunks<'_, A> {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<Chunk> {
        Some(!self.a.get_chunk(idx_of_chunk).unwrap_or(0))
    }
}

impl<A: ChunkRead, B: ChunkRead, O: BinChunkOp> ChunkRead for CombinedChunks<'_, A, B, O> {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<Chunk> {
        self.op.combine_chunks(self.a.get_chunk(idx_of_chunk), self.b.get_chunk(idx_of_chunk))
    }
}
