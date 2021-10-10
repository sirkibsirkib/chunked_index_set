use super::{Chunk, ChunkRead};

pub mod bin_ops;

///////////////////////////////

// pub struct NotChunks<'a, A: ChunkRead + ?Sized>(pub &'a A);

pub struct EmptyIndexSet;

impl ChunkRead for EmptyIndexSet {
    fn get_chunk(&self, _: usize) -> Option<usize> {
        None
    }
    fn no_chunks_from(&self) -> usize {
        0
    }
}

pub trait BinChunkOp: Copy {
    /// NONE means zero chunk AND all subsequent chunks are zero
    fn combine_chunks(self, a: Option<Chunk>, b: Option<Chunk>) -> Option<Chunk>;
    fn combine_no_chunks_from(self, ncfa: usize, ncfb: usize) -> usize;
    fn combine_readers<'a, A: ChunkRead, B: ChunkRead>(
        self,
        a: &'a A,
        b: &'a B,
    ) -> CombinedChunks<'a, A, B, Self> {
        CombinedChunks { a, b, op: self }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct CombinedChunks<'a, A: ChunkRead + ?Sized, B: ChunkRead + ?Sized, O: BinChunkOp> {
    pub a: &'a A,
    pub b: &'a B,
    pub op: O,
}
///////////////////////////////

impl<A: ChunkRead + ?Sized, B: ChunkRead + ?Sized, O: BinChunkOp> ChunkRead
    for CombinedChunks<'_, A, B, O>
{
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<Chunk> {
        self.op.combine_chunks(self.a.get_chunk(idx_of_chunk), self.b.get_chunk(idx_of_chunk))
    }
    fn no_chunks_from(&self) -> usize {
        self.op.combine_no_chunks_from(self.a.no_chunks_from(), self.b.no_chunks_from())
    }
}
