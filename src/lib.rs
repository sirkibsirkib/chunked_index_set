use core::fmt::Display;
pub use traits::ChunkRead;

mod traits;

mod index_set;
pub use index_set::PackedIndexSet;

pub mod combinators;
use combinators::{BinChunkOp, CombinedChunks};

pub mod iterators;
use iterators::{ChunkIter, IndexIter};

#[cfg(test)]
mod tests;

/////////////////////////////////////////////

pub type Chunk = usize; // stores up to usize::BITS Indexes
pub type Index = usize; // BIT index

pub struct DisplayableIndexSet<'a, A: ChunkRead + ?Sized>(&'a A);

#[derive(Debug, Copy, Clone)]
struct ChunkBitAddr {
    idx_of_chunk: usize, // CHUNK index not BIT index
    idx_in_chunk: u32,   // invariant: in 0..usize::BITs
}
///////////////////////////////////////////////////////////////////////
const CHUNK_BYTES: usize = core::mem::size_of::<Chunk>();

fn index_count_to_chunk_count(index_count: usize) -> usize {
    (index_count + usize::BITS as usize - 1) / usize::BITS as usize
}
impl<A: ChunkRead> Display for DisplayableIndexSet<'_, A> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_set().entries(self.0.iter_indexes()).finish()
    }
}
impl ChunkBitAddr {
    fn from_bit_idx(bit_idx: usize) -> Self {
        Self {
            idx_of_chunk: bit_idx / usize::BITS as usize,
            idx_in_chunk: (bit_idx % usize::BITS as usize) as u32,
        }
    }
    const fn chunk_mask(&self) -> usize {
        1 << self.idx_in_chunk
    }
    fn to_bit_idx(self) -> usize {
        self.idx_of_chunk * usize::BITS as usize + self.idx_in_chunk as usize
    }
}
