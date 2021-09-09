use super::*;

#[derive(Debug, Copy, Clone)]
pub struct ChunkIter<'a, A: ChunkRead + ?Sized> {
    a: &'a A,
    idx_of_next_chunk: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct IndexIter<'a, A: ChunkRead + ?Sized> {
    pub(crate) wi: ChunkIter<'a, A>,
    pub(crate) cached: usize,
}

impl<'a, A: ChunkRead> ChunkIter<'a, A> {
    pub fn new(a: &'a A) -> Self {
        Self { a, idx_of_next_chunk: 0 }
    }
}

impl<A: ChunkRead> Iterator for ChunkIter<'_, A> {
    type Item = usize;
    fn next(&mut self) -> Option<usize> {
        let next = self.a.get_chunk(self.idx_of_next_chunk)?;
        self.idx_of_next_chunk += 1;
        Some(next)
    }
}

impl<'a, A: ChunkRead> IndexIter<'a, A> {
    pub fn new(a: &'a A) -> Self {
        Self { wi: a.iter_chunks(), cached: 0 }
    }
}
impl<A: ChunkRead> Iterator for IndexIter<'_, A> {
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
