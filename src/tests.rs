use crate::*;
use core::ops::Range;

use std::collections::HashSet;
use std::iter::FromIterator;

type HSet = HashSet<usize>;

fn stream(seed: u64, bounds: Range<usize>) -> impl Iterator<Item = usize> {
    let rng = fastrand::Rng::with_seed(seed);
    std::iter::repeat_with(move || rng.usize(bounds.clone()))
}

fn seeded_stream(seed: u64) -> impl Iterator<Item = usize> {
    stream(seed, 0..900).take(500)
}

#[test]
fn collect_count_and_iter_count() {
    let w = IndexSet::from_iter(seeded_stream(0));
    assert_eq!(w.count_indexes(), w.iter_indexes().count())
}

#[test]
fn chunks_hset_collect_count_eq() {
    let w = IndexSet::from_iter(seeded_stream(0));
    let h = HSet::from_iter(seeded_stream(0));
    assert_eq!(w.count_indexes(), h.len());
}

#[test]
fn chunks_covers_hset() {
    let w = IndexSet::from_iter(seeded_stream(0));
    let mut h = HSet::from_iter(seeded_stream(0));
    for i in w.iter_indexes() {
        assert!(h.remove(&i));
    }
    assert!(h.is_empty());
}

#[test]
fn hset_covers_chunks() {
    let mut w = IndexSet::from_iter(seeded_stream(0));
    let h = HSet::from_iter(seeded_stream(0));
    for &i in h.iter() {
        assert!(w.remove_index(i));
    }
    assert!(w.is_empty());
}

#[test]
fn iter_and_collect_indices() {
    let a = IndexSet::from_iter(seeded_stream(0));
    let b = IndexSet::from_iter(a.iter_indexes());
    assert_eq!(a, b)
}

#[test]
fn iter_and_collect_chunks() {
    let a = IndexSet::from_iter(seeded_stream(0));
    let b = IndexSet::from_chunks(a.iter_chunks().collect());
    assert_eq!(a, b)
}

#[test]
fn and_intersects() {
    let a = IndexSet::from_iter(seeded_stream(0));
    let b = IndexSet::from_iter(seeded_stream(1));

    let a_and_b = a.and(&b).to_index_set();
    for i in a_and_b.iter_indexes() {
        assert!(a.contains_index(i) && b.contains_index(i))
    }
}

#[test]
fn stack_chunks() {
    let mut chunks: [usize; 3] = Default::default();
    chunks.try_insert_index(42).unwrap();
    assert!(chunks.contains_index(42));
}

#[test]
fn stack_chunk() {
    let mut chunk: usize = Default::default();
    chunk.try_insert_index(7).unwrap();
    assert!(chunk.contains_index(7));
}
