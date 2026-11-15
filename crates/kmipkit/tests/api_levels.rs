use kmipkit::{api_levels, capability_is_available};
use serde_json::Value;

#[test]
fn api_levels_match_the_shared_fixture_without_network_calls() {
    let fixture: Value = serde_json::from_str(include_str!(
        "../../../tests/fixtures/api/shared-capability-parity.json"
    ))
    .expect("the shared API fixture is valid JSON");
    let case = fixture["cases"]
        .as_array()
        .expect("fixture cases are an array")
        .iter()
        .find(|case| case["id"] == "api-levels.shared.v1")
        .expect("the API-level fixture case exists");
    let expected = case["expectedOutput"]["apiLevels"]
        .as_array()
        .expect("fixture API levels are an array")
        .iter()
        .map(|level| level.as_str().expect("API level IDs are strings"))
        .collect::<Vec<_>>();

    assert_eq!(api_levels(), expected);
    assert_eq!(capability_is_available("api_level_discovery"), Ok(true));
    assert_eq!(case["expectedOutput"]["networkCalls"], 0);
    assert_eq!(
        case["expectedOutput"]["distinctFromKmipOperation"],
        "Discover Versions"
    );
    assert_eq!(
        case["expectedOutput"]["operationMappingCompleteness"],
        "not-asserted"
    );
}
