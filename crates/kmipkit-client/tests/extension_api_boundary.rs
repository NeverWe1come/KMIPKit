//! Compile-fail contracts for the sealed vendor-extension request boundary.
//!
//! Verifies protocol model, request/response validation, and security invariants.

#[test]
fn unvalidated_items_raw_bodies_caller_conversions_and_plugins_cannot_enter_typed_requests() {
    trybuild::TestCases::new().compile_fail("tests/ui/extension_boundary/*.rs");
}
