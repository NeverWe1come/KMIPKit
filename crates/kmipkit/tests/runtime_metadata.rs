#[test]
fn public_version_matches_the_synchronized_package_version() {
    assert_eq!(kmipkit::VERSION, env!("CARGO_PKG_VERSION"));
    assert_eq!(kmipkit::VERSION, "1.0.0");
}
