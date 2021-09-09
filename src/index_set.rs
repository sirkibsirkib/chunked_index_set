use super::{Chunk, ChunkAccess, ChunkBitAddr, ChunkRead, Index, TryChunkAccess};
use core::iter::FromIterator;

#[derive(Default, Debug, Clone, Eq, PartialEq)]
pub struct IndexSet {
    // invariant: last chunk is non-zero
    // derived Eq and PartialEq rely on this invariant
    pub(crate) chunks: Vec<Chunk>,
}

pub struct IndexDrain<'a> {
    w: &'a mut IndexSet,
}

////////////

impl IndexSet {
    /// restores invariant
    fn pop_zero_tail(&mut self) {
        while let Some(0) = self.chunks.last() {
            self.chunks.pop();
        }
    }
    pub fn from_chunks(chunks: Vec<Chunk>) -> Self {
        let mut me = Self { chunks };
        me.pop_zero_tail();
        me
    }
    pub fn drain(&mut self) -> IndexDrain {
        IndexDrain { w: self }
    }
    pub fn add_all<A: ChunkRead>(&mut self, a: A) {
        let mut it = a.iter_chunks();
        for dest in self.chunks.iter_mut() {
            match it.next() {
                Some(w) => *dest |= w,
                None => return,
            }
        }
        while let Some(w) = it.next() {
            self.chunks.push(w);
        }
        self.pop_zero_tail();
    }
    pub fn remove_all<A: ChunkRead>(&mut self, a: A) {
        for i in 0.. {
            match (self.chunks.get_mut(i), a.get_chunk(i)) {
                (Some(dest), Some(src)) => *dest &= !src,
                _ => {
                    self.pop_zero_tail();
                    return;
                }
            }
        }
    }
    pub fn with_chunk_capacity(chunk_count: usize) -> Self {
        Self { chunks: Vec::with_capacity(chunk_count) }
    }
    pub fn shrink_to_fit(&mut self) {
        self.chunks.shrink_to_fit()
    }
    pub fn clear(&mut self) {
        self.chunks.clear();
    }
    pub fn insert_bit(&mut self, k: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(k);
        while self.chunks.len() <= cba.idx_of_chunk {
            self.chunks.push(0);
        }
        let chunk = unsafe { self.chunks.get_unchecked_mut(cba.idx_of_chunk) };
        let mask = cba.chunk_mask();
        let was = *chunk & mask != 0;
        *chunk |= mask;
        !was
    }
    pub fn flip_bit(&mut self, k: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(k);
        if let Some(chunk) = self.chunks.get_mut(cba.idx_of_chunk) {
            let mask = cba.chunk_mask();
            let was = *chunk & mask != 0;
            *chunk |= mask;
            was
        } else {
            self.insert_bit(k)
        }
    }
    pub fn remove_bit(&mut self, k: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(k);
        if let Some(chunk) = self.chunks.get_mut(cba.idx_of_chunk) {
            let mask = cba.chunk_mask();
            let was = *chunk & mask != 0;
            if was {
                *chunk &= !mask;
                self.pop_zero_tail();
                true
            } else {
                // can't remove. bit is already zero
                false
            }
        } else {
            // can't remove. absent chunk already encodes zero bit
            false
        }
    }
}
impl Drop for IndexDrain<'_> {
    fn drop(&mut self) {
        self.w.clear()
    }
}
impl ChunkRead for IndexDrain<'_> {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        self.w.get_chunk(idx_of_chunk)
    }
}
impl FromIterator<Index> for IndexSet {
    fn from_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
        let mut me = Self::default();
        for bit_idx in into_iter {
            me.insert_bit(bit_idx);
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
