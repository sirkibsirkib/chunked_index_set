use super::*;

#[derive(Debug, Copy, Clone)]
pub struct ChunkIter<A: ChunkRead> {
    a: A,
    idx_of_next_chunk: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct IndexIter<A: ChunkRead> {
    pub(crate) wi: ChunkIter<A>,
    pub(crate) cached: usize,
}

impl<A: ChunkRead> ChunkIter<A> {
    pub fn new(a: A) -> Self {
        Self { a, idx_of_next_chunk: 0 }
    }
}

impl<A: ChunkRead> Iterator for ChunkIter<A> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let next = self.a.get_chunk(self.idx_of_next_chunk)?;
        self.idx_of_next_chunk += 1;
        Some(next)
    }
}

impl<A: ChunkRead> IndexIter<A> {
    pub fn new(a: A) -> Self {
        Self { wi: a.iter_chunks(), cached: 0 }
    }
}
impl<A: ChunkRead> Iterator for IndexIter<A> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        while self.cached == 0 {
            self.cached = self.wi.next()?;
        }
        // self.cached is NONZERO
        let idx_in_chunk = self.cached.trailing_zeros();
        self.cached &= !(1 << idx_in_chunk);
        let cba = ChunkBitAddr { idx_in_chunk, idx_of_chunk: self.wi.idx_of_next_chunk - 1 };
        Some(cba.to_bit_idx())
    }
}
