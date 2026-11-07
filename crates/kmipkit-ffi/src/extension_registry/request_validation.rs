//! Safe C ABI access to the shared Rust request and TTLV validator.
use super::{
    ERROR_INVALID_INPUT, ERROR_RESOURCE_LIMIT, FfiResult, HandleValue, Kind, NestedStructureSchema,
    RequestPayloadSchema, SUCCESS, ValidationReport, ValidationReportSnapshot, ValidationSchema,
    copy_required_structure_handle, ffi_entrypoint, input_span, kmipkit_byte_span_v1_t,
    kmipkit_client_batch_item_t, kmipkit_ttlv_structure_t, kmipkit_validation_issue_view_v1_t,
    kmipkit_validation_report_t, record_ffi_error, reference_handle, release, size_of,
    try_make_handle, validate_structure,
};
use std::ptr;

const MAX_SCHEMA_ID_BYTES: u64 = 128;

fn request_schema(name: &str) -> Option<RequestPayloadSchema> {
    Some(match name {
        "Activate" => RequestPayloadSchema::Activate,
        "AddAttribute" => RequestPayloadSchema::AddAttribute,
        "AdjustAttribute" => RequestPayloadSchema::AdjustAttribute,
        "Archive" => RequestPayloadSchema::Archive,
        "Cancel" => RequestPayloadSchema::Cancel,
        "Certify" => RequestPayloadSchema::Certify,
        "Check" => RequestPayloadSchema::Check,
        "Create" => RequestPayloadSchema::Create,
        "CreateKeyPair" => RequestPayloadSchema::CreateKeyPair,
        "CreateSplitKey" => RequestPayloadSchema::CreateSplitKey,
        "Decrypt" => RequestPayloadSchema::Decrypt,
        "DelegatedLogin" => RequestPayloadSchema::DelegatedLogin,
        "DeleteAttribute" => RequestPayloadSchema::DeleteAttribute,
        "DeriveKey" => RequestPayloadSchema::DeriveKey,
        "Destroy" => RequestPayloadSchema::Destroy,
        "DiscoverVersions" => RequestPayloadSchema::DiscoverVersions,
        "Encrypt" => RequestPayloadSchema::Encrypt,
        "Export" => RequestPayloadSchema::Export,
        "Get" => RequestPayloadSchema::Get,
        "GetAttributeList" => RequestPayloadSchema::GetAttributeList,
        "GetAttributes" => RequestPayloadSchema::GetAttributes,
        "GetConstraints" => RequestPayloadSchema::GetConstraints,
        "GetUsageAllocation" => RequestPayloadSchema::GetUsageAllocation,
        "Hash" => RequestPayloadSchema::Hash,
        "Import" => RequestPayloadSchema::Import,
        "Interop" => RequestPayloadSchema::Interop,
        "JoinSplitKey" => RequestPayloadSchema::JoinSplitKey,
        "Locate" => RequestPayloadSchema::Locate,
        "Log" => RequestPayloadSchema::Log,
        "Login" => RequestPayloadSchema::Login,
        "Logout" => RequestPayloadSchema::Logout,
        "Mac" => RequestPayloadSchema::Mac,
        "MacVerify" => RequestPayloadSchema::MacVerify,
        "ModifyAttribute" => RequestPayloadSchema::ModifyAttribute,
        "ObtainLease" => RequestPayloadSchema::ObtainLease,
        "Ping" => RequestPayloadSchema::Ping,
        "Pkcs11" => RequestPayloadSchema::Pkcs11,
        "Poll" => RequestPayloadSchema::Poll,
        "Process" => RequestPayloadSchema::Process,
        "Query" => RequestPayloadSchema::Query,
        "QueryAsyncRequests" => RequestPayloadSchema::QueryAsyncRequests,
        "ReProvision" => RequestPayloadSchema::ReProvision,
        "Recertify" => RequestPayloadSchema::Recertify,
        "Recover" => RequestPayloadSchema::Recover,
        "Register" => RequestPayloadSchema::Register,
        "Rekey" => RequestPayloadSchema::Rekey,
        "RekeyKeyPair" => RequestPayloadSchema::RekeyKeyPair,
        "Revoke" => RequestPayloadSchema::Revoke,
        "RngRetrieve" => RequestPayloadSchema::RngRetrieve,
        "RngSeed" => RequestPayloadSchema::RngSeed,
        "SetAttribute" => RequestPayloadSchema::SetAttribute,
        "SetConstraints" => RequestPayloadSchema::SetConstraints,
        "SetDefaults" => RequestPayloadSchema::SetDefaults,
        "SetEndpointRole" => RequestPayloadSchema::SetEndpointRole,
        "Sign" => RequestPayloadSchema::Sign,
        "SignatureVerify" => RequestPayloadSchema::SignatureVerify,
        "Validate" => RequestPayloadSchema::Validate,
        _ => return None,
    })
}

fn nested_schema(name: &str) -> Option<NestedStructureSchema> {
    Some(match name {
        "Constraints" => NestedStructureSchema::Constraints,
        "Constraint" => NestedStructureSchema::Constraint,
        "ObjectTypes" => NestedStructureSchema::ObjectTypes,
        "ObjectGroups" => NestedStructureSchema::ObjectGroups,
        "Attributes" => NestedStructureSchema::Attributes,
        "DefaultsInformation" => NestedStructureSchema::DefaultsInformation,
        "ObjectDefaults" => NestedStructureSchema::ObjectDefaults,
        "CryptographicParameters" => NestedStructureSchema::CryptographicParameters,
        "DerivationParameters" => NestedStructureSchema::DerivationParameters,
        "KeyWrappingSpecification" => NestedStructureSchema::KeyWrappingSpecification,
        "EncryptionKeyInformation" => NestedStructureSchema::EncryptionKeyInformation,
        "MacSignatureKeyInformation" => NestedStructureSchema::MacSignatureKeyInformation,
        "Rights" => NestedStructureSchema::Rights,
        "Right" => NestedStructureSchema::Right,
        "UsageLimits" => NestedStructureSchema::UsageLimits,
        "Objects" => NestedStructureSchema::Objects,
        "Operations" => NestedStructureSchema::Operations,
        "CurrentAttribute" => NestedStructureSchema::CurrentAttribute,
        "NewAttribute" => NestedStructureSchema::NewAttribute,
        "Certificate" => NestedStructureSchema::Certificate,
        "CertificateRequest" => NestedStructureSchema::CertificateRequest,
        "AsynchronousCorrelationValues" => NestedStructureSchema::AsynchronousCorrelationValues,
        "Ticket" => NestedStructureSchema::Ticket,
        "ProtocolVersion" => NestedStructureSchema::ProtocolVersion,
        "RevocationReason" => NestedStructureSchema::RevocationReason,
        "Pkcs11InputParameters" => NestedStructureSchema::Pkcs11InputParameters,
        "Pkcs11Interface" => NestedStructureSchema::Pkcs11Interface,
        "KeyBlock" => NestedStructureSchema::KeyBlock,
        "KeyValue" => NestedStructureSchema::KeyValue,
        "KeyWrappingData" => NestedStructureSchema::KeyWrappingData,
        "OpaqueObject" => NestedStructureSchema::OpaqueObject,
        _ => return None,
    })
}

fn parse_schema(bytes: &[u8]) -> FfiResult<Option<ValidationSchema>> {
    if bytes.is_empty() {
        return Ok(None);
    }
    let value = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
    if value == "Unsupported" {
        return Ok(Some(ValidationSchema::Unsupported));
    }
    if let Some(name) = value.strip_prefix("RequestPayload/") {
        return request_schema(name)
            .map(|schema| Some(ValidationSchema::RequestPayload(schema)))
            .ok_or(ERROR_INVALID_INPUT);
    }
    if let Some(name) = value.strip_prefix("Nested/") {
        return nested_schema(name)
            .map(|schema| Some(ValidationSchema::Nested(schema)))
            .ok_or(ERROR_INVALID_INPUT);
    }
    Err(ERROR_INVALID_INPUT)
}

fn span(bytes: &[u8]) -> kmipkit_byte_span_v1_t {
    kmipkit_byte_span_v1_t {
        data: if bytes.is_empty() {
            ptr::null()
        } else {
            bytes.as_ptr()
        },
        length: bytes.len() as u64,
    }
}

fn report_handle(report: &ValidationReport) -> FfiResult<*mut kmipkit_validation_report_t> {
    try_make_handle(
        Kind::ValidationReport,
        HandleValue::ValidationReport(ValidationReportSnapshot::from_report(report)),
    )
}

/// Creates an owned report by validating a generic TTLV Structure against an explicit schema.
/// An empty schema span requests generic TTLV validation and never guesses an operation schema.
/// Schema identifiers use `RequestPayload/<Operation>` or `Nested/<Structure>`; `Unsupported`
/// asks the shared Rust validator to report its standard unsupported-schema finding.
///
/// # Safety
///
/// `structure` must be null or a live `KMIPKit` Structure handle. For a non-zero schema length,
/// `schema` must point to that many readable UTF-8 bytes. `out_report` must point to an aligned,
/// writable handle slot that does not overlap either input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_ttlv_structure_validation_report_v1(
    structure: *mut kmipkit_ttlv_structure_t,
    schema: *const u8,
    schema_length: u64,
    out_report: *mut *mut kmipkit_validation_report_t,
) -> i32 {
    ffi_entrypoint(|| {
        let output = out_slot!(out_report);
        let schema_bytes = try_ffi!(input_span(schema, schema_length, MAX_SCHEMA_ID_BYTES));
        let schema = match parse_schema(schema_bytes) {
            Ok(schema) => schema,
            Err(status) => return record_ffi_error(status),
        };
        let structure = try_ffi!(copy_required_structure_handle(structure));
        let report = validate_structure(&structure.view(), schema);
        *output = try_ffi!(report_handle(&report));
        SUCCESS
    })
}

/// Creates an owned report from the shared Rust typed request validator for one batch item.
///
/// # Safety
///
/// `item` must be null or a live `KMIPKit` client batch item. `out_report` must point to an aligned,
/// writable handle slot that does not overlap the input.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_client_batch_item_validation_report(
    item: *mut kmipkit_client_batch_item_t,
    out_report: *mut *mut kmipkit_validation_report_t,
) -> i32 {
    ffi_entrypoint(|| {
        let output = out_slot!(out_report);
        let handle = match reference_handle(item, Kind::BatchItem) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status),
        };
        let HandleValue::BatchItem(item) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let report = item.request().validation_report();
        *output = try_ffi!(report_handle(&report));
        SUCCESS
    })
}

/// Returns whether the report has no error-severity issues.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validation_report_is_valid(
    report: *mut kmipkit_validation_report_t,
    out_is_valid: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let output = scalar_out!(out_is_valid);
        let handle = match reference_handle(report, Kind::ValidationReport) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status),
        };
        let HandleValue::ValidationReport(report) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *output = u8::from(report.is_valid);
        SUCCESS
    })
}

/// Returns the ordered issue count for an owned validation report.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validation_report_issue_count(
    report: *mut kmipkit_validation_report_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let output = scalar_out!(out_count);
        let handle = match reference_handle(report, Kind::ValidationReport) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status),
        };
        let HandleValue::ValidationReport(report) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *output = report.issues.len() as u64;
        SUCCESS
    })
}

/// Copies safe issue metadata into a caller-versioned record.
///
/// The byte spans borrow the immutable report and stay valid until it is released. They contain
/// only stable codes, schema-derived paths, severity and source references; no request values.
///
/// # Safety
///
/// `report` must be null or a live validation report. `out_issue` must be aligned, readable and
/// writable for one version 1 record whose `struct_size` and `abi_version` are initialized by the
/// caller; it must not overlap the report or its storage.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn kmipkit_validation_report_issue_at_v1(
    report: *mut kmipkit_validation_report_t,
    index: u64,
    out_issue: *mut kmipkit_validation_issue_view_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_issue.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: the caller promises an aligned readable/writable versioned output record.
        let output = unsafe { &mut *out_issue };
        if output.struct_size as usize != size_of::<kmipkit_validation_issue_view_v1_t>()
            || output.abi_version != 1
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let handle = match reference_handle(report, Kind::ValidationReport) {
            Ok(handle) => handle,
            Err(status) => return record_ffi_error(status),
        };
        let HandleValue::ValidationReport(report) = &handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(index) = usize::try_from(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(issue) = report.issues.get(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Ok(struct_size) = u32::try_from(size_of::<kmipkit_validation_issue_view_v1_t>()) else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        *output = kmipkit_validation_issue_view_v1_t {
            struct_size,
            abi_version: 1,
            severity: issue.severity,
            source_kind: issue.source_kind,
            code: span(&issue.code),
            path: span(&issue.path),
            source_identifier: span(&issue.source_identifier),
            source_reference: span(&issue.source_reference),
        };
        SUCCESS
    })
}

/// Releases a validation report. Null and already released handles are ignored.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validation_report_release(report: *mut kmipkit_validation_report_t) {
    ffi_entrypoint(|| {
        release(report, Kind::ValidationReport);
    });
}
