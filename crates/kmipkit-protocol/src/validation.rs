#[cfg(test)]
mod tests {
    use super::*;

    fn issue(severity: ValidationSeverity) -> ValidationIssue {
        ValidationIssue::new(
            ValidationCode::new("KMIPKIT-0051-R001"),
            ValidationPath::root(),
            severity,
            ValidationSourceRef::oasis("KMIP v2.1", "§6.1.36, Table 271"),
        )
    }

    #[test]
    fn report_is_invalid_when_an_error_issue_is_present() {
        let report = ValidationReport::from_issues(vec![issue(ValidationSeverity::Error)]);

        assert!(!report.is_valid());
        assert_eq!(report.issues().len(), 1);
    }

    #[test]
    fn report_remains_valid_when_it_contains_only_warnings() {
        let report = ValidationReport::from_issues(vec![issue(ValidationSeverity::Warning)]);

        assert!(report.is_valid());
    }

    #[test]
    fn issue_debug_contains_only_safe_metadata() {
        let issue = issue(ValidationSeverity::Error);
        let formatted = format!("{issue:?}");

        assert!(formatted.contains("KMIPKIT-0051-R001"));
        assert!(formatted.contains("6.1.36"));
        assert!(!formatted.contains("secret"));
        assert!(!formatted.contains("bytes"));
    }
}
