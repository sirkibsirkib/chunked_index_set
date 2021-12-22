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
        assert_eq!(w.len(), w.iter().count());
    }
}

#[test]
fn chunks_hset_collect_count_eq() {
    for range in RANGES.iter().cloned() {
        let w = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let h = HSet::from_iter(stream(0, range));
        assert_eq!(w.len(), h.len());
    }
}

#[test]
fn chunks_covers_hset() {
    for range in RANGES.iter().cloned() {
        let w = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let mut h = HSet::from_iter(stream(0, range));
        for i in w.iter() {
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
        // println!("{:?}\n{:?}", &w, &h);
        for &i in h.iter() {
            assert!(w.remove(i));
        }
        // println!("w {:?}", &w);
        assert!(w.is_empty());
    }
}

#[test]
fn iter_and_collect_indices() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range));
        let b = IndexSet::from_iter(a.iter());
        assert_eq!(a, b)
    }
}

#[test]
fn iter_and_collect_chunks() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::from_iter(stream(0, range));
        let b = IndexSet::<2>::from_chunk_iter(a.iter_chunks());
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
            assert_eq!(a.contains(i) && b.contains(i), a_and_b.contains(i));
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
            assert_eq!(a.contains(i) || b.contains(i), a_or_b.contains(i));
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
            assert_eq!(a.contains(i) ^ b.contains(i), a_xor_b.contains(i));
        }
    }
}

#[test]
fn diff_diffs() {
    for range in RANGES.iter().cloned() {
        let a = IndexSet::<2>::from_iter(stream(0, range.clone()));
        let b = IndexSet::<2>::from_iter(stream(1, range.clone()));

        let a_diff_b = a.without(&b).to_index_set::<2>();

        for i in range {
            assert_eq!(a.contains(i) & !b.contains(i), a_diff_b.contains(i));
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

#[test]
fn from_chunk_slice() {
    let a = IndexSet::<2>::from_chunk_slice(&[0b11000]);
    let b: IndexSet<2> = [4, 3, 3].iter().copied().collect();
    assert_eq!(a, b);
}

#[test]
fn largest() {
    let set = IndexSet::<2>::from_iter([1, 2, 19, 91, 93]);
    assert_eq!(set.max_element(), Some(93));
}

#[test]
fn smallest() {
    let set = IndexSet::<2>::from_iter([19, 91, 93]);
    assert_eq!(set.min_element(), Some(19));
}

#[test]
fn powerset_order() {
    let chunk: Chunk = 0b11010;
    let mut set = IndexSet::<3>::from_chunk_slice(&[chunk]);
    let count =
        std::iter::repeat_with(|| set.try_decrease_in_powerset_order()).take_while(|&x| x).count();
    assert_eq!(count, 0b11010);
}

#[test]
fn insert_all_in_range() {
    let rng = fastrand::Rng::with_seed(31644);
    const N: usize = 242;
    const TESTS: usize = 2_000;
    let mut stream = std::iter::repeat_with(|| rng.usize(0..N)..rng.usize(N..N * 2));
    let mut set = IndexSet::<17>::default();
    for _tests in 0..TESTS {
        let range = stream.next().unwrap();
        set.insert_all_in_range(range.clone());
        assert_eq!(set.len(), range.len());
        for i in range {
            assert!(set.contains(i));
        }
        set.clear();
    }
}

#[test]
fn insert_then_remove_all_in_range() {
    let rng = fastrand::Rng::with_seed(364);
    const N: usize = 242;
    const TESTS: usize = 2_000;
    let mut stream = std::iter::repeat_with(|| rng.usize(0..N)..rng.usize(N..N * 2));
    let mut set = IndexSet::<17>::default();
    for _tests in 0..TESTS {
        set.insert_all_in_range(0..N * 2);
        let remove_range = stream.next().unwrap();
        set.remove_all_in_range(remove_range.clone());
        assert_eq!(set.len(), N * 2 - remove_range.len());
        for i in remove_range {
            assert!(!set.contains(i));
        }
        set.clear();
    }
}

#[test]
fn chunk_list_cmp() {
    let rng = fastrand::Rng::with_seed(8985);
    const TESTS: usize = 2_000;

    let mut a = IndexSet::<1>::default();
    let mut b = IndexSet::<3>::default();

    for _ in 0..TESTS {
        // populate
        const N: usize = 4;
        for _ in 0..N {
            a.insert(rng.usize(0..8));
            b.insert(rng.usize(0..8));
        }

        // test ordering
        use core::cmp::Ordering::Equal;
        assert_eq!(a.chunk_list_cmp(&b) == Equal, a.set_cmp(&b) == Some(Equal));

        // clear
        a.clear();
        b.clear();
    }
}
