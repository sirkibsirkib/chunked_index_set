use crate::combinators::bin_ops::Or;
use crate::*;
use core::ops::Range;

use std::collections::HashSet;
use std::iter::FromIterator;

type HSet = HashSet<usize>;
const RANGES: &[Range<Index>] = &[
    0..20,
    0..40,
    0..80,
    0..140,
    0..190,
    0..250,
    200..300,
    0..280,
    0..500,
    0..600,
    0..700,
    600..800,
    0..900,
    0..1000,
];

fn stream(seed: u64, bounds: Range<usize>) -> impl Iterator<Item = usize> {
    let rng = fastrand::Rng::with_seed(seed);
    let len = bounds.len() * usize::BITS as usize / rng.usize(4..32);
    std::iter::repeat_with(move || rng.usize(bounds.clone())).take(len)
}

#[test]
fn collect_count_and_iter_count() {
    for range in RANGES.iter().cloned() {
        let w = IndexSet::<2>::from_iter(stream(0, range));
        assert_eq!(w.count_indexes(), w.iter_indexes().count());
    }
}

#[test]
fn chunks_hset_collect_count_eq() {
    for range in RANGES.iter().cloned() {
        let w = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let h = HSet::from_iter(stream(0, range));
        assert_eq!(w.count_indexes(), h.len());
    }
}

#[test]
fn chunks_covers_hset() {
    for range in RANGES.iter().cloned() {
        let w = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let mut h = HSet::from_iter(stream(0, range));
        for i in w.iter_indexes() {
            assert!(h.remove(&i));
        }
        assert!(h.is_empty());
    }
}

#[test]
fn hset_covers_chunks() {
    for range in RANGES.iter().cloned() {
        let mut w = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let h = HSet::from_iter(stream(0, range));
        println!("{:?}\n{:?}", &w, &h);
        for &i in h.iter() {
            assert!(w.remove(i));
        }
        println!("w {:?}", &w);
        assert!(w.is_empty());
    }
}

#[test]
fn iter_and_collect_indices() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range));
        let b = IndexSet::from_iter(a.iter_indexes());
        assert_eq!(a, b)
    }
}

#[test]
fn iter_and_collect_chunks() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::from_iter(stream(0, range));
        let b = IndexSet::<2>::from_chunks(a.iter_chunks());
        assert_eq!(a, b)
    }
}

#[test]
fn and_intersects() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));
        let a_and_b = a.and(&b).to_index_set::<2>();

        for i in range {
            assert_eq!(a.contains_index(i) && b.contains_index(i), a_and_b.contains_index(i));
        }
    }
}

#[test]
fn or_unions() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));

        let a_or_b = a.or(&b).to_index_set::<2>();

        for i in range {
            assert_eq!(a.contains_index(i) || b.contains_index(i), a_or_b.contains_index(i));
        }
    }
}

#[test]
fn xor_sym_diffs() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));

        let a_xor_b = a.xor(&b).to_index_set::<2>();

        for i in range {
            assert_eq!(a.contains_index(i) ^ b.contains_index(i), a_xor_b.contains_index(i));
        }
    }
}

#[test]
fn diff_diffs() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));

        let a_diff_b = a.diff(&b).to_index_set::<2>();

        for i in range {
            assert_eq!(a.contains_index(i) & !b.contains_index(i), a_diff_b.contains_index(i));
        }
    }
}

#[test]
fn stack_chunks() {
    IndexSet::<2>::default();
}

#[test]
fn combine_overwrite_is_ok() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));

        let c = a.or(&b).to_index_set::<2>();
        let d = {
            let mut d = a.clone();
            d.overwrite_from_combination(Or, &b);
            d
        };
        assert_eq!(c, d);
    }
}
