//! Compile-fail boundary for the explicit usage-allocation context.

#[test]
fn usage_allocation_context_cannot_be_cloned() {
    trybuild::TestCases::new().compile_fail("tests/ui/usage_allocation_guard_not_clone.rs");
}
