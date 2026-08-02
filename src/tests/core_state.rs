use super::*;

#[test]
fn push_returns_true_only_for_new_items() {
    let mut v: DedupVec<u32> = DedupVec::new();
    assert!(v.push(1));
    assert!(v.push(2));
    assert!(!v.push(1));
    assert_eq!(&*v, &[1, 2]);
}

#[test]
fn extend_preserves_first_seen_order_and_dedups() {
    let mut v: DedupVec<u32> = DedupVec::new();
    v.extend([3, 3, 1, 2, 1, 4]);
    assert_eq!(&*v, &[3, 1, 2, 4]);
}

#[test]
fn clear_resets_items_and_membership() {
    let mut v: DedupVec<u32> = DedupVec::new();
    v.extend([1, 2, 3]);
    v.clear();
    assert!(v.is_empty());
    assert!(v.push(1));
    assert_eq!(&*v, &[1]);
}

#[test]
fn into_vec_returns_ordered_items() {
    let mut v: DedupVec<(u8, u32)> = DedupVec::new();
    v.push((0, 10));
    v.push((1, 20));
    v.push((0, 10));
    assert_eq!(v.into_vec(), vec![(0, 10), (1, 20)]);
}

#[test]
fn deref_exposes_slice_len_and_is_empty() {
    let mut v: DedupVec<u32> = DedupVec::new();
    assert!(v.is_empty());
    assert_eq!(v.len(), 0);
    v.push(7);
    assert_eq!(v.len(), 1);
    assert!(!v.is_empty());
}

#[test]
fn default_is_empty() {
    let v: DedupVec<u32> = DedupVec::default();
    assert!(v.is_empty());
}
