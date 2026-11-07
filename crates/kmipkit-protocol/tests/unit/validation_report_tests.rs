use crate::{
    ValidationCode, ValidationIssue, ValidationPath, ValidationReport, ValidationSeverity,
    ValidationSourceRef,
};

fn issue(severity: ValidationSeverity) -> ValidationIssue {
    ValidationIssue::new(
        ValidationCode::new("KMIPKIT-0051-R001"),
        ValidationPath::root().field("Request Payload").index(0),
        severity,
        ValidationSourceRef::oasis("KMIP v2.1", "§6.1.36, Table 271"),
    )
}

fn report(severity: ValidationSeverity) -> ValidationReport {
    ValidationReport::from_issues(vec![issue(severity)])
}

#[test]
fn report_is_invalid_for_errors_but_valid_for_warnings() {
    assert!(!report(ValidationSeverity::Error).is_valid());
    assert!(report(ValidationSeverity::Warning).is_valid());
}

#[test]
fn issue_exposes_stable_code_path_and_source_metadata() {
    let report = report(ValidationSeverity::Error);
    let issue = report.issues().first().expect("one issue was added");

    assert_eq!(issue.code().as_str(), "KMIPKIT-0051-R001");
    assert_eq!(issue.path().to_string(), "Request Payload[0]");
    assert_eq!(issue.source().to_string(), "KMIP v2.1 §6.1.36, Table 271");
}

#[test]
fn report_debug_is_deterministic_and_contains_only_issue_metadata() {
    let first = report(ValidationSeverity::Error);
    let second = report(ValidationSeverity::Error);
    let first_debug = format!("{first:?}");

    assert_eq!(first_debug, format!("{second:?}"));
    assert!(first_debug.contains("KMIPKIT-0051-R001"));
    assert!(first_debug.contains("Request Payload[0]"));
    assert!(first_debug.contains("6.1.36"));
    assert!(!first_debug.contains("example-secret"));
    assert!(!first_debug.contains("raw-ttlv"));
}
