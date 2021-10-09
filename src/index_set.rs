use super::{Chunk, ChunkRead, CHUNK_BYTES};
use crate::ChunkBitAddr;
use crate::Index;
use core::fmt::Debug;
use core::iter::FromIterator;

union PackedChunkStorage<const N: usize> {
    heap_chunks_ptr: *mut Chunk,
    stack_chunks: [Chunk; N],
}
impl<const N: usize> Debug for PackedIndexSet<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        self.as_chunks().fmt(f)
    }
}
impl<const N: usize> PartialEq for PackedIndexSet<N> {
    fn eq(&self, other: &Self) -> bool {
        self.set_cmp(other) == Some(core::cmp::Ordering::Equal)
    }
}
pub struct PackedIndexSet<const N: usize> {
    // invariants:
    // N <= self.chunk_count
    // if self.chunk_count == N: using stack_chunks
    // if N < self.chunk_count : using heap_chunks_ptr; points to [usize; self.chunk_count] on heap
    packed_chunk_storage: PackedChunkStorage<N>,
    chunk_count: usize,
}
impl<const N: usize> Default for PackedIndexSet<N> {
    fn default() -> Self {
        Self::with_chunk_capacity(0)
    }
}
impl<const N: usize> Clone for PackedIndexSet<N> {
    fn clone(&self) -> Self {
        let mut new = Self::with_chunk_capacity(self.chunk_count);
        for (src, dest) in self.as_chunks().iter().zip(new.as_chunks_mut().iter_mut()) {
            *dest = *src;
        }
        new
    }
}
impl<const N: usize> PackedIndexSet<N> {
    pub fn with_chunk_capacity(mut chunk_count: usize) -> Self {
        chunk_count = chunk_count.max(N);
        let packed_chunk_storage = if chunk_count == N {
            // stack
            PackedChunkStorage { stack_chunks: [0; N] }
        } else {
            // N < chunk_count
            // allocate chunk_count chunks on the heap
            assert!(chunk_count <= usize::MAX / CHUNK_BYTES);
            let layout = unsafe {
                // will be valid layout. Cannot be zero. cannot be misaligned. Cannot be too large.
                std::alloc::Layout::from_size_align_unchecked(
                    CHUNK_BYTES * chunk_count,
                    CHUNK_BYTES,
                )
            };
            let heap_chunks_ptr = unsafe {
                // safe! layout.size() != 0
                std::alloc::alloc_zeroed(layout)
            } as *mut usize;
            PackedChunkStorage { heap_chunks_ptr }
        };
        Self { packed_chunk_storage, chunk_count }
    }
    pub fn as_chunks(&self) -> &[Chunk] {
        unsafe {
            let chunks_ptr: *const Chunk = if self.chunk_count == N {
                // stack!
                self.packed_chunk_storage.stack_chunks.as_ptr()
            } else {
                // heap!
                self.packed_chunk_storage.heap_chunks_ptr
            };
            std::slice::from_raw_parts(chunks_ptr, self.chunk_count)
        }
    }
    pub fn from_chunks<I: IntoIterator<Item = Chunk>>(it: I) -> Self {
        let it = it.into_iter();
        let mut me = Self::with_chunk_capacity(it.size_hint().0);
        for (idx_of_chunk, read_chunk) in it.enumerate() {
            if me.chunk_count <= idx_of_chunk {
                me.grow_to_accomodate(idx_of_chunk)
            }

            let write_chunk = unsafe {
                // certainly in bounds
                me.as_chunks_mut().get_unchecked_mut(idx_of_chunk)
            };
            *write_chunk = read_chunk;
        }
        me
    }
    fn as_chunks_mut(&mut self) -> &mut [Chunk] {
        unsafe {
            let chunks_ptr: *mut Chunk = if self.chunk_count == N {
                // stack!
                self.packed_chunk_storage.stack_chunks.as_mut_ptr()
            } else {
                // heap!
                self.packed_chunk_storage.heap_chunks_ptr
            };
            std::slice::from_raw_parts_mut(chunks_ptr, self.chunk_count)
        }
    }
    pub fn contains(&self, bit_idx: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        self.as_chunks()
            .get(cba.idx_of_chunk)
            .map(|chunk| *chunk & cba.chunk_mask() != 0)
            .unwrap_or(false)
    }
    pub fn remove(&mut self, bit_idx: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        self.as_chunks_mut()
            .get_mut(cba.idx_of_chunk)
            .map(|chunk| {
                let was_set = *chunk & cba.chunk_mask() != 0;
                *chunk &= !cba.chunk_mask();
                was_set
            })
            .unwrap_or(false)
    }
    fn grow_to_accomodate(&mut self, idx_of_new_chunk: usize) {
        assert!(self.chunk_count <= idx_of_new_chunk);
        let mut new = Self::with_chunk_capacity(
            idx_of_new_chunk
                .checked_add(1)
                .and_then(usize::checked_next_power_of_two)
                .expect("out of capacity!"),
        );
        for (src, dest) in self.as_chunks().iter().zip(new.as_chunks_mut()) {
            *dest = *src;
        }
        *self = new; // drops current
    }
    pub fn insert(&mut self, bit_idx: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        if self.chunk_count <= cba.idx_of_chunk {
            self.grow_to_accomodate(cba.idx_of_chunk);
        }
        let chunk = unsafe {
            // certainly in bounds
            self.as_chunks_mut().get_unchecked_mut(cba.idx_of_chunk)
        };
        let was_unset = *chunk & cba.chunk_mask() == 0;
        *chunk |= cba.chunk_mask();
        was_unset
    }
    pub fn add_all<R: ChunkRead>(&mut self, r: &R) {
        for idx_of_chunk in 0.. {
            if let Some(read_chunk) = r.get_chunk(idx_of_chunk) {
                if read_chunk != 0 {
                    if self.chunk_count <= idx_of_chunk {
                        self.grow_to_accomodate(idx_of_chunk);
                    }
                    let write_chunk = unsafe {
                        // certainly in bounds
                        self.as_chunks_mut().get_unchecked_mut(idx_of_chunk)
                    };
                    *write_chunk |= read_chunk;
                }
            } else {
                return;
            }
        }
    }
    pub fn remove_all<R: ChunkRead>(&mut self, r: &R) {
        for (idx_of_chunk, write_chunk) in self.as_chunks_mut().iter_mut().enumerate() {
            if let Some(read_chunk) = r.get_chunk(idx_of_chunk) {
                if read_chunk != 0 {
                    *write_chunk &= !read_chunk;
                }
            } else {
                return;
            }
        }
    }
    pub fn shrink_to_fit(&mut self) {
        let chunks = self.as_chunks();
        for (idx_of_chunk, &chunk) in chunks.iter().enumerate().skip(N).rev() {
            if chunk != 0 {
                // largest chunk!
                if idx_of_chunk + 1 == self.chunk_count {
                    // don't bother
                    return;
                }
                let mut new = Self::with_chunk_capacity(idx_of_chunk + 1);
                for (src, dest) in chunks[0..idx_of_chunk + 1].iter().zip(new.as_chunks_mut()) {
                    *dest = *src;
                }
                *self = new;
                return;
            }
        }
    }
    pub fn clear(&mut self) {
        for chunk in self.as_chunks_mut() {
            *chunk = 0;
        }
    }
}
impl<const N: usize> ChunkRead for PackedIndexSet<N> {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<Chunk> {
        self.as_chunks().get(idx_of_chunk).copied()
    }
}
impl<const N: usize> Drop for PackedIndexSet<N> {
    fn drop(&mut self) {
        if N < self.chunk_count {
            let layout = unsafe {
                // will be valid layout. Cannot be zero. cannot be misaligned. Cannot be too large.
                std::alloc::Layout::from_size_align_unchecked(
                    CHUNK_BYTES * self.chunk_count,
                    CHUNK_BYTES,
                )
            };
            unsafe {
                // layout matches allocation
                std::alloc::dealloc(self.packed_chunk_storage.heap_chunks_ptr as *mut u8, layout)
            };
        }
        // todo
    }
}
impl<const N: usize> FromIterator<Index> for PackedIndexSet<N> {
    fn from_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
        let mut m = Self::default();
        for index in into_iter.into_iter() {
            m.insert(index);
        }
        m
    }
}
/////

// #[derive(Default, Debug, Clone, PartialEq, Eq)]
// pub struct IndexSet {
//     // invariant: last chunk is non-zero
//     // derived Eq and PartialEq rely on this invariant
//     chunks: Vec<Chunk>,
// }

// impl FromIterator<Index> for IndexSet {
//     fn from_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
//         Self::from_indexes(into_iter)
//     }
// }
// impl IndexSet {
//     /// restores invariant
//     pub(crate) fn pop_zero_tail(&mut self) {
//         while let Some(0) = self.chunks.last() {
//             self.chunks.pop();
//         }
//     }
//     // pub fn with_index_capacity(index_count: usize) -> Self {}
//     pub fn with_chunk_capacity(chunk_count: usize) -> Self {
//         Self { chunks: Vec::with_capacity(chunk_count) }
//     }
//     pub fn shrink_to_fit(&mut self) {
//         self.chunks.shrink_to_fit()
//     }

//     pub fn from_chunks(chunks: Vec<Chunk>) -> Self {
//         let mut me = Self { chunks };
//         me.pop_zero_tail();
//         me
//     }
//     pub fn from_index_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
//         let mut me = Self::default();
//         for index in into_iter {
//             let _ = me.try_insert_index(index);
//         }
//         me
//     }
// }

// impl ChunkRead for IndexSet {
//     fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
//         self.chunks.as_slice().get_chunk(idx_of_chunk)
//     }
// }

impl ChunkRead for [Chunk] {
    /// ChunkRead does not rely on IndexSet's non-zero-last-word invariant. Exposing this to the user is OK
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        self.get(idx_of_chunk).copied()
    }
}
// impl TryChunkAccess for IndexSet {
//     fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
//         while self.chunks.len() <= idx_of_chunk {
//             self.chunks.push(0)
//         }
//         Some(unsafe {
//             // safe!
//             self.chunks.get_unchecked_mut(idx_of_chunk)
//         })
//     }
//     fn try_get_mut_chunk_existing(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
//         self.chunks.as_mut_slice().try_get_mut_chunk_existing(idx_of_chunk)
//     }
// }
// impl ChunkAccess for IndexSet {}

// impl TryChunkAccess for [Chunk] {
//     fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
//         self.get_mut(idx_of_chunk)
//     }
// }

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
// impl TryChunkAccess for Chunk {
//     fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut usize> {
//         if idx_of_chunk == 0 {
//             Some(self)
//         } else {
//             None
//         }
//     }
// }
