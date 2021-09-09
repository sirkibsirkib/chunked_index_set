use crate::*;
use core::ops::Range;
use fastrand::Rng;
use std::collections::HashSet;
use std::iter::FromIterator;

type HSet = HashSet<usize>;

// fn random_elements()

fn usize_stream(seed: u64, bounds: Range<usize>) -> impl Iterator<Item = usize> {
    let rng = fastrand::Rng::with_seed(seed);
    std::iter::repeat_with(move || rng.usize(bounds.clone()))
}

fn paramd_usize_stream() -> impl Iterator<Item = usize> {
    usize_stream(SEED, BOUNDS.clone()).take(COUNT)
}

// fn build_and_drain()
const BOUNDS: Range<usize> = 0..600;
const COUNT: usize = 900;
const SEED: u64 = 3;

#[test]
fn collect_count_and_iter_count() {
    let w = Words::from_set_bits(paramd_usize_stream());
    assert_eq!(w.count_set_bits(), w.iter_set_bits().count())
}

#[test]
fn words_hset_collect_count_eq() {
    let w = Words::from_set_bits(paramd_usize_stream());
    let h = HSet::from_iter(paramd_usize_stream());
    assert_eq!(w.count_set_bits(), h.len());
}

#[test]
fn words_covers_hset() {
    let mut w = Words::from_set_bits(paramd_usize_stream());
    let mut h = HSet::from_iter(paramd_usize_stream());
    for i in w.drain().iter_set_bits() {
        assert!(h.remove(&i));
    }
    assert!(w.is_empty() && h.is_empty());
}

#[test]
fn hset_covers_words() {
    let mut w = Words::from_set_bits(paramd_usize_stream());
    let mut h = HSet::from_iter(paramd_usize_stream());
    for i in h.drain() {
        assert!(w.remove_bit(i));
    }
    assert!(w.is_empty() && h.is_empty());
}
