#![cfg(not(kmipkit_pkcs11_candidate_test))]

#[test]
fn pkcs11_candidate_types_are_not_available_in_default_builds() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/default_pkcs11_candidate_api.rs");
}
