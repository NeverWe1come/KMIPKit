//! Red-only tests for the private execute-boundary source checker.
//!
//! Fixture files are inert Rust source snippets loaded as text. They are not
//! compiled, executed, or read from OASIS sources. T011 replaces these
//! deliberately incomplete test-only checker candidates with the approved
//! pinned Rust-AST audit; T017 wires that production audit into CI.
//! The direct caller-owned transport fixture is an ADR-0014 positive control;
//! raw request access through the typed client or facade remains forbidden.
//!
//! Traceability: `KMIPKIT-0007-FR-001`, `-FR-003`, `-FR-011`, `-FR-012`,
//! `-FR-015`, `-FR-017`; `KMIPKIT-0007-SC-004`, `-SC-007`, `-SC-008`;
//! ADR-0012, ADR-0014, OD-005.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceCoverage {
    CandidateInspected,
    Uninspected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ExpectedDecision {
    Accept,
    Reject,
}

struct Fixture {
    id: &'static str,
    path: &'static str,
    source: &'static str,
    probe: &'static str,
    coverage: SourceCoverage,
    expected: ExpectedDecision,
}

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "valid_execute",
        path: "tests/fixtures/execute_boundary/valid_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/valid_execute.rs"),
        probe: "OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "generic_item_input",
        path: "tests/fixtures/execute_boundary/generic_item_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/generic_item_input.rs"),
        probe: "kmipkit_ttlv::Item",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_body_input",
        path: "tests/fixtures/execute_boundary/raw_body_input.rs",
        source: include_str!("../tests/fixtures/execute_boundary/raw_body_input.rs"),
        probe: "body: &[u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_writer",
        path: "tests/fixtures/execute_boundary/public_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/public_writer.rs"),
        probe: "pub fn encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_exchange_outside_execute",
        path: "tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs"),
        probe: "transport.exchange(request, max_response_bytes)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "client_raw_body_execute",
        path: "tests/fixtures/execute_boundary/client_raw_body_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/client_raw_body_execute.rs"),
        probe: "self.transport.exchange(caller_body)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_client_transport_injection",
        path: "tests/fixtures/execute_boundary/public_client_transport_injection.rs",
        source: include_str!(
            "../tests/fixtures/execute_boundary/public_client_transport_injection.rs"
        ),
        probe: "pub fn with_transport<T: Transport>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "facade_transport_reexport",
        path: "tests/fixtures/execute_boundary/facade_transport_reexport.rs",
        source: include_str!("../tests/fixtures/execute_boundary/facade_transport_reexport.rs"),
        probe: "pub use kmipkit_transport::Transport",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "missing_permit",
        path: "tests/fixtures/execute_boundary/missing_permit.rs",
        source: include_str!("../tests/fixtures/execute_boundary/missing_permit.rs"),
        probe: "self.writer.encode(request)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_permit",
        path: "tests/fixtures/execute_boundary/duplicate_permit.rs",
        source: include_str!("../tests/fixtures/execute_boundary/duplicate_permit.rs"),
        probe: "let second = OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_outside_execute",
        path: "tests/fixtures/execute_boundary/permit_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/permit_outside_execute.rs"),
        probe: "fn prepare_permit()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_writer",
        path: "tests/fixtures/execute_boundary/duplicate_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/duplicate_writer.rs"),
        probe: "self.writer.encode(request, permit);\n        self.writer.encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_outside_execute",
        path: "tests/fixtures/execute_boundary/writer_outside_execute.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_outside_execute.rs"),
        probe: "fn submit",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_alias",
        path: "tests/fixtures/execute_boundary/writer_alias.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_alias.rs"),
        probe: "as renamed_encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_reexport",
        path: "tests/fixtures/execute_boundary/writer_reexport.rs",
        source: include_str!("../tests/fixtures/execute_boundary/writer_reexport.rs"),
        probe: "pub use crate::private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "macro_token_tree",
        path: "tests/fixtures/execute_boundary/macro_token_tree.rs",
        source: include_str!("../tests/fixtures/execute_boundary/macro_token_tree.rs"),
        probe: "macro_rules! hidden_writer_call",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "include_bypass",
        path: "tests/fixtures/execute_boundary/include_bypass.rs",
        source: include_str!("../tests/fixtures/execute_boundary/include_bypass.rs"),
        probe: "include!(\"included_writer.rs\")",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "included_writer_source",
        path: "tests/fixtures/execute_boundary/included_writer.rs",
        source: include_str!("../tests/fixtures/execute_boundary/included_writer.rs"),
        probe: "private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "generated_source",
        path: "tests/fixtures/execute_boundary/generated_source.rs",
        source: include_str!("../tests/fixtures/execute_boundary/generated_source.rs"),
        probe: "OUT_DIR",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "conditional_compilation",
        path: "tests/fixtures/execute_boundary/conditional_compilation.rs",
        source: include_str!("../tests/fixtures/execute_boundary/conditional_compilation.rs"),
        probe: "#[cfg(any(",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_request_time_stamp",
        path: "tests/fixtures/execute_boundary/log_request_time_stamp.rs",
        source: include_str!("../tests/fixtures/execute_boundary/log_request_time_stamp.rs"),
        probe: "request_time_stamp = ?request_time_stamp",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_asynchronous_correlation_value",
        path: "tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs",
        source: include_str!(
            "../tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs"
        ),
        probe: "asynchronous_correlation_value = ?value",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "uninspected_construct",
        path: "tests/fixtures/execute_boundary/uninspected_construct.rs",
        source: include_str!("../tests/fixtures/execute_boundary/uninspected_construct.rs"),
        probe: "opaque_compiler_construct!",
        coverage: SourceCoverage::Uninspected,
        expected: ExpectedDecision::Reject,
    },
];

const EXPECTED_FIXTURE_IDS: &[&str] = &[
    "valid_execute",
    "generic_item_input",
    "raw_body_input",
    "public_writer",
    "raw_exchange_outside_execute",
    "client_raw_body_execute",
    "public_client_transport_injection",
    "facade_transport_reexport",
    "missing_permit",
    "duplicate_permit",
    "permit_outside_execute",
    "duplicate_writer",
    "writer_outside_execute",
    "writer_alias",
    "writer_reexport",
    "macro_token_tree",
    "include_bypass",
    "included_writer_source",
    "generated_source",
    "conditional_compilation",
    "log_request_time_stamp",
    "log_asynchronous_correlation_value",
    "uninspected_construct",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum CandidateRejection {
    BoundaryViolation,
    UninspectedSource,
}

/// Deliberately incorrect Red candidate: it accepts all fixture source without
/// checking writer/permit confinement, forbidden values, or syntax coverage.
fn candidate_check_fixture(_fixture: &Fixture) -> Result<(), CandidateRejection> {
    Ok(())
}

/// Deliberately incorrect Red candidate: it does not reject an uninspected
/// construct in the source inventory.
fn candidate_check_inventory(_inventory: &[&Fixture]) -> Result<(), CandidateRejection> {
    Ok(())
}

fn candidate_rejects_boundary_fixture(fixture: &Fixture) -> bool {
    candidate_check_fixture(fixture) == Err(CandidateRejection::BoundaryViolation)
}

fn fixture(id: &str) -> &'static Fixture {
    FIXTURES
        .iter()
        .find(|fixture| fixture.id == id)
        .expect("fixture id is part of the explicit source inventory")
}

fn accepted_ids_for_rejected_fixtures(ids: &[&str]) -> Vec<&'static str> {
    ids.iter()
        .filter_map(|id| {
            let fixture = fixture(id);
            (!candidate_rejects_boundary_fixture(fixture)).then_some(fixture.id)
        })
        .collect()
}

#[test]
fn generic_item_and_raw_body_inputs_are_rejected() {
    let accepted = accepted_ids_for_rejected_fixtures(&["generic_item_input", "raw_body_input"]);

    assert!(
        accepted.is_empty(),
        "accepted forbidden fixtures: {accepted:?}"
    );
}

#[test]
fn low_level_exception_does_not_bypass_the_typed_client_boundary() {
    assert_eq!(
        candidate_check_fixture(fixture("raw_exchange_outside_execute")),
        Ok(()),
        "ADR-0014 allows direct low-level exchange with caller-owned bytes"
    );
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "public_writer",
        "client_raw_body_execute",
        "public_client_transport_injection",
        "facade_transport_reexport",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted forbidden fixtures: {accepted:?}"
    );
}

#[test]
fn exactly_one_permit_and_writer_call_are_inside_execute() {
    assert_eq!(
        candidate_check_fixture(fixture("valid_execute")),
        Ok(()),
        "one permit mint and one writer call inside Client::execute is allowed"
    );
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "missing_permit",
        "duplicate_permit",
        "permit_outside_execute",
        "duplicate_writer",
        "writer_outside_execute",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted boundary violations: {accepted:?}"
    );
}

#[test]
fn aliases_macros_includes_generated_and_cfg_sources_cannot_bypass_the_audit() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "writer_alias",
        "writer_reexport",
        "macro_token_tree",
        "include_bypass",
        "included_writer_source",
        "generated_source",
        "conditional_compilation",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted bypass fixtures: {accepted:?}"
    );
}

#[test]
fn production_logs_cannot_format_request_time_stamp_or_correlation_value() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "log_request_time_stamp",
        "log_asynchronous_correlation_value",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted logging fixtures: {accepted:?}"
    );
}

#[test]
fn source_inventory_fails_closed_for_uninspected_constructs() {
    let uninspected = fixture("uninspected_construct");
    assert_eq!(uninspected.coverage, SourceCoverage::Uninspected);
    assert_eq!(uninspected.expected, ExpectedDecision::Reject);

    assert_eq!(
        candidate_check_inventory(&[uninspected]),
        Err(CandidateRejection::UninspectedSource),
        "the audit must fail closed when source syntax is not inspected"
    );
}

#[test]
fn fixture_inventory_is_explicit_nonempty_and_confined_to_client_tests() {
    let actual_ids = FIXTURES
        .iter()
        .map(|fixture| fixture.id)
        .collect::<Vec<_>>();
    assert_eq!(actual_ids, EXPECTED_FIXTURE_IDS);

    for fixture in FIXTURES {
        assert!(
            !fixture.source.trim().is_empty(),
            "empty fixture: {}",
            fixture.id
        );
        assert!(
            fixture.source.contains(fixture.probe),
            "fixture {} is missing its syntax probe",
            fixture.id
        );
        assert!(
            fixture.path.starts_with("tests/fixtures/execute_boundary/"),
            "fixture {} escapes the crate-local inventory",
            fixture.id
        );
        assert!(!fixture.path.contains("specification/oasis"));
        match fixture.expected {
            ExpectedDecision::Accept => assert!(matches!(
                fixture.id,
                "valid_execute" | "raw_exchange_outside_execute"
            )),
            ExpectedDecision::Reject => assert!(!matches!(
                fixture.id,
                "valid_execute" | "raw_exchange_outside_execute"
            )),
        }
    }
}
