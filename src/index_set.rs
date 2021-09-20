use super::{Chunk, ChunkAccess, ChunkRead, Index, TryChunkAccess};
use core::iter::FromIterator;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct IndexSet {
    // invariant: last chunk is non-zero
    // derived Eq and PartialEq rely on this invariant
    chunks: Vec<Chunk>,
}

////////////
impl FromIterator<Index> for IndexSet {
    fn from_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
        Self::from_indexes(into_iter)
    }
}
impl IndexSet {
    /// restores invariant
    pub(crate) fn pop_zero_tail(&mut self) {
        while let Some(0) = self.chunks.last() {
            self.chunks.pop();
        }
    }
    // pub fn with_index_capacity(index_count: usize) -> Self {}
    pub fn with_chunk_capacity(chunk_count: usize) -> Self {
        Self { chunks: Vec::with_capacity(chunk_count) }
    }
    pub fn shrink_to_fit(&mut self) {
        self.chunks.shrink_to_fit()
    }

    pub fn from_chunks(chunks: Vec<Chunk>) -> Self {
        let mut me = Self { chunks };
        me.pop_zero_tail();
        me
    }
    pub fn from_index_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
        let mut me = Self::default();
        for index in into_iter {
            let _ = me.try_insert_index(index);
        }
        me
    }
}

impl ChunkRead for IndexSet {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        self.chunks.as_slice().get_chunk(idx_of_chunk)
    }
}

impl ChunkRead for [Chunk] {
    /// ChunkRead does not rely on IndexSet's non-zero-last-word invariant. Exposing this to the user is OK
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        self.get(idx_of_chunk).copied()
    }
}
impl TryChunkAccess for IndexSet {
    fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
        while self.chunks.len() <= idx_of_chunk {
            self.chunks.push(0)
        }
        Some(unsafe {
            // safe!
            self.chunks.get_unchecked_mut(idx_of_chunk)
        })
    }
    fn try_get_mut_chunk_existing(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
        self.chunks.as_mut_slice().try_get_mut_chunk_existing(idx_of_chunk)
    }
}
impl ChunkAccess for IndexSet {}

impl TryChunkAccess for [Chunk] {
    fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
        self.get_mut(idx_of_chunk)
    }
}

impl ChunkRead for Chunk {
    /// ChunkRead does not rely on IndexSet's non-zero-last-word invariant. Exposing this to the user is OK
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        if idx_of_chunk == 0 {
            Some(*self)
        } else {
            None
        }
    }
}
impl TryChunkAccess for Chunk {
    fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
        if idx_of_chunk == 0 {
            Some(self)
        } else {
            None
        }
    }
}
