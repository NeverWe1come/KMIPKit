use super::{checked_u64_counter_add, checked_usize_counter_add};

// These direct additions exercise the checked arithmetic planned for the
// recursive schema-node and per-schema constraint counters (usize), plus
// registry aggregate node and constraint counters (u64) in T014.
#[test]
fn usize_checked_counter_accepts_maximum_then_rejects_one_more() {
    assert_eq!(
        checked_usize_counter_add(usize::MAX - 1, 1),
        Some(usize::MAX)
    );
    assert_eq!(checked_usize_counter_add(usize::MAX, 1), None);
}

#[test]
fn u64_checked_counter_accepts_maximum_then_rejects_one_more() {
    assert_eq!(checked_u64_counter_add(u64::MAX - 1, 1), Some(u64::MAX));
    assert_eq!(checked_u64_counter_add(u64::MAX, 1), None);
}
