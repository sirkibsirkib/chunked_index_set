use super::*;

use crate::combinators::bin_ops::*;

pub trait ChunkRead {
    fn get_chunk(&self, idx_of_chunk: Index) -> Option<Chunk>;
    fn zero_chunks_from(&self) -> usize;
    ///////
    fn zero_chunks_from_exact(&self) -> usize {

            // scan from back to front looking for 1st nonzero chunk
            let mut at = self.zero_chunks_from();
            // at points to SOME nonzero chunk. Is there another before it?
            while at > 0 && self.get_chunk(at-1).unwrap_or(0) == 0 {
                // yep. there's another one at at-1
                at -= 1;
            }
            at
    }
    fn to_index_set<const N: usize>(&self) -> IndexSet<N> {
        let mut me = IndexSet::<N>::with_chunk_capacity(self.zero_chunks_from_exact());
        for (index_of_chunk, write_chunk) in me.as_chunks_mut().iter_mut().enumerate() {
            if let Some(read_chunk) = self.get_chunk(index_of_chunk) {
                *write_chunk = read_chunk;
            } else {
                break
            }
        }
        me
    }
    fn is_subset_of<A: ChunkRead>(&self, other: &A) -> bool {
        use core::cmp::Ordering::*;
        match self.set_cmp(other) {
            Some(Equal | Less) => true,
            Some(Greater) | None => true,
        }
    }
    fn is_superset_of<A: ChunkRead>(&self, other: &A) -> bool {
        use core::cmp::Ordering::*;
        match self.set_cmp(other) {
            Some(Equal | Greater) => true,
            Some(Less) | None => true,
        }
    }
    fn is_disjoint_with<A: ChunkRead>(&self, other: &A) -> bool {
        self.combine_chunks(And, other).is_empty()
    }
    fn set_cmp<A: ChunkRead>(&self, other: &A) -> Option<core::cmp::Ordering> {
        let mut ord = core::cmp::Ordering::Equal;
        use core::cmp::Ordering as O;
        for idx_of_chunk in 0.. {
            let s = self.get_chunk(idx_of_chunk);
            let o = other.get_chunk(idx_of_chunk);
            if s.is_none() && o.is_none() {
                return Some(ord);
            }
            let s = s.unwrap_or(0);
            let o = o.unwrap_or(0);

            let snoto = s & !o != 0;
            if snoto {
                // self has 1+ indices that other does not have
                ord = match ord {
                    O::Equal | O::Greater => O::Greater,
                    O::Less => return None,
                };
            }
            let onots = o & !s != 0;
            if onots {
                // other has 1+ indices that self does not have
                ord = match ord {
                    O::Equal | O::Less => O::Less,
                    O::Greater => return None,
                };
            }
        }
        unreachable!()
    }
    fn is_empty(&self) -> bool {
        self.iter_chunks().all(|chunk| chunk == 0)
    }
    fn displayable(&self) -> DisplayableIndexSet<Self> {
        DisplayableIndexSet(self)
    }
    fn iter_indexes(&self) -> IndexIter<Self> {
        IndexIter::new(self)
    }
    fn iter_chunks(&self) -> ChunkIter<Self> {
        ChunkIter::new(self)
    }
    fn count_indexes(&self) -> usize {
        self.iter_chunks().map(|chunk: Chunk| chunk.count_ones() as usize).sum()
    }
    fn buffer_chunks_into(&self, buf: &mut Vec<Chunk>) {
        for chunk in self.iter_chunks() {
            buf.push(chunk)
        }
    }
    fn contains_index(&self, bit_idx: Index) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        match self.get_chunk(cba.idx_of_chunk) {
            None => false,
            Some(chunk) => chunk & cba.chunk_mask() != 0,
        }
    }
    fn combine_chunks<'a, B: ChunkRead, O: BinChunkOp>(
        &'a self,
        op: O,
        b: &'a B,
    ) -> CombinedChunkReads<Self, B, O> {
        CombinedChunkReads { a: self, b, op }
    }
    fn or<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunkReads<Self, B, Or> {
        self.combine_chunks(Or, b)
    }
    fn xor<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunkReads<Self, B, Xor> {
        self.combine_chunks(Xor, b)
    }
    fn and<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunkReads<Self, B, And> {
        self.combine_chunks(And, b)
    }
    fn diff<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunkReads<Self, B, Diff> {
        self.combine_chunks(Diff, b)
    }
}
