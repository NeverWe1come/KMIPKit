//! Compile-fail contracts for the sealed vendor-extension request boundary.
//!
//! Traceability: KMIPKIT-0012-FR-004, FR-005, and FR-011.

#[test]
fn unvalidated_items_raw_bodies_caller_conversions_and_plugins_cannot_enter_typed_requests() {
    trybuild::TestCases::new().compile_fail("tests/ui/extension_boundary/*.rs");
}
