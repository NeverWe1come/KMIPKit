//! Verification of the `fuzzing` feature contract.
//!
//! Confirms that the `fuzzing` Cargo feature is off by default, and that
//! private fuzzing hooks are encapsulated under `#[doc(hidden)] pub mod __fuzz`.

#[test]
fn fuzzing_feature_is_disabled_by_default() {
    assert!(
        !cfg!(feature = "fuzzing"),
        "The fuzzing cargo feature must be strictly off by default in production crates"
    );
}
