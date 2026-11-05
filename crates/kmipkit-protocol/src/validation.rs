//! Safe, deterministic validation diagnostics for KMIP structures and requests.

use std::fmt;

mod nested;
mod schema;
mod typed_requests;

pub use schema::{
    NestedStructureSchema, RequestPayloadSchema, ValidationSchema, validate_structure,
};
pub use typed_requests::ValidateKmipRequest;

pub(crate) fn validate_nested_structure_with_prefix(
    structure: &kmipkit_ttlv::StructureView<'_>,
    schema: NestedStructureSchema,
    prefix: &ValidationPath,
) -> ValidationReport {
    nested::validate_nested_with_prefix(structure, schema, prefix)
}

/// Severity of one locally detected validation finding.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationSeverity {
    /// A violation that makes the validated request or structure invalid.
    Error,
    /// A finding that does not establish invalidity and does not block execution.
    Warning,
    /// Informational validation metadata that does not block execution.
    Info,
}

/// Stable identifier for one validation rule or diagnostic condition.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ValidationCode(&'static str);

impl ValidationCode {
    pub(crate) const fn new(value: &'static str) -> Self {
        Self(value)
    }

    /// Returns the stable code without any request-derived data.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

/// One safe segment in a structure-relative validation path.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationPathSegment {
    /// A field name taken from the fixed schema descriptor.
    Field(&'static str),
    /// The zero-based index of a repeated child.
    Index(usize),
    /// A numeric TTLV tag, formatted as metadata rather than a field value.
    Tag(u32),
}

/// A structure-relative path containing schema names, tag identifiers, and list indices only.
#[derive(Clone, Default, Eq, PartialEq)]
pub struct ValidationPath(Vec<ValidationPathSegment>);

impl ValidationPath {
    /// Creates an empty path at the root of a validated structure.
    #[must_use]
    pub const fn root() -> Self {
        Self(Vec::new())
    }

    /// Appends a fixed schema field name.
    #[must_use]
    pub fn field(mut self, name: &'static str) -> Self {
        self.0.push(ValidationPathSegment::Field(name));
        self
    }

    /// Appends the zero-based index of a repeated field.
    #[must_use]
    pub fn index(mut self, index: usize) -> Self {
        self.0.push(ValidationPathSegment::Index(index));
        self
    }

    /// Appends a numeric TTLV tag without attaching its value.
    #[must_use]
    pub fn tag(mut self, tag: u32) -> Self {
        self.0.push(ValidationPathSegment::Tag(tag));
        self
    }

    /// Returns the path segments without exposing any associated field value.
    #[must_use]
    pub fn segments(&self) -> &[ValidationPathSegment] {
        &self.0
    }
}

impl fmt::Display for ValidationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut has_segment = false;
        for segment in &self.0 {
            match segment {
                ValidationPathSegment::Field(name) => {
                    write_child_separator(formatter, has_segment)?;
                    formatter.write_str(name)?;
                }
                ValidationPathSegment::Index(item_index) => {
                    write!(formatter, "[{item_index}]")?;
                }
                ValidationPathSegment::Tag(tag) => {
                    write_child_separator(formatter, has_segment)?;
                    write!(formatter, "tag(0x{tag:06x})")?;
                }
            }
            has_segment = true;
        }
        Ok(())
    }
}

fn write_child_separator(formatter: &mut fmt::Formatter<'_>, has_parent: bool) -> fmt::Result {
    if has_parent {
        formatter.write_str(".")
    } else {
        Ok(())
    }
}

impl fmt::Debug for ValidationPath {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("ValidationPath")
            .field(&self.to_string())
            .finish()
    }
}

/// Identifies the normative OASIS source or a `KMIPKit` project decision for an issue.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValidationSourceRef {
    /// A pinned OASIS document and exact section/table reference.
    Oasis {
        /// The immutable source document designation.
        document: &'static str,
        /// The exact section and optional table citation.
        reference: &'static str,
    },
    /// A `KMIPKit` decision not prescribed by OASIS.
    ProjectRule(&'static str),
}

impl ValidationSourceRef {
    #[cfg(test)]
    pub(crate) const fn oasis(document: &'static str, reference: &'static str) -> Self {
        Self::Oasis {
            document,
            reference,
        }
    }

    /// Returns the stable document/decision identifier as safe metadata.
    #[must_use]
    pub const fn identifier(self) -> &'static str {
        match self {
            Self::Oasis { document, .. } => document,
            Self::ProjectRule(identifier) => identifier,
        }
    }
}

impl fmt::Display for ValidationSourceRef {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Oasis {
                document,
                reference,
            } => write!(formatter, "{document} {reference}"),
            Self::ProjectRule(identifier) => formatter.write_str(identifier),
        }
    }
}

/// One redacted validation finding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationIssue {
    code: ValidationCode,
    path: ValidationPath,
    severity: ValidationSeverity,
    source: ValidationSourceRef,
}

impl ValidationIssue {
    pub(crate) const fn new(
        code: ValidationCode,
        path: ValidationPath,
        severity: ValidationSeverity,
        source: ValidationSourceRef,
    ) -> Self {
        Self {
            code,
            path,
            severity,
            source,
        }
    }

    /// Returns the stable rule or diagnostic code.
    #[must_use]
    pub const fn code(&self) -> ValidationCode {
        self.code
    }

    /// Returns a schema-relative path made only from safe metadata.
    #[must_use]
    pub const fn path(&self) -> &ValidationPath {
        &self.path
    }

    /// Returns the severity of this finding.
    #[must_use]
    pub const fn severity(&self) -> ValidationSeverity {
        self.severity
    }

    /// Returns the exact normative or project-policy source reference.
    #[must_use]
    pub const fn source(&self) -> ValidationSourceRef {
        self.source
    }
}

/// Deterministic findings from validating one request or structure.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ValidationReport {
    issues: Vec<ValidationIssue>,
}

impl ValidationReport {
    #[cfg(test)]
    pub(crate) fn from_issues(issues: Vec<ValidationIssue>) -> Self {
        Self { issues }
    }

    pub(crate) fn push(&mut self, issue: ValidationIssue) {
        self.issues.push(issue);
    }

    pub(crate) fn append(&mut self, report: Self) {
        self.issues.extend(report.issues);
    }

    /// Returns the ordered findings; each issue contains only safe metadata.
    #[must_use]
    pub fn issues(&self) -> &[ValidationIssue] {
        &self.issues
    }

    /// Returns whether the checked request or structure has no error-severity findings.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        self.issues
            .iter()
            .all(|issue| issue.severity != ValidationSeverity::Error)
    }
}

pub(crate) fn request_issue_report(
    code: &'static str,
    field_path: &'static str,
    source: ValidationSourceRef,
) -> ValidationReport {
    issue_report(
        code,
        ValidationPath::root()
            .field("Request Payload")
            .field(field_path),
        source,
    )
}

pub(crate) fn issue_report(
    code: &'static str,
    path: ValidationPath,
    source: ValidationSourceRef,
) -> ValidationReport {
    let mut report = ValidationReport::default();
    report.push(ValidationIssue::new(
        ValidationCode::new(code),
        path,
        ValidationSeverity::Error,
        source,
    ));
    report
}
