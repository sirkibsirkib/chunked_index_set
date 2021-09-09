use super::*;

use crate::combinators::bin_ops::*;

pub trait ChunkRead: Copy {
    fn get_chunk(self, idx_of_chunk: Index) -> Option<Chunk>;
    ///////
    fn to_index_set(self) -> IndexSet {
        IndexSet::from_chunks(self.iter_chunks().collect())
    }

    fn iter_indexes(self) -> IndexIter<Self> {
        IndexIter::new(self)
    }
    fn iter_chunks(self) -> ChunkIter<Self> {
        ChunkIter::new(self)
    }
    fn count_indexes(self) -> usize {
        self.iter_chunks().map(|chunk: Chunk| chunk.count_ones() as usize).sum()
    }
    fn buffer_chunks_into(self, buf: &mut Vec<Chunk>) {
        for chunk in self.iter_chunks() {
            buf.push(chunk)
        }
    }
    fn contains_index(self, bit_idx: Index) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        match self.get_chunk(cba.idx_of_chunk) {
            None => false,
            Some(chunk) => chunk & cba.chunk_mask() > 0,
        }
    }
    fn not_chunks(self) -> NotChunks<Self> {
        NotChunks { a: self }
    }
    fn combine_chunks<B: ChunkRead, O: BinChunkOp>(
        self,
        b: B,
        op: O,
    ) -> CombinedChunks<Self, B, O> {
        CombinedChunks { a: self, b, op }
    }
    fn nand<B: ChunkRead>(self, b: B) -> CombinedChunks<Self, B, Nand> {
        self.combine_chunks(b, Nand)
    }
    fn or<B: ChunkRead>(self, b: B) -> CombinedChunks<Self, B, Or> {
        self.combine_chunks(b, Or)
    }
    fn xor<B: ChunkRead>(self, b: B) -> CombinedChunks<Self, B, Xor> {
        self.combine_chunks(b, Xor)
    }
    fn and<B: ChunkRead>(self, b: B) -> CombinedChunks<Self, B, And> {
        self.combine_chunks(b, And)
    }
    fn diff<B: ChunkRead>(self, b: B) -> CombinedChunks<Self, B, Diff> {
        self.combine_chunks(b, Diff)
    }
}
