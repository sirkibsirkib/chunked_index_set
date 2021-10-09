use super::*;

use crate::combinators::bin_ops::*;

/*
pub trait ChunkAccess: TryChunkAccess {
    fn get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> &mut Chunk {
        self.try_get_mut_chunk_creating(idx_of_chunk).unwrap()
    }
    fn insert_index(&mut self, bit_idx: Index) -> bool {
        self.try_insert_index(bit_idx).unwrap()
    }

    fn from_indexes<I: IntoIterator<Item = Index>>(into_iter: I) -> Self
    where
        Self: Default + Sized,
    {
        let mut me = Self::default();
        for index in into_iter {
            me.insert_index(index);
        }
        me
    }
}
pub trait TryChunkAccess: ChunkRead {
    fn try_get_mut_chunk_creating(&mut self, idx_of_chunk: usize) -> Option<&mut Chunk>;
    ////////
    fn try_clone_from<A: ChunkRead>(&mut self, a: A) {
        self.clear();
        let mut idx_of_chunk = 0;
        while let (Some(src), Some(dest)) =
            (a.get_chunk(idx_of_chunk), self.try_get_mut_chunk_creating(idx_of_chunk))
        {
            *dest = src;
            idx_of_chunk += 1;
        }
    }
    fn try_get_mut_chunk_existing(&mut self, idx_of_chunk: usize) -> Option<&mut Chunk> {
        self.try_get_mut_chunk_creating(idx_of_chunk)
    }
    fn try_insert_index(&mut self, bit_idx: Index) -> Result<bool, ()> {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        if let Some(chunk) = self.try_get_mut_chunk_creating(cba.idx_of_chunk) {
            let mask = cba.chunk_mask();
            let was = *chunk & mask != 0;
            *chunk |= mask;
            Ok(was)
        } else {
            Err(())
        }
    }
    fn remove_index(&mut self, bit_idx: Index) -> bool {
        let cba = ChunkBitAddr::from_bit_idx(bit_idx);
        if let Some(chunk) = self.try_get_mut_chunk_existing(cba.idx_of_chunk) {
            let mask = cba.chunk_mask();
            let was = *chunk & mask != 0;
            if was {
                *chunk &= !mask;
            }
            was
        } else {
            false
        }
    }
    fn truncate(&mut self, as_of_chunk_index: usize) {
        for idx_of_chunk in as_of_chunk_index.. {
            match self.try_get_mut_chunk_existing(idx_of_chunk) {
                None => return,
                Some(chunk) => *chunk = 0,
            }
        }
    }
    fn clear(&mut self) {
        self.truncate(0)
    }
    fn overwrite_combined<B: ChunkRead, O: BinChunkOp>(&mut self, op: O, b: &B) {
        for idx_of_chunk in 0.. {
            let src: Option<Chunk> =
                op.combine_chunks(self.get_chunk(idx_of_chunk), b.get_chunk(idx_of_chunk));
            if let Some(src) = src {
                if let Some(dest) = self.try_get_mut_chunk_creating(idx_of_chunk) {
                    *dest = src;
                }
            } else {
                self.truncate(idx_of_chunk);
                return;
            }
        }
    }
    fn remove_all<A: ChunkRead>(&mut self, a: &A) {
        for idx_of_chunk in 0.. {
            match (self.try_get_mut_chunk_existing(idx_of_chunk), a.get_chunk(idx_of_chunk)) {
                (None, _) | (_, None) => return,
                (Some(dest), Some(src)) => *dest &= !src,
            }
        }
    }
    fn insert_all<A: ChunkRead>(&mut self, a: &A) {
        for idx_of_chunk in 0.. {
            match a.get_chunk(idx_of_chunk) {
                None => return,
                Some(0) => {}
                Some(src) => match self.try_get_mut_chunk_creating(idx_of_chunk) {
                    None => return,
                    Some(dest) => *dest |= src,
                },
            }
        }
    }
}
*/

pub trait ChunkRead {
    fn get_chunk(&self, idx_of_chunk: Index) -> Option<Chunk>;
    ///////
    fn to_index_set<const N: usize>(&self) -> PackedIndexSet<N> {
        PackedIndexSet::from_chunks(self.iter_chunks())
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
    fn not_chunks(&self) -> NotChunks<Self> {
        NotChunks(self)
    }
    fn combine_chunks<'a, B: ChunkRead, O: BinChunkOp>(
        &'a self,
        op: O,
        b: &'a B,
    ) -> CombinedChunks<Self, B, O> {
        CombinedChunks { a: self, b, op }
    }
    fn nand<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunks<Self, B, Nand> {
        self.combine_chunks(Nand, b)
    }
    fn or<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunks<Self, B, Or> {
        self.combine_chunks(Or, b)
    }
    fn xor<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunks<Self, B, Xor> {
        self.combine_chunks(Xor, b)
    }
    fn and<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunks<Self, B, And> {
        self.combine_chunks(And, b)
    }
    fn diff<'a, B: ChunkRead>(&'a self, b: &'a B) -> CombinedChunks<Self, B, Diff> {
        self.combine_chunks(Diff, b)
    }
}
