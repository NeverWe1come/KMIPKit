use super::{
    ValidationCode, ValidationIssue, ValidationPath, ValidationSeverity, ValidationSourceRef,
};

#[test]
fn issue_exposes_source_identifier_and_reference_separately() {
    let issue = ValidationIssue::new(
        ValidationCode::new("KMIPKIT_TEST"),
        ValidationPath::root(),
        ValidationSeverity::Error,
        ValidationSourceRef::Oasis {
            document: "KMIP 2.1",
            reference: "§ 9.1",
        },
    );

    assert_eq!("KMIP 2.1", issue.source_identifier());
    assert_eq!("§ 9.1", issue.source_reference());
}

#[test]
fn project_rule_has_an_empty_source_reference() {
    let issue = ValidationIssue::new(
        ValidationCode::new("KMIPKIT_TEST"),
        ValidationPath::root(),
        ValidationSeverity::Error,
        ValidationSourceRef::ProjectRule("KMIPKit policy"),
    );

    assert_eq!("KMIPKit policy", issue.source_identifier());
    assert_eq!("", issue.source_reference());
}
