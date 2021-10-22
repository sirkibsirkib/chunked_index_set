use super::{Chunk, ChunkRead, CHUNK_BYTES};
use crate::index_count_to_chunk_count;
use crate::BinChunkOp;
use crate::ChunkBitAddr;
use crate::Index;
use core::fmt::Debug;
use core::iter::FromIterator;
use core::ops::RangeTo;

union PackedChunkStorage<const N: usize> {
    heap_chunks_ptr: *mut Chunk,
    stack_chunks: [Chunk; N],
}
impl<const N: usize> Debug for IndexSet<N> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_set().entries(self.iter()).finish()
    }
}
impl<const N: usize> PartialEq for IndexSet<N> {
    fn eq(&self, other: &Self) -> bool {
        self.set_cmp(other) == Some(core::cmp::Ordering::Equal)
    }
}

/// Stores a set of indices in a contiguous array of bits packed into Chunks.
/// If chunk_capacity() <= N, the array is kept on the stack, otherwise on the heap.
///  
/// Stores an array of Chunks of the heap, each storing usize::BITS contiguous indices
/// IndexSets with <=N chunks store their data on the heap, otherwise they store it on the stack.
///
pub struct IndexSet<const N: usize> {
    // invariants:
    // N <= self.chunk_count
    // if self.chunk_count == N: using stack_chunks
    // if N < self.chunk_count : using heap_chunks_ptr; points to [usize; self.chunk_count] on heap
    packed_chunk_storage: PackedChunkStorage<N>,
    chunk_count: usize,
}
impl<const N: usize> Default for IndexSet<N> {
    fn default() -> Self {
        Self::with_chunk_capacity(0)
    }
}
impl<const N: usize> Clone for IndexSet<N> {
    fn clone(&self) -> Self {
        let mut new = Self::with_chunk_capacity(self.zero_chunks_from_exact());
        for (src, dest) in self.as_chunks().iter().zip(new.as_chunks_mut().iter_mut()) {
            *dest = *src;
        }
        new
    }
}
impl<const N: usize> IndexSet<N> {
    /// Returns the IndexSet's index capacity.
    pub fn capacity(&self) -> RangeTo<usize> {
        ..(self.chunk_count * usize::BITS as usize)
    }
    /// Creates an empty IndexSet with minimal chunk_count s.t. index_count <= capacity().
    pub fn with_min_capacity(index_count: usize) -> Self {
        Self::with_chunk_capacity(index_count_to_chunk_count(index_count))
    }
    /// Returns the IndexSet's chunk capacity
    pub fn chunk_capacity(&self) -> usize {
        self.chunk_count
    }
    /// Creates an empty IndexSet with the given chunk capacity
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
    /// Creates an IndexSet from a sequence of contiguous chunks.
    /// Result may have greater capacity than the number of chunks provided.
    pub fn from_chunk_iter<I: IntoIterator<Item = Chunk>>(it: I) -> Self {
        let it = it.into_iter();
        let mut me = Self::with_chunk_capacity(it.size_hint().0);
        for (idx_of_chunk, read_chunk) in it.enumerate() {
            if me.chunk_count <= idx_of_chunk {
                me.resize_chunks_to_accomodate(idx_of_chunk)
            }
            let write_chunk = unsafe {
                // certainly in bounds
                me.as_chunks_mut().get_unchecked_mut(idx_of_chunk)
            };
            *write_chunk = read_chunk;
        }
        me
    }
    /// Creates an IndexSet from a sequence of contiguous chunks.
    /// Result has a chunk capacity equivalent to the length of the slice.
    pub fn from_chunk_slice(chunks: &[Chunk]) -> Self {
        Self::from_chunk_iter(chunks.iter().copied())
    }
    /// Returns an immutable slice of the stored Chunks
    pub fn as_chunks(&self) -> &[Chunk] {
        unsafe {
            let chunks_ptr = if self.chunk_count == N {
                self.packed_chunk_storage.stack_chunks.as_ptr()
            } else {
                self.packed_chunk_storage.heap_chunks_ptr
            };
            std::slice::from_raw_parts(chunks_ptr, self.chunk_count)
        }
    }
    /// Returns a mutable slice of the stored Chunks
    pub fn as_chunks_mut(&mut self) -> &mut [Chunk] {
        unsafe {
            let chunks_ptr = if self.chunk_count == N {
                self.packed_chunk_storage.stack_chunks.as_mut_ptr()
            } else {
                self.packed_chunk_storage.heap_chunks_ptr
            };
            std::slice::from_raw_parts_mut(chunks_ptr, self.chunk_count)
        }
    }

    /// Removes the given index from the set. Returns whether it was present before i.e. the set has changed.
    pub fn remove(&mut self, index: Index) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(index);
        self.as_chunks_mut()
            .get_mut(cba.idx_of_chunk)
            .map(|chunk| {
                let was_set = *chunk & cba.chunk_mask() != 0;
                *chunk &= !cba.chunk_mask();
                was_set
            })
            .unwrap_or(false)
    }
    fn size_accomodating_chunk_idx(idx_of_chunk: usize) -> usize {
        idx_of_chunk
            .checked_add(1)
            .and_then(usize::checked_next_power_of_two)
            .expect("Cannot accomodate that many chunks")
    }
    /// Resizes the chunk storage, trucating chunks if chunk_count < self.chunk_count().
    /// Afterwards, has the same chunks as before in [0..chunk_count] and self.chunk_count() == chunk_count.
    pub fn resize_chunks_to(&mut self, chunk_count: usize) {
        let mut new = Self::with_chunk_capacity(chunk_count);
        for (src, dest) in self.as_chunks().iter().zip(new.as_chunks_mut()) {
            *dest = *src;
        }
        *self = new; // drops current
    }
    /// afterwards has same contents but idx_of_chunk < self.chunk_count()
    pub fn resize_chunks_to_accomodate(&mut self, idx_of_chunk: usize) {
        self.resize_chunks_to(Self::size_accomodating_chunk_idx(idx_of_chunk))
    }
    /// Adds the given index to the set. Returns whether it was absent before i.e. the set has changed.
    pub fn insert(&mut self, bit_idx: usize) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        if self.chunk_count <= cba.idx_of_chunk {
            self.resize_chunks_to_accomodate(cba.idx_of_chunk)
        }
        let chunk = unsafe {
            // certainly in bounds
            self.as_chunks_mut().get_unchecked_mut(cba.idx_of_chunk)
        };
        let was_unset = *chunk & cba.chunk_mask() == 0;
        *chunk |= cba.chunk_mask();
        was_unset
    }
    /// Equivalent to for i in r.iter_indexes() { self.insert(i); }
    pub fn insert_all<R: ChunkRead + ?Sized>(&mut self, r: &R) {
        for idx_of_chunk in 0.. {
            if let Some(read_chunk) = r.get_chunk(idx_of_chunk) {
                if read_chunk != 0 {
                    if self.chunk_count <= idx_of_chunk {
                        self.resize_chunks_to_accomodate(idx_of_chunk)
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
    /// Equivalent to for i in r.iter_indexes() { self.insert(i); }
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
    /// Minimizes self.chunk_count() without changing the set's elements.
    pub fn shrink_to_fit(&mut self) {
        let new_chunk_count = self.zero_chunks_from_exact();
        if new_chunk_count < self.chunk_count {
            let mut new = Self::with_chunk_capacity(new_chunk_count);
            for (dest, src) in new.as_chunks_mut().iter_mut().zip(self.as_chunks().iter()) {
                *dest = *src;
            }
        }
    }
    /// Leaves chunks unchanged. Afterwards, containits no indexes
    pub fn clear(&mut self) {
        for chunk in self.as_chunks_mut() {
            *chunk = 0;
        }
    }
    /// in-place equivalent to {
    ///   *self = self.clone().combined(op, other).to_index_set();
    ///   self.shrink_to_fit();
    /// }
    pub fn overwrite_from_combination<O: BinChunkOp, A: ChunkRead>(&mut self, op: O, other: &A) {
        let zcf = op.combine_readers(self, other).zero_chunks_from_exact();
        if self.chunk_count < zcf {
            self.resize_chunks_to(zcf);
        }
        for (idx_of_chunk, write_chunk) in self.as_chunks_mut().iter_mut().enumerate() {
            if let Some(combined) =
                op.combine_chunks(Some(*write_chunk), other.get_chunk(idx_of_chunk))
            {
                *write_chunk = combined;
            }
        }
    }
}
impl<const N: usize> ChunkRead for IndexSet<N> {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<Chunk> {
        self.as_chunks().get(idx_of_chunk).copied()
    }
    fn zero_chunks_from_conservative(&self) -> usize {
        self.chunk_count
    }
}
impl<const N: usize> Drop for IndexSet<N> {
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
    }
}
impl<const N: usize> FromIterator<Index> for IndexSet<N> {
    fn from_iter<I: IntoIterator<Item = Index>>(into_iter: I) -> Self {
        let mut m = Self::default();
        for index in into_iter.into_iter() {
            m.insert(index);
        }
        m
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
    fn zero_chunks_from_conservative(&self) -> usize {
        1
    }
}

impl ChunkRead for [Chunk] {
    fn get_chunk(&self, idx_of_chunk: usize) -> Option<usize> {
        self.get(idx_of_chunk).copied()
    }
    fn zero_chunks_from_conservative(&self) -> usize {
        self.len()
    }
}
