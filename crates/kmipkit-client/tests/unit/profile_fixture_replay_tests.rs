#[test]
fn every_available_profile_fixture_runs_through_typed_client_execution() {
    let results = run_pinned_profile_fixtures();
    assert_eq!(
        results.len(),
        16,
        "all pinned profile case IDs have results"
    );
    assert!(results.iter().all(|result| result.ends_with("=PASS")));
}

fn run_pinned_profile_fixtures() -> Vec<String> {
    Vec::new()
}
