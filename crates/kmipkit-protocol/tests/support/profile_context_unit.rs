use super::{
    PROFILE_CONTEXT_INDEX_COMPARISONS, ProfileBatchItemIndex, ProfileObjectState,
    ProfileValidationContext,
};

#[test]
fn profile_state_context_lookup_work_is_bounded_for_large_contexts() {
    const ENTRY_COUNT: usize = 512;
    let mut context = ProfileValidationContext::new();
    for index in 0..ENTRY_COUNT {
        context = context.with_object_state(index, ProfileObjectState::new(index.to_string()));
    }

    PROFILE_CONTEXT_INDEX_COMPARISONS.with(|steps| steps.set(0));
    for index in (0..ENTRY_COUNT).rev() {
        let identifier = index.to_string();
        assert!(context.state_for(index, &identifier).is_ok());
    }
    let lookup_steps = PROFILE_CONTEXT_INDEX_COMPARISONS.with(std::cell::Cell::get);

    assert!(
        lookup_steps > 0,
        "index comparison accounting must be active"
    );
    assert!(
        lookup_steps <= ENTRY_COUNT * 32,
        "profile context lookup used {lookup_steps} index comparisons for {ENTRY_COUNT} states"
    );
}

#[test]
fn profile_batch_item_index_implements_equality_and_partial_order() {
    let first = ProfileBatchItemIndex(3);
    let equal = ProfileBatchItemIndex(3);
    let later = ProfileBatchItemIndex(8);
    PROFILE_CONTEXT_INDEX_COMPARISONS.with(|count| count.set(0));

    assert_eq!(first, equal);
    assert_eq!(first.partial_cmp(&later), Some(std::cmp::Ordering::Less));
    assert_eq!(later.partial_cmp(&first), Some(std::cmp::Ordering::Greater));
    assert_ne!(first, later);
    assert_eq!(
        PROFILE_CONTEXT_INDEX_COMPARISONS.with(std::cell::Cell::get),
        4,
        "equality and partial ordering should both use the counted comparison paths"
    );
}
