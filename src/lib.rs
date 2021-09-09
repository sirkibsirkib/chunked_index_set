mod traits;
pub use traits::{ChunkAccess, ChunkRead, TryChunkAccess};

mod index_set;
pub use index_set::{IndexDrain, IndexSet};

pub mod combinators;
use combinators::{BinChunkOp, CombinedChunks, NotChunks};

pub mod iterators;
use iterators::{ChunkIter, IndexIter};

// #[cfg(test)]
// mod tests;

/////////////////////////////////////////////

pub type Chunk = usize; // stores up to usize::BITS Indexes
pub type Index = usize; // BIT index

#[derive(Debug, Copy, Clone)]
struct ChunkBitAddr {
    idx_of_chunk: usize, // CHUNK index not BIT index
    idx_in_chunk: u32,   // invariant: in 0..usize::BITs
}
///////////////////////////////////////////////////////////////////////

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
