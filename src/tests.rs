use crate::*;

#[test]
fn collect_eq() {
    assert_eq!(
        Words::from_set_bits([1, 2, 3]),
        Words::from_set_bits([3, 2, 1])
    )
}
