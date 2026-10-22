//! C ABI implementation for the manifest-backed extension registry API.
#![allow(clippy::missing_safety_doc, clippy::too_many_arguments)]

use aws_lc_rs::{hmac, rand};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::{collections::HashMap, collections::hash_map::Entry, mem::size_of};
use std::{fs, io::Read};

use crate::{
    kmipkit_client_batch_item_metadata_v1_t, kmipkit_client_batch_item_metadata_v2_t,
    kmipkit_client_batch_item_response_t, kmipkit_client_batch_item_t,
    kmipkit_client_batch_response_t, kmipkit_client_batch_t, kmipkit_client_configuration_t,
    kmipkit_client_extension_registry_t, kmipkit_client_request_message_extension_t,
    kmipkit_client_t, kmipkit_codec_limits_t, kmipkit_extension_child_rule_t,
    kmipkit_extension_compatibility_t, kmipkit_extension_definition_t,
    kmipkit_extension_discriminator_t, kmipkit_extension_identity_t,
    kmipkit_extension_information_t, kmipkit_extension_order_constraint_t,
    kmipkit_extension_recognition_t, kmipkit_extension_registry_limits_v1_t,
    kmipkit_extension_schema_t, kmipkit_raw_tag_t, kmipkit_register_request_t,
    kmipkit_registered_extension_value_t, kmipkit_request_options_v1_t,
    kmipkit_revocation_reason_t, kmipkit_revoke_request_t, kmipkit_split_key_reference_t,
    kmipkit_tag_t, kmipkit_timeout_limit_v1_t, kmipkit_transport_config_t,
    kmipkit_transport_config_v1_t, kmipkit_transport_input_v1_t, kmipkit_ttlv_item_t,
    kmipkit_ttlv_item_view_t, kmipkit_ttlv_path_t, kmipkit_ttlv_structure_t,
    kmipkit_ttlv_structure_view_t, kmipkit_ttlv_value_t, kmipkit_ttlv_value_view_t,
    kmipkit_unique_identifier_v1_t, kmipkit_usage_allocation_context_t,
    kmipkit_validated_extension_value_t,
};
use kmipkit_client::UsageAllocationContext;
use kmipkit_client::extension_registry as client_extension;
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchResponse, ClientCauseCategory, ClientError,
    ClientErrorCategory, ClientRequest,
};
use kmipkit_protocol::Pkcs11Request;
use kmipkit_protocol::extension;
use kmipkit_protocol::{
    AttributeSet, CertifyRequest, CheckRequest, CreateKeyPairRequest, CreateRequest,
    CreateSplitKeyRequest, DecryptRequest, DelegatedLoginRequest, DerivationMethod,
    DerivationParameters, DeriveKeyRequest, EncryptRequest, ExportRequest, GetConstraintsRequest,
    GetRequest, GetUsageAllocationRequest, HashRequest, ImportRequest, InteropRequest,
    JoinSplitKeyRequest, KeyCompressionType, KeyFormatType, KeyWrapType, LocateRequest, LogRequest,
    LoginRequest, LogoutRequest, MacRequest, MacVerifyRequest, ObjectGroupMember, ObjectType,
    ObtainLeaseRequest, OpaqueObjectContext, OperationData, ProtocolError, ProtocolErrorKind,
    QueryFunction, QueryRequest, ReProvisionRequest, RecertifyRequest, RegisterRequest,
    RekeyKeyPairRequest, RekeyRequest, RevocationReason, RevokeRequest, RngRetrieveRequest,
    RngSeedRequest, SecretBytes, SecretText, SetConstraintsRequest, SetDefaultsRequest,
    SignRequest, SignatureVerifyRequest, SplitKeyMethod, SplitKeyReference, StorageStatusMask,
    Ticket, UniqueIdentifier, ValidateRequest,
};
use kmipkit_transport::{
    CertificateInput, ClientIdentity, Endpoint, PrivateKeyInput, RequestDeliveryState,
    RequestOptions, RevocationListInput, TimeoutLimit, TimeoutPolicy, TransportCauseCategory,
    TransportConfig, TrustSource,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Item, ItemType, RawTag, Structure, Tag, Value, ValueView};
use std::time::Duration;

#[cfg(test)]
#[path = "../tests/support/cryptographic_operation_abi_tests.rs"]
mod cryptographic_operation_abi_tests;
#[cfg(test)]
#[path = "../tests/support/get_constraints_abi_tests.rs"]
mod get_constraints_abi_tests;
#[cfg(test)]
#[path = "../tests/support/get_usage_allocation_abi_tests.rs"]
mod get_usage_allocation_abi_tests;
#[cfg(test)]
#[path = "../tests/support/interop_abi_tests.rs"]
mod interop_abi_tests;
#[cfg(test)]
#[path = "../tests/support/log_abi_tests.rs"]
mod log_abi_tests;
#[cfg(test)]
#[path = "../tests/support/metadata_tests.rs"]
mod metadata_tests;
#[cfg(test)]
#[path = "../tests/support/opaque_handle_validation_tests.rs"]
mod opaque_handle_validation_tests;
#[cfg(test)]
#[path = "../tests/support/pkcs11_abi_tests.rs"]
mod pkcs11_abi_tests;
#[cfg(test)]
#[path = "../tests/support/registry_preflight_tests.rs"]
mod registry_preflight_tests;
#[cfg(test)]
#[path = "../tests/support/test_handle_helpers.rs"]
mod test_handle_helpers;
#[cfg(test)]
#[path = "../tests/support/validate_abi_tests.rs"]
mod validate_abi_tests;
#[cfg(test)]
#[path = "../tests/support/view_ownership_tests.rs"]
mod view_ownership_tests;
#[cfg(test)]
use self::test_handle_helpers::make_handle;

#[cfg(test)]
thread_local! {
    static REGISTRY_DEFINITION_CLONE_COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

const SUCCESS: i32 = 0;
const ERROR_COMPATIBILITY_MISMATCH: i32 = 1;
const ERROR_DUPLICATE_KEY: i32 = 2;
const ERROR_INVALID_IDENTITY: i32 = 3;
const ERROR_INVALID_INPUT: i32 = 4;
const ERROR_INVALID_SCHEMA: i32 = 5;
const ERROR_RESOURCE_LIMIT: i32 = 6;
const ERROR_INTERNAL: i32 = 7;
const ERROR_PROTOCOL: i32 = 8;
const ERROR_TRANSPORT: i32 = 9;
const ERROR_SERVER_RESULT: i32 = 10;
const ERROR_CATEGORY_NONE: u32 = 0;
const ERROR_CATEGORY_VALIDATION: u32 = 1;
const ERROR_CATEGORY_PROTOCOL: u32 = 2;
const ERROR_CATEGORY_TRANSPORT: u32 = 3;
const ERROR_CATEGORY_SERVER_RESULT: u32 = 4;
const ERROR_CATEGORY_OTHER: u32 = 5;
const ERROR_CAUSE_NONE: u32 = 0;
const ERROR_CAUSE_INVALID_INPUT: u32 = 1;
const ERROR_CAUSE_PROTOCOL_INVALID_VALUE: u32 = 2;
const ERROR_CAUSE_PROTOCOL_INVALID_ENCODING: u32 = 3;
const ERROR_CAUSE_PROTOCOL_OTHER: u32 = 4;
const ERROR_CAUSE_TRANSPORT_IO: u32 = 5;
const ERROR_CAUSE_TRANSPORT_TLS: u32 = 6;
const ERROR_CAUSE_TRANSPORT_HTTP: u32 = 7;
const ERROR_CAUSE_TRANSPORT_TIMEOUT: u32 = 8;
const ERROR_CAUSE_TRANSPORT_OTHER: u32 = 9;
const ERROR_CAUSE_OTHER: u32 = 10;
const DELIVERY_STATE_NONE: u32 = 0;
const DELIVERY_STATE_NOT_SENT: u32 = 1;
const DELIVERY_STATE_POSSIBLY_SENT: u32 = 2;
const DELIVERY_STATE_RESPONSE_STARTED: u32 = 3;
const MAX_CAPABILITY_ID_BYTES: u64 = 128;
const MAX_TEXT_BYTES: u64 = 4_096;
const MAX_CHILD_RULES: u64 = 4_096;
const MAX_ORDER_CONSTRAINTS: u64 = 4_096;
const MAX_TRANSPORT_CONFIG_ITEMS: u64 = 4_096;
const MAX_TRANSPORT_TEXT_BYTES: u64 = 4_096;
// Facade-local allocation guard; KMIPKIT-0013 does not set a material size cap.
const MAX_FFI_INPUT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_DERIVE_KEY_SOURCE_IDENTIFIERS: u64 = 100_000;
const MAX_VALIDATE_INPUT_ITEMS: u64 = 100_000;
const MAX_TRANSPORT_MATERIAL_BYTES: u64 = MAX_FFI_INPUT_BYTES;
const TRANSPORT_ENDPOINT_RAW_TLS: u32 = 1;
const TRANSPORT_ENDPOINT_HTTPS: u32 = 2;
const TRANSPORT_TRUST_PLATFORM: u32 = 1;
const TRANSPORT_TRUST_CERTIFICATES: u32 = 2;
const TRANSPORT_SOURCE_MEMORY: u32 = 1;
const TRANSPORT_SOURCE_FILE: u32 = 2;
const TRANSPORT_ENCODING_PEM: u32 = 1;
const TRANSPORT_ENCODING_DER: u32 = 2;
const TRANSPORT_TIMEOUT_BOUNDED: u32 = 1;
const TRANSPORT_TIMEOUT_UNBOUNDED: u32 = 2;
const TRANSPORT_FLAG_TLS_SERVER_NAME: u32 = 1;
const TRANSPORT_FLAG_TARGET_URI: u32 = 2;
const BATCH_OUTCOME_COMPLETED: u32 = 1;
const BATCH_OUTCOME_PENDING: u32 = 2;

enum DelegatedLoginLeaseTime {
    Interval(u32),
    DateTime(i64),
}

enum LoginLeaseTime {
    Interval(u32),
    DateTime(i64),
}

#[derive(Clone, Copy)]
struct LastError {
    status: i32,
    category: u32,
    cause: u32,
    delivery: u32,
}

impl LastError {
    const NONE: Self = Self {
        status: SUCCESS,
        category: ERROR_CATEGORY_NONE,
        cause: ERROR_CAUSE_NONE,
        delivery: DELIVERY_STATE_NONE,
    };
}

thread_local! {
    static LAST_ERROR: std::cell::Cell<LastError> = const { std::cell::Cell::new(LastError::NONE) };
}

fn record_ffi_error(status: i32) -> i32 {
    let (category, cause) = match status {
        ERROR_INVALID_INPUT | ERROR_INVALID_IDENTITY => {
            (ERROR_CATEGORY_VALIDATION, ERROR_CAUSE_INVALID_INPUT)
        }
        ERROR_COMPATIBILITY_MISMATCH
        | ERROR_DUPLICATE_KEY
        | ERROR_INVALID_SCHEMA
        | ERROR_RESOURCE_LIMIT => (ERROR_CATEGORY_VALIDATION, ERROR_CAUSE_OTHER),
        _ => (ERROR_CATEGORY_OTHER, ERROR_CAUSE_OTHER),
    };
    LAST_ERROR.with(|last_error| {
        last_error.set(LastError {
            status,
            category,
            cause,
            delivery: DELIVERY_STATE_NONE,
        });
    });
    status
}

fn record_client_error(error: &ClientError) -> i32 {
    let category = match error.category() {
        ClientErrorCategory::Validation => ERROR_CATEGORY_VALIDATION,
        ClientErrorCategory::Protocol => ERROR_CATEGORY_PROTOCOL,
        ClientErrorCategory::Transport => ERROR_CATEGORY_TRANSPORT,
        ClientErrorCategory::ServerResult => ERROR_CATEGORY_SERVER_RESULT,
        _ => ERROR_CATEGORY_OTHER,
    };
    let cause = match error.cause_category() {
        None => ERROR_CAUSE_NONE,
        Some(ClientCauseCategory::InvalidInput) => ERROR_CAUSE_INVALID_INPUT,
        Some(ClientCauseCategory::Protocol(cause)) => match cause {
            kmipkit_protocol::ProtocolCauseCategory::InvalidValue => {
                ERROR_CAUSE_PROTOCOL_INVALID_VALUE
            }
            kmipkit_protocol::ProtocolCauseCategory::InvalidEncoding => {
                ERROR_CAUSE_PROTOCOL_INVALID_ENCODING
            }
            _ => ERROR_CAUSE_PROTOCOL_OTHER,
        },
        Some(ClientCauseCategory::Transport(cause)) => match cause {
            TransportCauseCategory::Io => ERROR_CAUSE_TRANSPORT_IO,
            TransportCauseCategory::Tls => ERROR_CAUSE_TRANSPORT_TLS,
            TransportCauseCategory::Http => ERROR_CAUSE_TRANSPORT_HTTP,
            TransportCauseCategory::Timeout => ERROR_CAUSE_TRANSPORT_TIMEOUT,
            _ => ERROR_CAUSE_TRANSPORT_OTHER,
        },
        Some(_) => ERROR_CAUSE_OTHER,
    };
    let delivery = match error.delivery_state() {
        None => DELIVERY_STATE_NONE,
        Some(RequestDeliveryState::NotSent) => DELIVERY_STATE_NOT_SENT,
        Some(RequestDeliveryState::PossiblySent) => DELIVERY_STATE_POSSIBLY_SENT,
        Some(RequestDeliveryState::ResponseStarted | _) => DELIVERY_STATE_RESPONSE_STARTED,
    };
    let status = match error.category() {
        ClientErrorCategory::Validation => ERROR_INVALID_INPUT,
        ClientErrorCategory::Protocol => ERROR_PROTOCOL,
        ClientErrorCategory::Transport => ERROR_TRANSPORT,
        ClientErrorCategory::ServerResult => ERROR_SERVER_RESULT,
        _ => ERROR_INTERNAL,
    };
    LAST_ERROR.with(|last_error| {
        last_error.set(LastError {
            status,
            category,
            cause,
            delivery,
        });
    });
    status
}

trait FfiPanicFallback: Sized {
    fn from_panic() -> Self;
}

impl FfiPanicFallback for i32 {
    fn from_panic() -> Self {
        record_ffi_error(ERROR_INTERNAL)
    }
}

impl FfiPanicFallback for () {
    fn from_panic() -> Self {
        let _ = record_ffi_error(ERROR_INTERNAL);
    }
}

/// Keeps Rust unwinding inside each exported C entry point and returns only a fixed fallback.
///
/// The panic payload is intentionally discarded; the process panic hook remains owned by the
/// embedding application and may observe it before this boundary runs.
fn ffi_entrypoint<R: FfiPanicFallback>(operation: impl FnOnce() -> R) -> R {
    match catch_unwind(AssertUnwindSafe(operation)) {
        Ok(result) => result,
        Err(_payload) => R::from_panic(),
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Kind {
    Identity,
    Compatibility,
    Path,
    Discriminator,
    Schema,
    Information,
    Definition,
    Registry,
    Configuration,
    Validated,
    Registered,
    MessageExtension,
    Structure,
    BatchItem,
    RegisterRequest,
    ChildRule,
    OrderConstraint,
    Value,
    Recognition,
    RawTag,
    Tag,
    Item,
    CodecLimits,
    StructureView,
    ItemView,
    ValueView,
    TransportConfig,
    Client,
    ClientBatch,
    ClientBatchResponse,
    ClientBatchItemResponse,
    UsageAllocationContext,
    SplitKeyReference,
    RevocationReason,
    RevokeRequest,
}

enum HandleValue {
    Identity(extension::ExtensionIdentity),
    Compatibility(extension::Compatibility),
    Path(extension::TtlvPath),
    Discriminator(extension::Discriminator),
    Schema(extension::ExtensionSchema),
    Information(extension::ExtensionInformation),
    Definition(Box<extension::ExtensionDefinition>),
    DefinitionRef {
        registry: Arc<client_extension::ClientExtensionRegistry>,
        index: usize,
    },
    Registry(Arc<client_extension::ClientExtensionRegistry>),
    Configuration(client_extension::ClientConfiguration),
    Validated(kmipkit_protocol::extension::ValidatedExtensionValue),
    ValidatedRef(Arc<Handle>),
    Registered(client_extension::RegisteredExtensionValue),
    MessageExtension(client_extension::ClientRequestMessageExtension),
    Structure(Structure),
    BatchItem(Box<ClientBatchItem>),
    RegisterRequest(Box<RegisterRequest>),
    RevocationReason {
        reason: RevocationReason,
        message: Option<String>,
    },
    RevokeRequest(Box<RevokeRequest>),
    ChildRule(Box<extension::ExtensionChildRule>),
    OrderConstraint(extension::ExtensionOrderConstraint),
    Value(Value),
    Recognition(client_extension::ExtensionRecognition),
    RawTag(RawTag),
    Tag(Tag),
    Item(Item),
    CodecLimits(Arc<CodecLimits>),
    StructureView(TtlvView),
    ItemView(TtlvView),
    ValueView(TtlvView),
    TransportConfig(TransportConfig),
    Client(Box<std::sync::Mutex<Client>>),
    ClientBatch(std::sync::Mutex<ClientBatchBuilder>),
    ClientBatchResponse(Arc<ClientBatchResponse>),
    ClientBatchItemResponse {
        response: Arc<ClientBatchResponse>,
        index: usize,
    },
    UsageAllocationContext(Mutex<UsageAllocationContext>),
    SplitKeyReference(SplitKeyReference),
}

struct Handle {
    kind: Kind,
    value: HandleValue,
}

struct ClientBatchBuilder {
    items: Vec<ClientBatchItem>,
    request_time_stamp: Option<i64>,
    batch_order_option: Option<bool>,
    batch_error_continuation_values: Vec<u32>,
    asynchronous_indicator: Option<u32>,
    client_correlation_value: Option<String>,
}

impl ClientBatchBuilder {
    fn new() -> Self {
        Self {
            items: Vec::new(),
            request_time_stamp: None,
            batch_order_option: None,
            batch_error_continuation_values: Vec::new(),
            asynchronous_indicator: None,
            client_correlation_value: None,
        }
    }

    fn into_batch(self) -> ClientBatch {
        let mut batch = ClientBatch::from_items(self.items);
        if let Some(time_stamp) = self.request_time_stamp {
            batch = batch.with_request_time_stamp(time_stamp);
        }
        if let Some(batch_order_option) = self.batch_order_option {
            batch = batch.with_batch_order_option(batch_order_option);
        }
        for value in self.batch_error_continuation_values {
            batch = batch.with_batch_error_continuation_option(value);
        }
        if let Some(asynchronous_indicator) = self.asynchronous_indicator {
            batch = batch.with_asynchronous_indicator(asynchronous_indicator);
        }
        if let Some(client_correlation_value) = self.client_correlation_value {
            batch = batch.with_client_correlation_value(client_correlation_value);
        }
        batch
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ViewRoot {
    Structure,
    Value,
    RecognitionGeneric,
    ValidatedGeneric,
}

#[derive(Clone, Copy)]
enum ViewStep {
    Item(usize),
    Value,
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum ViewTarget {
    Structure,
    Item,
    Value,
}

// Each of the 64 supported nesting levels records one item selection and its value.
const MAX_VIEW_PATH_STEPS: usize = 2 * 64;

fn copy_view_steps(steps: &[ViewStep], additional_steps: usize) -> FfiResult<Vec<ViewStep>> {
    let required_len = steps
        .len()
        .checked_add(additional_steps)
        .filter(|length| *length <= MAX_VIEW_PATH_STEPS)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let mut copied = Vec::new();
    copied
        .try_reserve_exact(required_len)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    copied.extend_from_slice(steps);
    Ok(copied)
}

/// A C read-only view resolves into an immutable, Arc-retained TTLV owner.
///
/// The path stores item indexes rather than tags so repeated tags retain their
/// original order and the same selected item across descendant views.
struct TtlvView {
    owner: Arc<Handle>,
    root: ViewRoot,
    steps: Vec<ViewStep>,
    target: ViewTarget,
}

impl TtlvView {
    fn new(owner: Arc<Handle>, root: ViewRoot, target: ViewTarget) -> Self {
        Self {
            owner,
            root,
            steps: Vec::new(),
            target,
        }
    }

    fn append(&self, step: ViewStep, target: ViewTarget) -> FfiResult<Self> {
        let mut steps = copy_view_steps(&self.steps, 1)?;
        steps.push(step);
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target,
        })
    }

    fn with_structure_at<R>(
        &self,
        steps: &[ViewStep],
        callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
    ) -> FfiResult<R> {
        match self.root {
            ViewRoot::Structure => {
                let HandleValue::Structure(structure) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                resolve_structure_path(&structure.view(), steps, callback)
            }
            ViewRoot::Value => {
                let HandleValue::Value(value) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                value.with_value(|value| match value {
                    ValueView::Structure(structure) => {
                        resolve_structure_path(&structure, steps, callback)
                    }
                    _ => Err(ERROR_INVALID_INPUT),
                })
            }
            ViewRoot::RecognitionGeneric => {
                let HandleValue::Recognition(recognition) = &self.owner.value else {
                    return Err(ERROR_INVALID_INPUT);
                };
                resolve_structure_path(
                    &client_extension::generic_value(recognition).view(),
                    steps,
                    callback,
                )
            }
            ViewRoot::ValidatedGeneric => {
                let validated = validated_from_handle(&self.owner)?;
                resolve_structure_path(&extension::generic_value(validated).view(), steps, callback)
            }
        }
    }

    fn with_structure<R>(
        &self,
        callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
    ) -> FfiResult<R> {
        if self.target != ViewTarget::Structure {
            return Err(ERROR_INVALID_INPUT);
        }
        self.with_structure_at(&self.steps, callback)
    }

    fn with_item<R>(&self, callback: impl FnOnce(&Item) -> R) -> FfiResult<R> {
        if self.target != ViewTarget::Item {
            return Err(ERROR_INVALID_INPUT);
        }
        let Some((ViewStep::Item(index), parent_steps)) = self.steps.split_last() else {
            return Err(ERROR_INVALID_INPUT);
        };
        self.with_structure_at(parent_steps, |structure| {
            let item = structure
                .children()
                .get(*index)
                .ok_or(ERROR_INVALID_INPUT)?;
            Ok(callback(item))
        })?
    }

    fn with_value<R>(
        &self,
        callback: impl for<'a> FnOnce(ValueView<'a>) -> FfiResult<R>,
    ) -> FfiResult<R> {
        if self.target != ViewTarget::Value {
            return Err(ERROR_INVALID_INPUT);
        }
        if self.root == ViewRoot::Value && self.steps.is_empty() {
            let HandleValue::Value(value) = &self.owner.value else {
                return Err(ERROR_INVALID_INPUT);
            };
            return value.with_value(callback);
        }

        self.with_structure_at(&[], |structure| {
            resolve_value_path(structure, &self.steps, callback)
        })?
    }

    fn item_at(&self, index: usize) -> FfiResult<Self> {
        let count = self.with_structure(|structure| structure.children().len())?;
        if index >= count {
            return Err(ERROR_INVALID_INPUT);
        }
        self.append(ViewStep::Item(index), ViewTarget::Item)
    }

    fn item_value(&self) -> FfiResult<Self> {
        if self.target != ViewTarget::Item {
            return Err(ERROR_INVALID_INPUT);
        }
        self.append(ViewStep::Value, ViewTarget::Value)
    }

    fn structure_from_value(&self) -> FfiResult<Self> {
        self.as_structure(ERROR_INVALID_INPUT)
    }

    fn nested_structure_from_value(&self) -> FfiResult<Self> {
        self.as_structure(ERROR_INVALID_INPUT)
    }

    fn as_structure(&self, mismatch_status: i32) -> FfiResult<Self> {
        let is_structure = self.with_value(|value| Ok(matches!(value, ValueView::Structure(_))))?;
        if !is_structure {
            return Err(mismatch_status);
        }
        let steps = copy_view_steps(&self.steps, 0)?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target: ViewTarget::Structure,
        })
    }

    fn value_at_tags(&self, tags: &[Tag]) -> FfiResult<Self> {
        if self.target != ViewTarget::Structure {
            return Err(ERROR_INVALID_INPUT);
        }
        if tags.is_empty() {
            return Err(ERROR_INVALID_INPUT);
        }
        let mut selected = self.duplicate()?;
        for (index, tag) in tags.iter().enumerate() {
            let child_index = selected.with_structure(|structure| {
                structure
                    .children()
                    .iter()
                    .position(|item| item.tag() == *tag)
            })?;
            let child_index = child_index.ok_or(ERROR_INVALID_INPUT)?;
            selected = selected.item_at(child_index)?;
            if index + 1 < tags.len() {
                selected = selected.item_value()?.nested_structure_from_value()?;
            } else {
                selected = selected.item_value()?;
            }
        }
        Ok(selected)
    }

    fn duplicate(&self) -> FfiResult<Self> {
        let steps = copy_view_steps(&self.steps, 0)?;
        Ok(Self {
            owner: Arc::clone(&self.owner),
            root: self.root,
            steps,
            target: self.target,
        })
    }
}

fn resolve_structure_path<R>(
    structure: &kmipkit_ttlv::StructureView<'_>,
    steps: &[ViewStep],
    callback: impl for<'a> FnOnce(&kmipkit_ttlv::StructureView<'a>) -> R,
) -> FfiResult<R> {
    if steps.is_empty() {
        return Ok(callback(structure));
    }
    let [ViewStep::Item(index), ViewStep::Value, remaining @ ..] = steps else {
        return Err(ERROR_INVALID_INPUT);
    };
    let item = structure
        .children()
        .get(*index)
        .ok_or(ERROR_INVALID_INPUT)?;
    item.with_value(|value| match value {
        ValueView::Structure(nested) => resolve_structure_path(&nested, remaining, callback),
        _ => Err(ERROR_INVALID_INPUT),
    })
}

fn resolve_value_path<R>(
    structure: &kmipkit_ttlv::StructureView<'_>,
    steps: &[ViewStep],
    callback: impl for<'a> FnOnce(ValueView<'a>) -> FfiResult<R>,
) -> FfiResult<R> {
    let [ViewStep::Item(index), ViewStep::Value, remaining @ ..] = steps else {
        return Err(ERROR_INVALID_INPUT);
    };
    let item = structure
        .children()
        .get(*index)
        .ok_or(ERROR_INVALID_INPUT)?;
    item.with_value(|value| {
        if remaining.is_empty() {
            callback(value)
        } else {
            match value {
                ValueView::Structure(nested) => resolve_value_path(&nested, remaining, callback),
                _ => Err(ERROR_INVALID_INPUT),
            }
        }
    })
}

type FfiResult<T> = Result<T, i32>;

const TOKEN_HALF_BITS: u32 = usize::BITS / 2;
const TOKEN_HALF_MASK: usize = (1_usize << TOKEN_HALF_BITS) - 1;
const TOKEN_MAX: usize = isize::MAX as usize;
const TOKEN_ROUNDS: u8 = 8;
const TOKEN_CYCLE_WALK_LIMIT: usize = 128;
const TOKEN_DOMAIN: &[u8] = b"KMIPKit FFI opaque handle token v1";
const RIGHT_BYTES_OFFSET: usize = std::mem::size_of::<usize>() - std::mem::size_of::<u32>();

fn cycle_walk_token(mut token: usize, mut permute: impl FnMut(usize) -> usize) -> FfiResult<usize> {
    for _ in 0..TOKEN_CYCLE_WALK_LIMIT {
        token = permute(token);
        if (1..=TOKEN_MAX).contains(&token) {
            return Ok(token);
        }
    }
    Err(ERROR_RESOURCE_LIMIT)
}

struct HandleRegistry {
    next_sequence: AtomicUsize,
    token_key: hmac::Key,
    handles: Mutex<HashMap<usize, Arc<Handle>>>,
}

impl HandleRegistry {
    fn new() -> FfiResult<Self> {
        let token_key = hmac::Key::generate(hmac::HMAC_SHA256, &rand::SystemRandom::new())
            .map_err(|_| ERROR_INTERNAL)?;
        Ok(Self::with_token_key(token_key))
    }

    fn with_token_key(token_key: hmac::Key) -> Self {
        Self {
            next_sequence: AtomicUsize::new(1),
            token_key,
            handles: Mutex::new(HashMap::new()),
        }
    }

    #[cfg(test)]
    fn with_test_key(key: &[u8]) -> Self {
        Self::with_token_key(hmac::Key::new(hmac::HMAC_SHA256, key))
    }

    fn allocate_id(&self) -> FfiResult<usize> {
        let mut current = self.next_sequence.load(Ordering::Relaxed);
        loop {
            if current == 0 || current > TOKEN_MAX {
                return Err(ERROR_RESOURCE_LIMIT);
            }
            let next = if current == TOKEN_MAX { 0 } else { current + 1 };
            match self.next_sequence.compare_exchange_weak(
                current,
                next,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => return self.token_for_sequence(current),
                Err(observed) => current = observed,
            }
        }
    }

    fn token_for_sequence(&self, sequence: usize) -> FfiResult<usize> {
        debug_assert!((1..=TOKEN_MAX).contains(&sequence));
        cycle_walk_token(sequence, |token| self.permute(token))
    }

    // A keyed Feistel permutation hides the monotonic sequence while preserving uniqueness.
    // Bounded cycle walking excludes null and high-bit pointer values without collisions.
    fn permute(&self, input: usize) -> usize {
        let mut left = (input >> TOKEN_HALF_BITS) & TOKEN_HALF_MASK;
        let mut right = input & TOKEN_HALF_MASK;

        for round in 0..TOKEN_ROUNDS {
            let mut context = hmac::Context::with_key(&self.token_key);
            context.update(TOKEN_DOMAIN);
            context.update(&[round]);
            let right_bytes = right.to_be_bytes();
            context.update(&right_bytes[RIGHT_BYTES_OFFSET..]);
            let tag = context.sign();
            let digest = tag.as_ref();
            let round_key = u32::from_be_bytes([digest[0], digest[1], digest[2], digest[3]])
                as usize
                & TOKEN_HALF_MASK;
            (left, right) = (right, (left ^ round_key) & TOKEN_HALF_MASK);
        }

        (left << TOKEN_HALF_BITS) | right
    }
}

static HANDLE_REGISTRY: OnceLock<FfiResult<HandleRegistry>> = OnceLock::new();

fn handle_registry() -> FfiResult<&'static HandleRegistry> {
    HANDLE_REGISTRY
        .get_or_init(HandleRegistry::new)
        .as_ref()
        .map_err(|status| *status)
}

fn handle_id<T>(pointer: *mut T) -> FfiResult<usize> {
    if pointer.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // Opaque tokens have no pointer provenance and are never dereferenced. Their address bits are
    // used only as keys into the live-handle table.
    Ok(pointer.addr())
}

fn try_make_handle<T>(kind: Kind, value: HandleValue) -> FfiResult<*mut T> {
    let registry = handle_registry()?;
    let id = registry.allocate_id()?;
    let handle = Arc::new(Handle { kind, value });
    let mut handles = registry.handles.lock().map_err(|_| ERROR_INTERNAL)?;
    match handles.entry(id) {
        Entry::Vacant(entry) => {
            entry.insert(handle);
        }
        Entry::Occupied(_) => return Err(ERROR_INTERNAL),
    }
    Ok(ptr::without_provenance_mut(id))
}

fn reference_handle<T>(pointer: *mut T, expected: Kind) -> FfiResult<Arc<Handle>> {
    let id = handle_id(pointer)?;
    let handles = handle_registry()?
        .handles
        .lock()
        .map_err(|_| ERROR_INTERNAL)?;
    let handle = handles.get(&id).cloned().ok_or(ERROR_INVALID_INPUT)?;
    if handle.kind != expected {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(handle)
}

fn clone_handle_owner<T>(pointer: *mut T, expected: Kind) -> FfiResult<Arc<Handle>> {
    reference_handle(pointer, expected)
}

fn consume_owner<T>(pointer: *mut T, expected: Kind) -> FfiResult<Arc<Handle>> {
    let id = handle_id(pointer)?;
    let mut handles = handle_registry()?
        .handles
        .lock()
        .map_err(|_| ERROR_INTERNAL)?;
    match handles.entry(id) {
        Entry::Occupied(entry) if entry.get().kind == expected => Ok(entry.remove()),
        Entry::Occupied(_) | Entry::Vacant(_) => Err(ERROR_INVALID_INPUT),
    }
}

fn unwrap_payload(handle: Arc<Handle>) -> FfiResult<HandleValue> {
    match Arc::try_unwrap(handle) {
        Ok(handle) => Ok(handle.value),
        Err(shared) => {
            drop(shared);
            Err(ERROR_INVALID_INPUT)
        }
    }
}

fn owned_payload<T>(pointer: *mut T, expected: Kind) -> FfiResult<HandleValue> {
    unwrap_payload(consume_owner(pointer, expected)?)
}

macro_rules! out_slot {
    ($out:expr) => {{
        if $out.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: this is the non-null writable output pointer supplied by the C caller.
        let slot = unsafe { &mut *$out };
        *slot = ptr::null_mut();
        slot
    }};
}

macro_rules! scalar_out {
    ($out:expr) => {{
        if $out.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        // SAFETY: this is the non-null writable output pointer supplied by the C caller.
        unsafe { &mut *$out }
    }};
}

fn input_span<'a>(data: *const u8, length: u64, maximum: u64) -> FfiResult<&'a [u8]> {
    if length > maximum || length > isize::MAX as u64 {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let length = usize::try_from(length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if length == 0 {
        return Ok(&[]);
    }
    if data.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: for a non-zero span, the C caller must provide `length` readable bytes; the length
    // has already been checked against the operation's bound and converted to usize.
    Ok(unsafe { std::slice::from_raw_parts(data, length) })
}

fn input_array<'a, T: Copy>(array: *const T, count: u64, maximum: u64) -> FfiResult<&'a [T]> {
    if count > maximum {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let count = usize::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let byte_length = count
        .checked_mul(size_of::<T>())
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if isize::try_from(count).is_err() || isize::try_from(byte_length).is_err() {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    if count == 0 {
        return Ok(&[]);
    }
    if array.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: the C caller must provide `count` readable elements; count and
    // byte length fit Rust slice limits before constructing the borrowed slice.
    Ok(unsafe { std::slice::from_raw_parts(array, count) })
}

fn input_handle_pointers<'a, T>(
    array: *mut *mut T,
    count: u64,
    maximum: u64,
) -> FfiResult<&'a [*mut T]> {
    if count > maximum {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let count = usize::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if count == 0 {
        return Ok(&[]);
    }
    if array.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: for nonzero `count`, the C caller must provide a readable array of that many handle
    // pointers. The count is bounded before reading any array element.
    Ok(unsafe { std::slice::from_raw_parts(array, count) })
}

fn input_handle_array<T, U>(
    array: *mut *mut T,
    count: u64,
    maximum: u64,
    kind: Kind,
    mut copy: impl FnMut(&Handle) -> FfiResult<U>,
) -> FfiResult<Vec<U>> {
    let pointers = input_handle_pointers(array, count, maximum)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(pointers.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for pointer in pointers {
        let handle = reference_handle(*pointer, kind)?;
        result.push(copy(&handle)?);
    }
    Ok(result)
}

fn copy_join_split_key_references(
    array: *mut *mut kmipkit_split_key_reference_t,
    count: u64,
) -> FfiResult<Vec<SplitKeyReference>> {
    let pointers = input_handle_pointers(array, count, 100_000)?;
    let mut handles = Vec::new();
    handles
        .try_reserve_exact(pointers.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;

    let mut total_identifier_bytes = 0_u64;
    for pointer in pointers {
        let handle = reference_handle(*pointer, Kind::SplitKeyReference)?;
        let HandleValue::SplitKeyReference(reference) = &handle.value else {
            return Err(ERROR_INVALID_INPUT);
        };
        if reference.split_key_threshold().is_none() {
            return Err(ERROR_INVALID_INPUT);
        }
        if let UniqueIdentifier::TextString(value) = reference.unique_identifier() {
            let length = u64::try_from(value.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            total_identifier_bytes = total_identifier_bytes
                .checked_add(length)
                .filter(|total| *total <= MAX_FFI_INPUT_BYTES)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
        }
        handles.push(handle);
    }

    let mut references = Vec::new();
    references
        .try_reserve_exact(handles.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for handle in handles {
        let HandleValue::SplitKeyReference(reference) = &handle.value else {
            return Err(ERROR_INVALID_INPUT);
        };
        let threshold = reference.split_key_threshold().ok_or(ERROR_INVALID_INPUT)?;
        let identifier = try_clone_join_split_key_identifier(reference.unique_identifier())?;
        references.push(SplitKeyReference::new(identifier).with_split_key_threshold(threshold));
    }
    Ok(references)
}

fn copy_join_split_key_identifier_with_reservation<E>(
    value: &str,
    reserve: impl FnOnce(&mut String, usize) -> Result<(), E>,
) -> FfiResult<String> {
    let mut copy = String::new();
    reserve(&mut copy, value.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    copy.push_str(value);
    Ok(copy)
}

fn try_clone_join_split_key_identifier(
    identifier: &UniqueIdentifier,
) -> FfiResult<UniqueIdentifier> {
    match identifier {
        UniqueIdentifier::TextString(value) => Ok(UniqueIdentifier::TextString(
            copy_join_split_key_identifier_with_reservation(value, |copy, length| {
                copy.try_reserve_exact(length)
            })?,
        )),
        UniqueIdentifier::Enumeration(value) => Ok(UniqueIdentifier::Enumeration(*value)),
        UniqueIdentifier::Integer(value) => Ok(UniqueIdentifier::Integer(*value)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn protocol_status(error: &ProtocolError) -> i32 {
    match error.kind() {
        ProtocolErrorKind::CompatibilityMismatch => ERROR_COMPATIBILITY_MISMATCH,
        ProtocolErrorKind::DuplicateKey => ERROR_DUPLICATE_KEY,
        ProtocolErrorKind::InvalidIdentity => ERROR_INVALID_IDENTITY,
        ProtocolErrorKind::InvalidInput => ERROR_INVALID_INPUT,
        ProtocolErrorKind::ResourceLimit => ERROR_RESOURCE_LIMIT,
        _ => ERROR_INVALID_SCHEMA,
    }
}

fn client_status(error: &kmipkit_client::ClientError) -> i32 {
    match error {
        kmipkit_client::ClientError::Protocol { error, .. } => protocol_status(error),
        _ => ERROR_INVALID_INPUT,
    }
}

fn copy_registry_definitions(
    pointers: &[*mut kmipkit_extension_definition_t],
    limits: extension::ExtensionRegistryLimits,
    mut copy: impl FnMut(&extension::ExtensionDefinition) -> FfiResult<extension::ExtensionDefinition>,
) -> FfiResult<Vec<extension::ExtensionDefinition>> {
    let count = u64::try_from(pointers.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut preflight = client_extension::RegistryDefinitionPreflight::new(count, limits)
        .map_err(|error| protocol_status(&error))?;
    for pointer in pointers {
        let handle = reference_handle(*pointer, Kind::Definition)?;
        let definition = definition_from_handle(&handle)?;
        preflight
            .add_definition(definition)
            .map_err(|error| protocol_status(&error))?;
    }
    preflight
        .finish()
        .map_err(|error| protocol_status(&error))?;

    let mut copied = Vec::new();
    copied
        .try_reserve_exact(pointers.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for pointer in pointers {
        let handle = reference_handle(*pointer, Kind::Definition)?;
        copied.push(copy(definition_from_handle(&handle)?)?);
    }
    Ok(copied)
}

fn model_status(error: kmipkit_ttlv::ModelError) -> i32 {
    match error {
        kmipkit_ttlv::ModelError::RawTagOutOfRange => ERROR_INVALID_INPUT,
        kmipkit_ttlv::ModelError::StructureDepthExceeded => ERROR_RESOURCE_LIMIT,
        _ => ERROR_INVALID_SCHEMA,
    }
}

fn item_type_from_u32(value: u32) -> FfiResult<ItemType> {
    match value {
        1 => Ok(ItemType::Structure),
        2 => Ok(ItemType::Integer),
        3 => Ok(ItemType::LongInteger),
        4 => Ok(ItemType::BigInteger),
        5 => Ok(ItemType::Enumeration),
        6 => Ok(ItemType::Boolean),
        7 => Ok(ItemType::TextString),
        8 => Ok(ItemType::ByteString),
        9 => Ok(ItemType::DateTime),
        10 => Ok(ItemType::Interval),
        11 => Ok(ItemType::DateTimeExtended),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn item_type_code(value: ItemType) -> u8 {
    match value {
        ItemType::Structure => 1,
        ItemType::Integer => 2,
        ItemType::LongInteger => 3,
        ItemType::BigInteger => 4,
        ItemType::Enumeration => 5,
        ItemType::Boolean => 6,
        ItemType::TextString => 7,
        ItemType::ByteString => 8,
        ItemType::DateTime => 9,
        ItemType::Interval => 10,
        ItemType::DateTimeExtended => 11,
        _ => 0,
    }
}

fn value_view_type_code(value: &ValueView<'_>) -> u8 {
    match value {
        ValueView::Structure(_) => 1,
        ValueView::Integer(_) => 2,
        ValueView::LongInteger(_) => 3,
        ValueView::BigInteger(_) => 4,
        ValueView::Enumeration(_) => 5,
        ValueView::Boolean(_) => 6,
        ValueView::TextString(_) => 7,
        ValueView::ByteString(_) => 8,
        ValueView::DateTime(_) => 9,
        ValueView::Interval(_) => 10,
        ValueView::DateTimeExtended(_) => 11,
        _ => 0,
    }
}

fn clone_value(value: ValueView<'_>) -> FfiResult<Value> {
    match value {
        ValueView::Structure(structure) => Ok(Value::structure(clone_structure_view(&structure)?)),
        ValueView::Integer(value) => Ok(Value::integer(*value)),
        ValueView::LongInteger(value) => Ok(Value::long_integer(*value)),
        ValueView::BigInteger(value) => Ok(Value::big_integer(value.to_vec())),
        ValueView::Enumeration(value) => Ok(Value::enumeration(*value)),
        ValueView::Boolean(value) => Ok(Value::boolean(*value)),
        ValueView::TextString(value) => Ok(Value::text_string((*value).to_owned())),
        ValueView::ByteString(value) => Ok(Value::byte_string(value.to_vec())),
        ValueView::DateTime(value) => Ok(Value::date_time(*value)),
        ValueView::Interval(value) => Ok(Value::interval(*value)),
        ValueView::DateTimeExtended(value) => Ok(Value::date_time_extended(*value)),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn clone_item(item: &Item) -> FfiResult<Item> {
    Item::new(item.tag(), item.with_value(clone_value)?).map_err(model_status)
}

fn clone_structure(structure: &Structure) -> FfiResult<Structure> {
    let mut clone = Structure::new();
    for item in structure.view().children() {
        clone.try_push(clone_item(item)?).map_err(model_status)?;
    }
    Ok(clone)
}

fn clone_structure_view(structure: &kmipkit_ttlv::StructureView<'_>) -> FfiResult<Structure> {
    let mut clone = Structure::new();
    for item in structure.children() {
        clone.try_push(clone_item(item)?).map_err(model_status)?;
    }
    Ok(clone)
}

fn derive_key_parameters_from_handle(
    structure: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<DerivationParameters> {
    const CRYPTOGRAPHIC_PARAMETERS: u32 = 0x0042_002B;
    const INITIALIZATION_VECTOR: u32 = 0x0042_003A;
    const DERIVATION_DATA: u32 = 0x0042_0030;
    const SALT: u32 = 0x0042_0084;
    const ITERATION_COUNT: u32 = 0x0042_003C;

    if structure.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    let handle = reference_handle(structure, Kind::Structure)?;
    let HandleValue::Structure(source) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };

    let mut seen = [false; 4];
    let mut parameters = DerivationParameters::new();
    for item in source.view().children() {
        match item.tag().raw() {
            CRYPTOGRAPHIC_PARAMETERS => {
                if seen[0] {
                    return Err(ERROR_INVALID_INPUT);
                }
                seen[0] = true;
                let value = item.with_value(|value| match value {
                    ValueView::Structure(value) => Some(clone_structure_view(&value)),
                    _ => None,
                });
                let structure = value.ok_or(ERROR_INVALID_INPUT)??;
                parameters = parameters.with_cryptographic_parameters(structure);
            }
            INITIALIZATION_VECTOR => {
                if seen[1] {
                    return Err(ERROR_INVALID_INPUT);
                }
                seen[1] = true;
                let bytes = item
                    .with_value(|value| match value {
                        ValueView::ByteString(value) => Some(value.to_vec()),
                        _ => None,
                    })
                    .ok_or(ERROR_INVALID_INPUT)?;
                parameters = parameters.with_initialization_vector(SecretBytes::new(bytes));
            }
            DERIVATION_DATA => {
                let bytes = item
                    .with_value(|value| match value {
                        ValueView::ByteString(value) => Some(value.to_vec()),
                        _ => None,
                    })
                    .ok_or(ERROR_INVALID_INPUT)?;
                parameters = parameters.with_derivation_data(SecretBytes::new(bytes));
            }
            SALT => {
                if seen[2] {
                    return Err(ERROR_INVALID_INPUT);
                }
                seen[2] = true;
                let bytes = item
                    .with_value(|value| match value {
                        ValueView::ByteString(value) => Some(value.to_vec()),
                        _ => None,
                    })
                    .ok_or(ERROR_INVALID_INPUT)?;
                parameters = parameters.with_salt(SecretBytes::new(bytes));
            }
            ITERATION_COUNT => {
                if seen[3] {
                    return Err(ERROR_INVALID_INPUT);
                }
                seen[3] = true;
                let value = item
                    .with_value(|value| match value {
                        ValueView::Integer(value) => Some(*value),
                        _ => None,
                    })
                    .ok_or(ERROR_INVALID_INPUT)?;
                parameters = parameters.with_iteration_count(value);
            }
            _ => {
                parameters = parameters
                    .with_extension(item.try_clone().map_err(model_status)?)
                    .map_err(|error| protocol_status(&error))?;
            }
        }
    }
    Ok(parameters)
}
fn limits_from_fields(fields: [u64; 12]) -> FfiResult<extension::ExtensionRegistryLimits> {
    extension::with_values(
        fields[0], fields[1], fields[2], fields[3], fields[4], fields[5], fields[6], fields[7],
        fields[8], fields[9], fields[10], fields[11],
    )
    .map_err(|error| protocol_status(&error))
}

fn validate_versioned_limits_header(
    limits: *const kmipkit_extension_registry_limits_v1_t,
) -> FfiResult<()> {
    if limits.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    // SAFETY: a non-null pointer to a readable v1 limits structure is a caller precondition.
    let header = unsafe { &*limits };
    let Ok(minimum_size) =
        u32::try_from(std::mem::size_of::<kmipkit_extension_registry_limits_v1_t>())
    else {
        return Err(ERROR_INVALID_INPUT);
    };
    if header.struct_size != minimum_size || header.abi_version != 1 {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(())
}

fn limits_from_versioned(
    limits: *const kmipkit_extension_registry_limits_v1_t,
) -> FfiResult<extension::ExtensionRegistryLimits> {
    validate_versioned_limits_header(limits)?;
    // SAFETY: the header validator established a non-null pointer to the complete v1 prefix.
    let limits = unsafe { &*limits };
    limits_from_fields([
        limits.max_definitions,
        limits.max_schema_nodes,
        limits.max_child_rules_per_structure,
        limits.max_text_bytes_per_field,
        limits.max_registry_text_bytes,
        limits.max_discriminator_scalar_bytes,
        limits.max_total_discriminator_scalar_bytes,
        limits.max_constraint_members_per_rule,
        limits.max_total_constraint_members,
        limits.max_payload_index_records,
        limits.max_lookup_comparisons,
        limits.max_depth,
    ])
}

fn registry_from_handle(
    handle: &Handle,
) -> FfiResult<Arc<client_extension::ClientExtensionRegistry>> {
    match &handle.value {
        HandleValue::Registry(registry) => Ok(Arc::clone(registry)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn definition_from_handle(handle: &Handle) -> FfiResult<&extension::ExtensionDefinition> {
    match &handle.value {
        HandleValue::Definition(definition) => Ok(definition),
        HandleValue::DefinitionRef { registry, index } => {
            client_extension::definition_at(registry, *index).ok_or(ERROR_INVALID_INPUT)
        }
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn validated_from_handle(
    handle: &Handle,
) -> FfiResult<&kmipkit_protocol::extension::ValidatedExtensionValue> {
    match &handle.value {
        HandleValue::Validated(value) => Ok(value),
        HandleValue::ValidatedRef(owner) => match &owner.value {
            HandleValue::Recognition(recognition) => {
                recognition.validated_value().ok_or(ERROR_INVALID_INPUT)
            }
            _ => Err(ERROR_INVALID_INPUT),
        },
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn reference_registry(
    pointer: *mut kmipkit_client_extension_registry_t,
) -> FfiResult<Arc<client_extension::ClientExtensionRegistry>> {
    let handle = reference_handle(pointer, Kind::Registry)?;
    registry_from_handle(&handle)
}

fn release<T>(pointer: *mut T, kind: Kind) {
    if let Ok(handle) = consume_owner(pointer, kind) {
        drop(handle);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_release(handle: *mut kmipkit_extension_identity_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Identity);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_compatibility_release(
    handle: *mut kmipkit_extension_compatibility_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Compatibility);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_release(handle: *mut kmipkit_ttlv_path_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Path);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_discriminator_release(
    handle: *mut kmipkit_extension_discriminator_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Discriminator);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_release(handle: *mut kmipkit_extension_schema_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Schema);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_release(
    handle: *mut kmipkit_extension_information_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Information);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_release(
    handle: *mut kmipkit_extension_definition_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Definition);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_release(
    handle: *mut kmipkit_client_extension_registry_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Registry);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_release(
    handle: *mut kmipkit_client_configuration_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Configuration);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_transport_config_release(handle: *mut kmipkit_transport_config_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::TransportConfig);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_release(
    handle: *mut kmipkit_validated_extension_value_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Validated);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_registered_extension_value_release(
    handle: *mut kmipkit_registered_extension_value_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Registered);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_request_message_extension_release(
    handle: *mut kmipkit_client_request_message_extension_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::MessageExtension);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_register_request_release(handle: *mut kmipkit_register_request_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::RegisterRequest);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_revocation_reason_release(handle: *mut kmipkit_revocation_reason_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::RevocationReason);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_revoke_request_release(handle: *mut kmipkit_revoke_request_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::RevokeRequest);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_release(handle: *mut kmipkit_ttlv_structure_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Structure);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_release(handle: *mut kmipkit_client_batch_item_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::BatchItem);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_split_key_reference_release(handle: *mut kmipkit_split_key_reference_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::SplitKeyReference);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_release(
    handle: *mut kmipkit_extension_child_rule_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::ChildRule);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_order_constraint_release(
    handle: *mut kmipkit_extension_order_constraint_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::OrderConstraint);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_release(handle: *mut kmipkit_ttlv_value_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Value);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_release(
    handle: *mut kmipkit_extension_recognition_t,
) {
    ffi_entrypoint(|| {
        release(handle, Kind::Recognition);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_raw_tag_release(handle: *mut kmipkit_raw_tag_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::RawTag);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_tag_release(handle: *mut kmipkit_tag_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Tag);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_release(handle: *mut kmipkit_ttlv_item_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::Item);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_release(handle: *mut kmipkit_codec_limits_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::CodecLimits);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_release(handle: *mut kmipkit_ttlv_structure_view_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::StructureView);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_release(handle: *mut kmipkit_ttlv_item_view_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::ItemView);
    });
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_release(handle: *mut kmipkit_ttlv_value_view_t) {
    ffi_entrypoint(|| {
        release(handle, Kind::ValueView);
    });
}

macro_rules! try_ffi {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(status) => return record_ffi_error(status),
        }
    };
}

fn package_version_components() -> Option<[u32; 3]> {
    let core = env!("CARGO_PKG_VERSION").split(['-', '+']).next()?;
    let mut components = core.split('.');
    let major = components.next()?.parse().ok()?;
    let minor = components.next()?.parse().ok()?;
    let patch = components.next()?.parse().ok()?;
    if components.next().is_some() {
        return None;
    }
    Some([major, minor, patch])
}

/// Returns the package semantic version and the stable C ABI revision.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_version_get(
    out_major: *mut u32,
    out_minor: *mut u32,
    out_patch: *mut u32,
    out_abi_version: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        if out_major.is_null()
            || out_minor.is_null()
            || out_patch.is_null()
            || out_abi_version.is_null()
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Some([major, minor, patch]) = package_version_components() else {
            return record_ffi_error(ERROR_INVALID_SCHEMA);
        };
        // SAFETY: all four output pointers were checked non-null and are caller-provided writable
        // scalar outputs for the duration of this call.
        unsafe {
            *out_major = major;
            *out_minor = minor;
            *out_patch = patch;
            *out_abi_version = kmipkit::ABI_VERSION;
        }
        SUCCESS
    })
}

/// Returns whether a canonical capability has an unblocked C ABI mapping.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_capability_is_available(
    capability_id_data: *const u8,
    capability_id_length: u64,
    out_available: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        if out_available.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let bytes = try_ffi!(input_span(
            capability_id_data,
            capability_id_length,
            MAX_CAPABILITY_ID_BYTES,
        ));
        let Ok(capability_id) = std::str::from_utf8(bytes) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let available =
            crate::public_api_generated::C_AVAILABLE_CAPABILITIES.contains(&capability_id);
        // SAFETY: the output pointer was checked non-null and is writable for this call.
        unsafe { *out_available = u8::from(available) };
        SUCCESS
    })
}

/// Returns the local API taxonomy through borrowed spans into static strings.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_api_levels_get_v1(
    out_high_level: *mut crate::kmipkit_byte_span_v1_t,
    out_typed_protocol: *mut crate::kmipkit_byte_span_v1_t,
    out_generic_ttlv: *mut crate::kmipkit_byte_span_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if out_high_level.is_null() || out_typed_protocol.is_null() || out_generic_ttlv.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }

        let levels = kmipkit::api_levels();
        if levels.len() != 3 {
            return record_ffi_error(ERROR_INTERNAL);
        }
        let span = |level: &str| crate::kmipkit_byte_span_v1_t {
            data: level.as_ptr(),
            length: level.len() as u64,
        };

        // SAFETY: all output pointers were checked non-null and are caller-provided writable
        // span slots. Each span points to a static string returned by `kmipkit::api_levels()`.
        unsafe {
            *out_high_level = span(levels[0]);
            *out_typed_protocol = span(levels[1]);
            *out_generic_ttlv = span(levels[2]);
        }
        SUCCESS
    })
}

/// Copies this thread's last categorized ABI failure without clearing it.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_last_error_get(
    out_status: *mut i32,
    out_category: *mut u32,
    out_cause: *mut u32,
    out_delivery: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        if out_status.is_null()
            || out_category.is_null()
            || out_cause.is_null()
            || out_delivery.is_null()
        {
            // This accessor is deliberately non-destructive, including for invalid output pointers.
            return ERROR_INVALID_INPUT;
        }
        let snapshot = LAST_ERROR.with(std::cell::Cell::get);
        // SAFETY: all four output pointers were checked non-null and are caller-provided writable
        // scalar outputs for the duration of this call.
        unsafe {
            *out_status = snapshot.status;
            *out_category = snapshot.category;
            *out_cause = snapshot.cause;
            *out_delivery = snapshot.delivery;
        }
        SUCCESS
    })
}

/// Clears this thread's last ABI failure snapshot.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_last_error_clear() -> i32 {
    ffi_entrypoint(|| {
        LAST_ERROR.with(|last_error| last_error.set(LastError::NONE));
        SUCCESS
    })
}

macro_rules! read_handle {
    ($pointer:expr, $kind:ident) => {
        try_ffi!(reference_handle($pointer, Kind::$kind))
    };
}

#[allow(unsafe_code)]
pub(super) mod client_abi;

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_create(
    vendor_identifier_data: *const u8,
    vendor_identifier_length: u64,
    name_data: *const u8,
    name_length: u64,
    version_data: *const u8,
    version_length: u64,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_identity);
        let vendor = try_ffi!(input_span(
            vendor_identifier_data,
            vendor_identifier_length,
            MAX_TEXT_BYTES
        ));
        let name = try_ffi!(input_span(name_data, name_length, MAX_TEXT_BYTES));
        let version = try_ffi!(input_span(version_data, version_length, MAX_TEXT_BYTES));
        let vendor = try_ffi!(std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_IDENTITY));
        let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_IDENTITY));
        let version = try_ffi!(std::str::from_utf8(version).map_err(|_| ERROR_INVALID_IDENTITY));
        let value = try_ffi!(
            extension::extension_identity(vendor, name, version).map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Identity,
            HandleValue::Identity(value)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_vendor_identifier(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let identity = read_handle!(identity, Identity);
        let HandleValue::Identity(identity) = &identity.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::text_string(identity.vendor_identifier().to_owned())),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_name(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let identity = read_handle!(identity, Identity);
        let HandleValue::Identity(identity) = &identity.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::text_string(identity.name().to_owned())),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_identity_version(
    identity: *mut kmipkit_extension_identity_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let identity = read_handle!(identity, Identity);
        let HandleValue::Identity(identity) = &identity.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::text_string(identity.version().to_owned())),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_compatibility_create(
    kmip_min_major: u8,
    kmip_min_minor: u8,
    kmip_max_major: u8,
    kmip_max_minor: u8,
    kmipkit_minimum_data: *const u8,
    kmipkit_minimum_length: u64,
    kmipkit_maximum_data: *const u8,
    kmipkit_maximum_length: u64,
    out_compatibility: *mut *mut kmipkit_extension_compatibility_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_compatibility);
        let minimum = try_ffi!(input_span(
            kmipkit_minimum_data,
            kmipkit_minimum_length,
            MAX_TEXT_BYTES
        ));
        let maximum = try_ffi!(input_span(
            kmipkit_maximum_data,
            kmipkit_maximum_length,
            MAX_TEXT_BYTES
        ));
        let minimum =
            try_ffi!(std::str::from_utf8(minimum).map_err(|_| ERROR_COMPATIBILITY_MISMATCH));
        let maximum =
            try_ffi!(std::str::from_utf8(maximum).map_err(|_| ERROR_COMPATIBILITY_MISMATCH));
        let compatibility = try_ffi!(
            extension::compatibility(
                kmip_min_major,
                kmip_min_minor,
                kmip_max_major,
                kmip_max_minor,
                minimum,
                maximum,
            )
            .map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Compatibility,
            HandleValue::Compatibility(compatibility),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_create(
    first_tag: u32,
    out_path: *mut *mut kmipkit_ttlv_path_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_path);
        let tag = try_ffi!(RawTag::new(first_tag).map_err(model_status));
        let tag = try_ffi!(tag.try_checked().map_err(model_status));
        let path = try_ffi!(extension::ttlv_path(tag).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(Kind::Path, HandleValue::Path(path)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_path_with_child_tag(
    path: *mut kmipkit_ttlv_path_t,
    tag: u32,
    out_path: *mut *mut kmipkit_ttlv_path_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_path);
        let path = try_ffi!(owned_payload(path, Kind::Path));
        let HandleValue::Path(path) = path else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let tag = try_ffi!(RawTag::new(tag).map_err(model_status));
        let tag = try_ffi!(tag.try_checked().map_err(model_status));
        let path = try_ffi!(extension::with_child_tag(path, tag).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(Kind::Path, HandleValue::Path(path)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_discriminator_create(
    path: *mut kmipkit_ttlv_path_t,
    scalar_value: *mut kmipkit_ttlv_value_t,
    out_discriminator: *mut *mut kmipkit_extension_discriminator_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_discriminator);
        let _ = read_handle!(path, Path);
        let _ = read_handle!(scalar_value, Value);
        let path = try_ffi!(consume_owner(path, Kind::Path));
        let scalar = try_ffi!(consume_owner(scalar_value, Kind::Value));
        let path = try_ffi!(unwrap_payload(path));
        let scalar = try_ffi!(unwrap_payload(scalar));
        let HandleValue::Path(path) = path else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let HandleValue::Value(scalar) = scalar else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let discriminator =
            try_ffi!(extension::discriminator(path, scalar).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(
            Kind::Discriminator,
            HandleValue::Discriminator(discriminator),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_scalar(
    item_type: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_schema);
        let item_type = try_ffi!(item_type_from_u32(item_type));
        let schema = try_ffi!(extension::scalar(item_type).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(Kind::Schema, HandleValue::Schema(schema)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_required(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    ffi_entrypoint(|| child_rule_export(tag, schema, out_rule, extension::required))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_optional(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    ffi_entrypoint(|| child_rule_export(tag, schema, out_rule, extension::optional))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_child_rule_repeated(
    tag: u32,
    schema: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
) -> i32 {
    ffi_entrypoint(|| child_rule_export(tag, schema, out_rule, extension::repeated))
}

fn child_rule_export(
    raw_tag: u32,
    schema_pointer: *mut kmipkit_extension_schema_t,
    out_rule: *mut *mut kmipkit_extension_child_rule_t,
    build: fn(
        Tag,
        extension::ExtensionSchema,
    ) -> Result<extension::ExtensionChildRule, ProtocolError>,
) -> i32 {
    let out = out_slot!(out_rule);
    let schema_handle = read_handle!(schema_pointer, Schema);
    let HandleValue::Schema(schema) = &schema_handle.value else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let tag = try_ffi!(RawTag::new(raw_tag).map_err(model_status));
    let tag = try_ffi!(tag.try_checked().map_err(model_status));
    let rule = try_ffi!(build(tag, schema.clone()).map_err(|e| protocol_status(&e)));
    *out = try_ffi!(try_make_handle(
        Kind::ChildRule,
        HandleValue::ChildRule(Box::new(rule))
    ));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_order_constraint_create(
    before_tag: u32,
    after_tag: u32,
    out_constraint: *mut *mut kmipkit_extension_order_constraint_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_constraint);
        let before = try_ffi!(RawTag::new(before_tag).map_err(model_status));
        let before = try_ffi!(before.try_checked().map_err(model_status));
        let after = try_ffi!(RawTag::new(after_tag).map_err(model_status));
        let after = try_ffi!(after.try_checked().map_err(model_status));
        let constraint = try_ffi!(
            extension::extension_order_constraint(before, after).map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::OrderConstraint,
            HandleValue::OrderConstraint(constraint),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_structure(
    children: *mut *mut kmipkit_extension_child_rule_t,
    child_count: u64,
    order_constraints: *mut *mut kmipkit_extension_order_constraint_t,
    order_constraint_count: u64,
    preserve_undeclared_children: u8,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_schema);
        if preserve_undeclared_children > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let children = try_ffi!(input_handle_array(
            children,
            child_count,
            MAX_CHILD_RULES,
            Kind::ChildRule,
            |handle| match &handle.value {
                HandleValue::ChildRule(rule) => Ok((**rule).clone()),
                _ => Err(ERROR_INVALID_INPUT),
            },
        ));
        let order_constraints = try_ffi!(input_handle_array(
            order_constraints,
            order_constraint_count,
            MAX_ORDER_CONSTRAINTS,
            Kind::OrderConstraint,
            |handle| match &handle.value {
                HandleValue::OrderConstraint(constraint) => Ok(*constraint),
                _ => Err(ERROR_INVALID_INPUT),
            },
        ));
        let schema = try_ffi!(
            extension::structure(
                children,
                order_constraints,
                preserve_undeclared_children == 1
            )
            .map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(Kind::Schema, HandleValue::Schema(schema)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_minimum_length(
    schema: *mut kmipkit_extension_schema_t,
    bytes: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_minimum_length(schema, bytes)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_maximum_length(
    schema: *mut kmipkit_extension_schema_t,
    bytes: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_maximum_length(schema, bytes)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_signed_numeric_range(
    schema: *mut kmipkit_extension_schema_t,
    minimum: i64,
    maximum: i64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_signed_range(schema, minimum, maximum)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_unsigned_numeric_range(
    schema: *mut kmipkit_extension_schema_t,
    minimum: u64,
    maximum: u64,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_unsigned_range(schema, minimum, maximum)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_allowed_enumeration(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_allowed_enumeration(schema, value)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_allowed_bit_mask(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_allowed_bit_mask(schema, value)
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_schema_required_bit_mask(
    schema: *mut kmipkit_extension_schema_t,
    value: u32,
    out_schema: *mut *mut kmipkit_extension_schema_t,
) -> i32 {
    ffi_entrypoint(|| {
        schema_transform(schema, out_schema, |schema| {
            extension::with_required_bit_mask(schema, value)
        })
    })
}

fn schema_transform(
    pointer: *mut kmipkit_extension_schema_t,
    out_schema: *mut *mut kmipkit_extension_schema_t,
    transform: impl FnOnce(
        extension::ExtensionSchema,
    ) -> Result<extension::ExtensionSchema, ProtocolError>,
) -> i32 {
    let out = out_slot!(out_schema);
    let schema = try_ffi!(owned_payload(pointer, Kind::Schema));
    let HandleValue::Schema(schema) = schema else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let schema = try_ffi!(transform(schema).map_err(|e| protocol_status(&e)));
    *out = try_ffi!(try_make_handle(Kind::Schema, HandleValue::Schema(schema)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_create(
    identity: *mut kmipkit_extension_identity_t,
    compatibility: *mut kmipkit_extension_compatibility_t,
    discriminator: *mut kmipkit_extension_discriminator_t,
    schema: *mut kmipkit_extension_schema_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_definition);
        let identity = read_handle!(identity, Identity);
        let compatibility = read_handle!(compatibility, Compatibility);
        let discriminator = read_handle!(discriminator, Discriminator);
        let schema = read_handle!(schema, Schema);
        let (
            HandleValue::Identity(identity),
            HandleValue::Compatibility(compatibility),
            HandleValue::Discriminator(discriminator),
            HandleValue::Schema(schema),
        ) = (
            &identity.value,
            &compatibility.value,
            &discriminator.value,
            &schema.value,
        )
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        // Discriminator scalar data is secret-bearing; the protocol helper makes a zeroizing copy.
        let discriminator = try_ffi!(
            extension::clone_extension_discriminator(discriminator)
                .map_err(|e| protocol_status(&e))
        );
        let definition = try_ffi!(
            extension::extension_definition(
                identity.clone(),
                *compatibility,
                discriminator,
                schema.clone(),
            )
            .map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Definition,
            HandleValue::Definition(Box::new(definition)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_v1_default_values(
    out_limits: *mut kmipkit_extension_registry_limits_v1_t,
) -> i32 {
    ffi_entrypoint(|| {
        if let Err(status) = validate_versioned_limits_header(out_limits) {
            return record_ffi_error(status);
        }
        let limits = extension::defaults();
        // SAFETY: validation established that the caller supplied a writable v1-sized output.
        unsafe {
            (*out_limits).max_definitions = limits.max_definitions();
            (*out_limits).max_schema_nodes = limits.max_schema_nodes();
            (*out_limits).max_child_rules_per_structure = limits.max_child_rules_per_structure();
            (*out_limits).max_text_bytes_per_field = limits.max_text_bytes_per_field();
            (*out_limits).max_registry_text_bytes = limits.max_registry_text_bytes();
            (*out_limits).max_discriminator_scalar_bytes = limits.max_discriminator_scalar_bytes();
            (*out_limits).max_total_discriminator_scalar_bytes =
                limits.max_total_discriminator_scalar_bytes();
            (*out_limits).max_constraint_members_per_rule =
                limits.max_constraint_members_per_rule();
            (*out_limits).max_total_constraint_members = limits.max_total_constraint_members();
            (*out_limits).max_payload_index_records = limits.max_payload_index_records();
            (*out_limits).max_lookup_comparisons = limits.max_lookup_comparisons();
            (*out_limits).max_depth = limits.max_depth();
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_v1_validate(
    limits: *const kmipkit_extension_registry_limits_v1_t,
) -> i32 {
    ffi_entrypoint(|| match limits_from_versioned(limits) {
        Ok(_) => SUCCESS,
        Err(status) => record_ffi_error(status),
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_create_v1(
    definitions: *mut *mut kmipkit_extension_definition_t,
    definition_count: u64,
    limits: *const kmipkit_extension_registry_limits_v1_t,
    out_registry: *mut *mut kmipkit_client_extension_registry_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_registry);
        let limits = try_ffi!(limits_from_versioned(limits));
        let pointers = try_ffi!(input_handle_pointers(
            definitions,
            definition_count,
            limits.max_definitions(),
        ));
        let copied = try_ffi!(copy_registry_definitions(pointers, limits, |definition| {
            #[cfg(test)]
            REGISTRY_DEFINITION_CLONE_COUNT.with(|count| count.set(count.get() + 1));
            extension::clone_extension_definition(definition)
                .map_err(|error| protocol_status(&error))
        },));
        let registry = try_ffi!(
            client_extension::client_extension_registry(copied, limits)
                .map_err(|error| client_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Registry,
            HandleValue::Registry(Arc::new(registry))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_default_values(
    out_max_definitions: *mut u64,
    out_max_schema_nodes: *mut u64,
    out_max_child_rules_per_structure: *mut u64,
    out_max_text_bytes_per_field: *mut u64,
    out_max_registry_text_bytes: *mut u64,
    out_max_discriminator_scalar_bytes: *mut u64,
    out_max_total_discriminator_scalar_bytes: *mut u64,
    out_max_constraint_members_per_rule: *mut u64,
    out_max_total_constraint_members: *mut u64,
    out_max_payload_index_records: *mut u64,
    out_max_lookup_comparisons: *mut u64,
    out_max_depth: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let outputs = [
            out_max_definitions,
            out_max_schema_nodes,
            out_max_child_rules_per_structure,
            out_max_text_bytes_per_field,
            out_max_registry_text_bytes,
            out_max_discriminator_scalar_bytes,
            out_max_total_discriminator_scalar_bytes,
            out_max_constraint_members_per_rule,
            out_max_total_constraint_members,
            out_max_payload_index_records,
            out_max_lookup_comparisons,
            out_max_depth,
        ];
        if outputs.iter().any(|output| output.is_null()) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let limits = extension::defaults();
        let values = [
            limits.max_definitions(),
            limits.max_schema_nodes(),
            limits.max_child_rules_per_structure(),
            limits.max_text_bytes_per_field(),
            limits.max_registry_text_bytes(),
            limits.max_discriminator_scalar_bytes(),
            limits.max_total_discriminator_scalar_bytes(),
            limits.max_constraint_members_per_rule(),
            limits.max_total_constraint_members(),
            limits.max_payload_index_records(),
            limits.max_lookup_comparisons(),
            limits.max_depth(),
        ];
        for (output, value) in outputs.into_iter().zip(values) {
            // SAFETY: all output pointers were checked non-null and are writable C output slots.
            unsafe { *output = value };
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_registry_limits_validate(
    max_definitions: u64,
    max_schema_nodes: u64,
    max_child_rules_per_structure: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
    max_constraint_members_per_rule: u64,
    max_total_constraint_members: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
) -> i32 {
    ffi_entrypoint(|| {
        match limits_from_fields([
            max_definitions,
            max_schema_nodes,
            max_child_rules_per_structure,
            max_text_bytes_per_field,
            max_registry_text_bytes,
            max_discriminator_scalar_bytes,
            max_total_discriminator_scalar_bytes,
            max_constraint_members_per_rule,
            max_total_constraint_members,
            max_payload_index_records,
            max_lookup_comparisons,
            max_depth,
        ]) {
            Ok(_) => SUCCESS,
            Err(status) => record_ffi_error(status),
        }
    })
}

#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub extern "C" fn kmipkit_client_extension_registry_create(
    definitions: *mut *mut kmipkit_extension_definition_t,
    definition_count: u64,
    max_definitions: u64,
    max_schema_nodes: u64,
    max_child_rules_per_structure: u64,
    max_text_bytes_per_field: u64,
    max_registry_text_bytes: u64,
    max_discriminator_scalar_bytes: u64,
    max_total_discriminator_scalar_bytes: u64,
    max_constraint_members_per_rule: u64,
    max_total_constraint_members: u64,
    max_payload_index_records: u64,
    max_lookup_comparisons: u64,
    max_depth: u64,
    out_registry: *mut *mut kmipkit_client_extension_registry_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_registry);
        let limits = try_ffi!(limits_from_fields([
            max_definitions,
            max_schema_nodes,
            max_child_rules_per_structure,
            max_text_bytes_per_field,
            max_registry_text_bytes,
            max_discriminator_scalar_bytes,
            max_total_discriminator_scalar_bytes,
            max_constraint_members_per_rule,
            max_total_constraint_members,
            max_payload_index_records,
            max_lookup_comparisons,
            max_depth,
        ]));
        let pointers = try_ffi!(input_handle_pointers(
            definitions,
            definition_count,
            max_definitions,
        ));
        let copied = try_ffi!(copy_registry_definitions(pointers, limits, |definition| {
            #[cfg(test)]
            REGISTRY_DEFINITION_CLONE_COUNT.with(|count| count.set(count.get() + 1));
            extension::clone_extension_definition(definition)
                .map_err(|error| protocol_status(&error))
        },));
        let registry = try_ffi!(
            client_extension::client_extension_registry(copied, limits)
                .map_err(|error| client_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Registry,
            HandleValue::Registry(Arc::new(registry))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_create(
    extension_registry: *mut kmipkit_client_extension_registry_t,
    out_configuration: *mut *mut kmipkit_client_configuration_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_configuration);
        let registry = try_ffi!(owned_payload(extension_registry, Kind::Registry));
        let HandleValue::Registry(registry) = registry else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let configuration = client_extension::ClientConfiguration::from_shared_registry(registry);
        *out = try_ffi!(try_make_handle(
            Kind::Configuration,
            HandleValue::Configuration(configuration),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_configuration_extension_registry(
    configuration: *mut kmipkit_client_configuration_t,
    out_registry: *mut *mut kmipkit_client_extension_registry_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_registry);
        let configuration = read_handle!(configuration, Configuration);
        let HandleValue::Configuration(configuration) = &configuration.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let registry = configuration.shared_extension_registry();
        *out = try_ffi!(try_make_handle(
            Kind::Registry,
            HandleValue::Registry(registry)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_count(
    registry: *mut kmipkit_client_extension_registry_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let count = scalar_out!(out_count);
        let registry = try_ffi!(reference_registry(registry));
        *count = client_extension::definition_count(&registry);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_at(
    registry: *mut kmipkit_client_extension_registry_t,
    index: u64,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_definition);
        let registry = try_ffi!(reference_registry(registry));
        let Some(index) = usize::try_from(index).ok() else {
            return SUCCESS;
        };
        if client_extension::definition_at(&registry, index).is_some() {
            *out = try_ffi!(try_make_handle(
                Kind::Definition,
                HandleValue::DefinitionRef {
                    registry: Arc::clone(&registry),
                    index,
                },
            ));
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_definition_for_identity(
    registry: *mut kmipkit_client_extension_registry_t,
    identity: *mut kmipkit_extension_identity_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_definition);
        let registry = try_ffi!(reference_registry(registry));
        let identity_handle = read_handle!(identity, Identity);
        let HandleValue::Identity(identity) = &identity_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let found = (0..client_extension::definition_count(&registry)).find(|index| {
            usize::try_from(*index)
                .ok()
                .and_then(|position| client_extension::definition_at(&registry, position))
                .is_some_and(|definition| definition.identity_ref() == identity)
        });
        if let Some(index) = found.and_then(|value| usize::try_from(value).ok()) {
            *out = try_ffi!(try_make_handle(
                Kind::Definition,
                HandleValue::DefinitionRef {
                    registry: Arc::clone(&registry),
                    index,
                },
            ));
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_validate(
    registry: *mut kmipkit_client_extension_registry_t,
    identity: *mut kmipkit_extension_identity_t,
    value: *mut kmipkit_ttlv_structure_t,
    limits: *mut kmipkit_codec_limits_t,
    out_value: *mut *mut kmipkit_registered_extension_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let registry = try_ffi!(reference_registry(registry));
        let identity_handle = read_handle!(identity, Identity);
        let structure_handle = read_handle!(value, Structure);
        let limits_handle = read_handle!(limits, CodecLimits);
        let (
            HandleValue::Identity(identity),
            HandleValue::Structure(value),
            HandleValue::CodecLimits(limits),
        ) = (
            &identity_handle.value,
            &structure_handle.value,
            &limits_handle.value,
        )
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(clone_structure(value));
        let registered = try_ffi!(
            client_extension::validate_extension_value(&registry, identity.clone(), value, limits,)
                .map_err(|error| client_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Registered,
            HandleValue::Registered(registered)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_extension_registry_inspect(
    registry: *mut kmipkit_client_extension_registry_t,
    vendor_identifier_data: *const u8,
    vendor_identifier_length: u64,
    value: *mut kmipkit_ttlv_structure_t,
    out_value: *mut *mut kmipkit_extension_recognition_t,
    limits: *mut kmipkit_codec_limits_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let registry = try_ffi!(reference_registry(registry));
        let vendor = try_ffi!(input_span(
            vendor_identifier_data,
            vendor_identifier_length,
            client_extension::limits(&registry).max_text_bytes_per_field()
        ));
        let vendor = try_ffi!(std::str::from_utf8(vendor).map_err(|_| ERROR_INVALID_INPUT));
        let structure_handle = read_handle!(value, Structure);
        let limits_handle = read_handle!(limits, CodecLimits);
        let (HandleValue::Structure(value), HandleValue::CodecLimits(limits)) =
            (&structure_handle.value, &limits_handle.value)
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(clone_structure(value));
        let recognition = try_ffi!(
            client_extension::inspect(&registry, vendor, value, limits)
                .map_err(|error| client_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Recognition,
            HandleValue::Recognition(recognition)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_validate(
    definition: *mut kmipkit_extension_definition_t,
    value: *mut kmipkit_ttlv_structure_t,
    out_value: *mut *mut kmipkit_validated_extension_value_t,
    limits: *mut kmipkit_codec_limits_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let definition_handle = read_handle!(definition, Definition);
        let definition = try_ffi!(definition_from_handle(&definition_handle));
        let value_handle = read_handle!(value, Structure);
        let limits_handle = read_handle!(limits, CodecLimits);
        let (HandleValue::Structure(value), HandleValue::CodecLimits(limits)) =
            (&value_handle.value, &limits_handle.value)
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(clone_structure(value));
        let validated = try_ffi!(
            extension::validate(definition, value, limits).map_err(|error| protocol_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Validated,
            HandleValue::Validated(validated)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_request_message_extension_create(
    value: *mut kmipkit_registered_extension_value_t,
    criticality_indicator: u8,
    out_extension: *mut *mut kmipkit_client_request_message_extension_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_extension);
        let _ = read_handle!(value, Registered);
        let value = try_ffi!(owned_payload(value, Kind::Registered));
        let HandleValue::Registered(value) = value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if criticality_indicator > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let extension = try_ffi!(
            client_extension::client_request_message_extension(value, criticality_indicator == 1,)
                .map_err(|error| client_status(&error))
        );
        *out = try_ffi!(try_make_handle(
            Kind::MessageExtension,
            HandleValue::MessageExtension(extension),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_with_extension(
    item: *mut kmipkit_client_batch_item_t,
    extension: *mut kmipkit_client_request_message_extension_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let _ = read_handle!(item, BatchItem);
        let _ = read_handle!(extension, MessageExtension);
        let item = try_ffi!(consume_owner(item, Kind::BatchItem));
        let extension = try_ffi!(consume_owner(extension, Kind::MessageExtension));
        let item = try_ffi!(unwrap_payload(item));
        let extension = try_ffi!(unwrap_payload(extension));
        let (HandleValue::BatchItem(item), HandleValue::MessageExtension(extension)) =
            (item, extension)
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(item.with_extension(extension))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_with_unique_batch_item_id(
    item: *mut kmipkit_client_batch_item_t,
    id_data: *const u8,
    id_length: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let _ = read_handle!(item, BatchItem);
        let id = try_ffi!(input_span(id_data, id_length, MAX_FFI_INPUT_BYTES,));
        let item = try_ffi!(consume_owner(item, Kind::BatchItem));
        let item = try_ffi!(unwrap_payload(item));
        let HandleValue::BatchItem(item) = item else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(item.with_unique_batch_item_id(id.to_vec()))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_discover_versions(
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::discover_versions())),
        ));
        SUCCESS
    })
}

/// Creates an Interop request from KMIP 2.1 §6.1.26, Table 241.
///
/// The Function Enumeration is open and the Identifier is a UTF-8 Text String.
/// The identifier is retained only in the request's redacted, zeroizing secret type.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_interop(
    interop_function: u32,
    interop_identifier_data: *const u8,
    interop_identifier_length: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(input_span(
            interop_identifier_data,
            interop_identifier_length,
            MAX_FFI_INPUT_BYTES,
        ));
        let identifier = try_ffi!(std::str::from_utf8(identifier).map_err(|_| ERROR_INVALID_INPUT));
        let request = InteropRequest::new(interop_function, SecretText::new(identifier.to_owned()));
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::interop(
                request,
            )))),
        ));
        SUCCESS
    })
}

/// Creates a typed PKCS#11 batch item from KMIP 2.1 §6.1.37, Table 273.
///
/// Function is required. The optional Interface, Correlation Value, and Input Parameters retain
/// their presence independently; correlation bytes are copied into a redacted, zeroizing owner.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_pkcs11(
    function: *mut kmipkit_ttlv_structure_t,
    interface: *mut kmipkit_ttlv_structure_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    input_parameters: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let Some(function) = (try_ffi!(copy_structure_handle(function, true))) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let interface = try_ffi!(copy_structure_handle(interface, false));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let input = try_ffi!(copy_structure_handle(input_parameters, false));
        let mut request = Pkcs11Request::new(function);
        if let Some(interface) = interface {
            request = request.with_interface(interface);
        }
        if let Some(correlation) = correlation {
            request = request.with_correlation_value(correlation);
        }
        if let Some(input) = input {
            request = request.with_input_parameters(input);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::pkcs11(request)));
        SUCCESS
    })
}

/// Creates a KMIP v2.1 §6.1.29, Table 250 Log request.
///
/// The caller must not submit secrets to the server log. The message is
/// validated as UTF-8 and stored in the redacted, zeroizing secret type.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_log(
    log_message_data: *const u8,
    log_message_length: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let message = try_ffi!(input_span(
            log_message_data,
            log_message_length,
            MAX_FFI_INPUT_BYTES,
        ));
        let message = try_ffi!(std::str::from_utf8(message).map_err(|_| ERROR_INVALID_INPUT));
        let request = LogRequest::new(SecretText::new(message.to_owned()));
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::log(request)))),
        ));
        SUCCESS
    })
}

/// Creates a KMIP 2.1 §6.1.57 Validate request from ordered certificate and identifier lists.
///
/// Certificate structures and identifier records are borrowed and copied. A zero count accepts
/// a null array, and omission of the validity date is preserved for server-side evaluation.
/// Input array storage and referenced identifier text bytes must not overlap `out_item`.
/// Overlapping ranges are rejected before either region is written.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_validate(
    certificates: *mut *mut kmipkit_ttlv_structure_t,
    certificate_count: u64,
    identifiers: *const kmipkit_unique_identifier_v1_t,
    identifier_count: u64,
    payload_extensions: *mut *mut kmipkit_ttlv_item_t,
    payload_extension_count: u64,
    has_validity_date: u8,
    validity_date: i64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        try_ffi!(validate_validate_input_aliases(
            out_item,
            certificates,
            certificate_count,
            identifiers,
            identifier_count,
            payload_extensions,
            payload_extension_count,
        ));
        let out = out_slot!(out_item);
        if has_validity_date > 1 || (has_validity_date == 0 && validity_date != 0) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let Ok(certificate_count) = usize::try_from(certificate_count) else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        let Ok(identifier_count) = usize::try_from(identifier_count) else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        let Ok(payload_extension_count) = usize::try_from(payload_extension_count) else {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        };
        // SAFETY: the preflight checked the array pointer/count and rejected output overlap.
        let identifier_records = if identifier_count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(identifiers, identifier_count) }
        };
        // SAFETY: non-null arrays are checked above and both counts are capped before reading.
        let certificate_handles = if certificate_count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(certificates, certificate_count) }
        };
        // SAFETY: non-null arrays are checked above and the combined count is capped before reading.
        let extension_handles = if payload_extension_count == 0 {
            &[][..]
        } else {
            unsafe { std::slice::from_raw_parts(payload_extensions, payload_extension_count) }
        };

        let default_limits = CodecLimits::defaults();
        let mut total_encoded_input_bytes = 0_u64;
        let mut total_input_elements = 1_u64;
        let mut total_identifier_bytes = 0_u64;
        let certificates = try_ffi!(copy_validate_certificates(
            certificate_handles,
            &default_limits,
            &mut total_encoded_input_bytes,
            &mut total_input_elements,
        ));
        let identifiers = try_ffi!(copy_validate_identifiers(
            identifier_records,
            &default_limits,
            &mut total_identifier_bytes,
            &mut total_encoded_input_bytes,
            &mut total_input_elements,
        ));
        let extensions = try_ffi!(copy_validate_extensions(
            extension_handles,
            &default_limits,
            &mut total_encoded_input_bytes,
            &mut total_input_elements,
        ));
        let mut request = ValidateRequest::new();
        for certificate in certificates {
            request = request.with_certificate(certificate);
        }
        for identifier in identifiers {
            request = request.with_unique_identifier(identifier);
        }
        for extension in extensions {
            request = request.with_extension(extension);
        }
        if has_validity_date == 1 {
            try_ffi!(accumulate_validate_input_elements(
                &mut total_input_elements,
                1,
                &default_limits,
            ));
            try_ffi!(accumulate_validate_input_bytes(
                &mut total_encoded_input_bytes,
                16,
            ));
            request = request.with_validity_date(validity_date);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Validate(request)));
        SUCCESS
    })
}

fn validate_input_overlaps_output<T>(
    output: *mut *mut kmipkit_client_batch_item_t,
    input: *const T,
    input_count: u64,
) -> bool {
    let Ok(input_count) = usize::try_from(input_count) else {
        return true;
    };
    let Some(input_bytes) = input_count.checked_mul(std::mem::size_of::<T>()) else {
        return true;
    };
    memory_regions_overlap(
        output.cast(),
        std::mem::size_of::<*mut kmipkit_client_batch_item_t>(),
        input.cast(),
        input_bytes,
    )
}

fn memory_regions_overlap(
    first: *const u8,
    first_length: usize,
    second: *const u8,
    second_length: usize,
) -> bool {
    if first_length == 0 || second_length == 0 {
        return false;
    }
    let first_start = first as usize;
    let second_start = second as usize;
    let Some(first_end) = first_start.checked_add(first_length) else {
        return true;
    };
    let Some(second_end) = second_start.checked_add(second_length) else {
        return true;
    };
    first_start < second_end && second_start < first_end
}

fn validate_validate_input_aliases(
    output: *mut *mut kmipkit_client_batch_item_t,
    certificates: *const *mut kmipkit_ttlv_structure_t,
    certificate_count: u64,
    identifiers: *const kmipkit_unique_identifier_v1_t,
    identifier_count: u64,
    payload_extensions: *const *mut kmipkit_ttlv_item_t,
    payload_extension_count: u64,
) -> FfiResult<()> {
    if output.is_null()
        || validate_input_overlaps_output(output, certificates, certificate_count)
        || validate_input_overlaps_output(output, identifiers, identifier_count)
        || validate_input_overlaps_output(output, payload_extensions, payload_extension_count)
    {
        return Err(ERROR_INVALID_INPUT);
    }
    validate_validate_input_count(certificate_count, identifier_count, payload_extension_count)?;
    if (certificate_count > 0 && certificates.is_null())
        || (identifier_count > 0 && identifiers.is_null())
        || (payload_extension_count > 0 && payload_extensions.is_null())
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let identifier_records = input_array(identifiers, identifier_count, MAX_VALIDATE_INPUT_ITEMS)?;
    if identifier_records.iter().any(|identifier| {
        identifier.kind == crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING
            && validate_input_bytes_overlap_output(
                output,
                identifier.text_data,
                identifier.text_length,
            )
    }) {
        return Err(ERROR_INVALID_INPUT);
    }
    Ok(())
}

fn validate_validate_input_count(
    certificate_count: u64,
    identifier_count: u64,
    payload_extension_count: u64,
) -> FfiResult<()> {
    let total_count = certificate_count
        .checked_add(identifier_count)
        .and_then(|count| count.checked_add(payload_extension_count))
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if total_count > MAX_VALIDATE_INPUT_ITEMS {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok(())
}

fn validate_input_bytes_overlap_output(
    output: *mut *mut kmipkit_client_batch_item_t,
    input: *const u8,
    input_length: u64,
) -> bool {
    let Ok(input_bytes) = usize::try_from(input_length) else {
        return true;
    };
    memory_regions_overlap(
        output.cast(),
        std::mem::size_of::<*mut kmipkit_client_batch_item_t>(),
        input,
        input_bytes,
    )
}

fn copy_validate_certificates(
    handles: &[*mut kmipkit_ttlv_structure_t],
    limits: &CodecLimits,
    total_encoded_bytes: &mut u64,
    total_elements: &mut u64,
) -> FfiResult<Vec<Structure>> {
    let mut certificates = Vec::new();
    certificates
        .try_reserve_exact(handles.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for handle in handles {
        let handle = reference_handle(*handle, Kind::Structure)?;
        let HandleValue::Structure(source) = &handle.value else {
            return Err(ERROR_INVALID_INPUT);
        };
        let (encoded_size, elements) = check_structure_limits(source, limits)?;
        accumulate_validate_input_elements(total_elements, elements, limits)?;
        accumulate_validate_input_bytes(total_encoded_bytes, encoded_size)?;
        certificates.push(source.view().try_clone().map_err(model_status)?);
    }
    Ok(certificates)
}

fn copy_validate_identifiers(
    records: &[kmipkit_unique_identifier_v1_t],
    limits: &CodecLimits,
    total_identifier_bytes: &mut u64,
    total_encoded_bytes: &mut u64,
    total_elements: &mut u64,
) -> FfiResult<Vec<UniqueIdentifier>> {
    let mut identifiers = Vec::new();
    identifiers
        .try_reserve_exact(records.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for record in records {
        let encoded_size = match record.kind {
            crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING => 8_u64
                .checked_add(padded_length(record.text_length)?)
                .ok_or(ERROR_RESOURCE_LIMIT)?,
            crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION
            | crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER => 16,
            _ => return Err(ERROR_INVALID_INPUT),
        };
        accumulate_validate_input_elements(total_elements, 1, limits)?;
        accumulate_validate_input_bytes(total_encoded_bytes, encoded_size)?;
        let Some(identifier) = copy_unique_identifier(ptr::from_ref(record))? else {
            return Err(ERROR_INVALID_INPUT);
        };
        if let UniqueIdentifier::TextString(text) = &identifier {
            let length = u64::try_from(text.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            *total_identifier_bytes = total_identifier_bytes
                .checked_add(length)
                .filter(|total| *total <= MAX_FFI_INPUT_BYTES)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
        }
        identifiers.push(identifier);
    }
    Ok(identifiers)
}

fn copy_validate_extensions(
    handles: &[*mut kmipkit_ttlv_item_t],
    limits: &CodecLimits,
    total_encoded_bytes: &mut u64,
    total_elements: &mut u64,
) -> FfiResult<Vec<Item>> {
    let mut extensions = Vec::new();
    extensions
        .try_reserve_exact(handles.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for handle in handles {
        let handle = reference_handle(*handle, Kind::Item)?;
        let HandleValue::Item(extension) = &handle.value else {
            return Err(ERROR_INVALID_INPUT);
        };
        let (encoded_size, elements) = check_item_limits(extension, limits)?;
        accumulate_validate_input_elements(total_elements, elements, limits)?;
        accumulate_validate_input_bytes(total_encoded_bytes, encoded_size)?;
        extensions.push(clone_item(extension)?);
    }
    Ok(extensions)
}

fn accumulate_validate_input_bytes(total: &mut u64, added: u64) -> FfiResult<()> {
    *total = total
        .checked_add(added)
        .filter(|value| *value <= MAX_FFI_INPUT_BYTES)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    Ok(())
}

fn accumulate_validate_input_elements(
    total: &mut u64,
    added: u64,
    limits: &CodecLimits,
) -> FfiResult<()> {
    let maximum = u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    *total = total
        .checked_add(added)
        .filter(|value| *value <= maximum)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    Ok(())
}

fn copy_structure_handle(
    structure: *mut kmipkit_ttlv_structure_t,
    required: bool,
) -> FfiResult<Option<Structure>> {
    if structure.is_null() {
        return if required {
            Err(ERROR_INVALID_INPUT)
        } else {
            Ok(None)
        };
    }
    copy_required_structure_handle(structure).map(Some)
}

fn copy_required_structure_handle(
    structure: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<Structure> {
    let handle = reference_handle(structure, Kind::Structure)?;
    let HandleValue::Structure(source) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    source.view().try_clone().map_err(model_status)
}

fn copy_attribute_set_handle(
    structure: *mut kmipkit_ttlv_structure_t,
    required: bool,
) -> FfiResult<Option<AttributeSet>> {
    if structure.is_null() {
        return if required {
            Err(ERROR_INVALID_INPUT)
        } else {
            Ok(None)
        };
    }
    let handle = reference_handle(structure, Kind::Structure)?;
    let HandleValue::Structure(source) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let view = source.view();
    let children = view.children();
    let mut items = Vec::new();
    items
        .try_reserve_exact(children.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    for child in children {
        items.push(child.try_clone().map_err(model_status)?);
    }
    AttributeSet::try_new(items)
        .map(Some)
        .map_err(|_| ERROR_INVALID_INPUT)
}

fn copy_unique_identifier(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<Option<UniqueIdentifier>> {
    if identifier.is_null() {
        return Ok(None);
    }
    // SAFETY: the caller promises a readable versioned input record for this call.
    let value = unsafe { identifier.read() };
    if value.struct_size as usize != std::mem::size_of::<kmipkit_unique_identifier_v1_t>()
        || value.abi_version != 1
        || value.reserved != 0
    {
        return Err(ERROR_INVALID_INPUT);
    }
    let identifier = match value.kind {
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_TEXT_STRING => {
            let text = input_span(value.text_data, value.text_length, MAX_FFI_INPUT_BYTES)?;
            let text = std::str::from_utf8(text).map_err(|_| ERROR_INVALID_INPUT)?;
            let mut owned = String::new();
            owned
                .try_reserve_exact(text.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)?;
            owned.push_str(text);
            UniqueIdentifier::TextString(owned)
        }
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_ENUMERATION => {
            UniqueIdentifier::Enumeration(value.enumeration_value)
        }
        crate::KMIPKIT_UNIQUE_IDENTIFIER_KIND_INTEGER => {
            UniqueIdentifier::Integer(value.integer_value)
        }
        _ => return Err(ERROR_INVALID_INPUT),
    };
    Ok(Some(identifier))
}

fn copy_optional_bytes(data: *const u8, length: u64, present: u8) -> FfiResult<Option<Vec<u8>>> {
    if present == 0 {
        return if data.is_null() && length == 0 {
            Ok(None)
        } else {
            Err(ERROR_INVALID_INPUT)
        };
    }
    if present != 1 {
        return Err(ERROR_INVALID_INPUT);
    }
    let bytes = input_span(data, length, MAX_FFI_INPUT_BYTES)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(bytes);
    Ok(Some(owned))
}

fn copy_optional_secret_span(
    span: *const crate::kmipkit_byte_span_v1_t,
) -> FfiResult<Option<SecretBytes>> {
    if span.is_null() {
        return Ok(None);
    }
    // SAFETY: the C caller promises a readable byte-span record for this call.
    let span = unsafe { span.read() };
    let bytes = input_span(span.data, span.length, MAX_FFI_INPUT_BYTES)?;
    let mut owned = Vec::new();
    owned
        .try_reserve_exact(bytes.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    owned.extend_from_slice(bytes);
    Ok(Some(SecretBytes::new(owned)))
}

fn copy_operation_data(value: *mut kmipkit_ttlv_value_t) -> FfiResult<Option<OperationData>> {
    if value.is_null() {
        return Ok(None);
    }
    let handle = reference_handle(value, Kind::Value)?;
    let HandleValue::Value(value) = &handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    value.with_value(|view| match view {
        ValueView::ByteString(bytes) => {
            let mut owned = Vec::new();
            owned
                .try_reserve_exact(bytes.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)?;
            owned.extend_from_slice(bytes);
            Ok(Some(OperationData::ByteString(SecretBytes::new(owned))))
        }
        ValueView::Enumeration(value) => Ok(Some(OperationData::Enumeration(*value))),
        ValueView::Integer(value) => Ok(Some(OperationData::Integer(*value))),
        _ => Err(ERROR_INVALID_INPUT),
    })
}

fn optional_indicator(value: u8) -> FfiResult<Option<bool>> {
    match value {
        0 => Ok(None),
        1 => Ok(Some(false)),
        2 => Ok(Some(true)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

fn with_cryptographic_framing<T>(
    mut request: T,
    correlation: Option<SecretBytes>,
    init_indicator: Option<bool>,
    final_indicator: Option<bool>,
) -> T
where
    T: CryptographicRequestFraming,
{
    if let Some(value) = correlation {
        request = request.with_correlation_value(value);
    }
    if let Some(value) = init_indicator {
        request = request.with_init_indicator(value);
    }
    if let Some(value) = final_indicator {
        request = request.with_final_indicator(value);
    }
    request
}

trait CryptographicRequestFraming: Sized {
    fn with_correlation_value(self, value: SecretBytes) -> Self;
    fn with_init_indicator(self, value: bool) -> Self;
    fn with_final_indicator(self, value: bool) -> Self;
}

macro_rules! impl_cryptographic_request_framing {
    ($($request:ty),+ $(,)?) => {
        $(
            impl CryptographicRequestFraming for $request {
                fn with_correlation_value(self, value: SecretBytes) -> Self {
                    <$request>::with_correlation_value(self, value)
                }

                fn with_init_indicator(self, value: bool) -> Self {
                    <$request>::with_init_indicator(self, value)
                }

                fn with_final_indicator(self, value: bool) -> Self {
                    <$request>::with_final_indicator(self, value)
                }
            }
        )+
    };
}

impl_cryptographic_request_framing!(
    HashRequest,
    MacRequest,
    MacVerifyRequest,
    SignRequest,
    SignatureVerifyRequest,
);

fn copy_optional_identifier(
    identifier: *const kmipkit_unique_identifier_v1_t,
) -> FfiResult<Option<UniqueIdentifier>> {
    if identifier.is_null() {
        return Ok(None);
    }
    copy_unique_identifier(identifier)
}

fn batch_item_handle(request: ClientRequest) -> FfiResult<*mut kmipkit_client_batch_item_t> {
    try_make_handle(
        Kind::BatchItem,
        HandleValue::BatchItem(Box::new(ClientBatchItem::new(request))),
    )
}

/// Creates a typed Check request from KMIP 2.1 §6.1.7, Table 183.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_check(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_usage_limits_count: u8,
    usage_limits_count: i64,
    has_cryptographic_usage_mask: u8,
    cryptographic_usage_mask: i32,
    has_lease_time: u8,
    lease_time: u32,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let usage_limits_count = match has_usage_limits_count {
            0 => None,
            1 => Some(usage_limits_count),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let cryptographic_usage_mask = match has_cryptographic_usage_mask {
            0 => None,
            1 => Some(cryptographic_usage_mask),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let lease_time = match has_lease_time {
            0 => None,
            1 => Some(lease_time),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let mut request = CheckRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(count) = usage_limits_count {
            request = request.with_usage_limits_count(count);
        }
        if let Some(mask) = cryptographic_usage_mask {
            request = request.with_cryptographic_usage_mask(mask);
        }
        if let Some(lease_time) = lease_time {
            request = request.with_lease_time(lease_time);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::check(request)));
        SUCCESS
    })
}

/// Creates a typed Delegated Login request from KMIP 2.1 §6.1.12, Table 199.
///
/// `lease_time_kind` is 0 for absent, 1 for Interval, or 2 for Date Time.
/// Rights is required; Usage Limits is copied when supplied.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_delegated_login(
    rights: *mut kmipkit_ttlv_structure_t,
    lease_time_kind: u32,
    lease_time_value: i64,
    has_request_count: u8,
    request_count: i32,
    usage_limits: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let rights = match copy_structure_handle(rights, true) {
            Ok(Some(rights)) => rights,
            Ok(None) => return record_ffi_error(ERROR_INVALID_INPUT), // COVERAGE-EXCLUDE: Required handles return Err when null and Some when valid.
            Err(status) => return record_ffi_error(status),
        };
        let lease_time = match lease_time_kind {
            0 if lease_time_value == 0 => None,
            1 => match u32::try_from(lease_time_value) {
                Ok(value) => Some(DelegatedLoginLeaseTime::Interval(value)),
                Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            },
            2 => Some(DelegatedLoginLeaseTime::DateTime(lease_time_value)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let request_count = match (has_request_count, request_count) {
            (0, 0) => None,
            (1, value) => Some(value),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let usage_limits = match copy_structure_handle(usage_limits, false) {
            Ok(value) => value,
            Err(status) => return record_ffi_error(status),
        };
        let mut request = DelegatedLoginRequest::new(rights);
        if let Some(lease_time) = lease_time {
            request = match lease_time {
                DelegatedLoginLeaseTime::Interval(value) => request.with_lease_time_interval(value),
                DelegatedLoginLeaseTime::DateTime(value) => {
                    request.with_lease_time_date_time(value)
                }
            };
        }
        if let Some(count) = request_count {
            request = request.with_request_count(count);
        }
        if let Some(usage_limits) = usage_limits {
            request = request.with_usage_limits(usage_limits);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::delegated_login(request)));
        SUCCESS
    })
}

/// Creates a typed Login request from KMIP 2.1 §6.1.30, Table 253.
///
/// `lease_time_kind` is 0 for absent, 1 for Interval, or 2 for Date Time.
/// Usage Limits is copied when supplied.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_login(
    lease_time_kind: u32,
    lease_time_value: i64,
    has_request_count: u8,
    request_count: i32,
    usage_limits: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let lease_time = match lease_time_kind {
            0 if lease_time_value == 0 => None,
            1 => match u32::try_from(lease_time_value) {
                Ok(value) => Some(LoginLeaseTime::Interval(value)),
                Err(_) => return record_ffi_error(ERROR_INVALID_INPUT),
            },
            2 => Some(LoginLeaseTime::DateTime(lease_time_value)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let request_count = match (has_request_count, request_count) {
            (0, 0) => None,
            (1, value) => Some(value),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let usage_limits = match copy_structure_handle(usage_limits, false) {
            Ok(value) => value,
            Err(status) => return record_ffi_error(status),
        };
        let mut request = LoginRequest::new();
        if let Some(lease_time) = lease_time {
            request = match lease_time {
                LoginLeaseTime::Interval(value) => request.with_lease_time_interval(value),
                LoginLeaseTime::DateTime(value) => request.with_lease_time_date_time(value),
            };
        }
        if let Some(count) = request_count {
            request = request.with_request_count(count);
        }
        if let Some(usage_limits) = usage_limits {
            request = request.with_usage_limits(usage_limits);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::login(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 §6.1.31, Table 256 Logout request.
///
/// The caller supplies the required §7.39, Table 391 Ticket TTLV Structure.
/// Ticket bytes are copied into KMIPKit-owned zeroizing storage and never
/// included in diagnostics.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_logout(
    ticket: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let Some(ticket_structure) = (match copy_structure_handle(ticket, true) {
            Ok(ticket) => ticket,
            Err(status) => return record_ffi_error(status),
        }) else {
            return record_ffi_error(ERROR_INVALID_INPUT); // COVERAGE-EXCLUDE: Required handles return Err when null and Some when valid.
        };
        let Ok(ticket) = Ticket::try_from_ttlv(ticket_structure) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::logout(
            LogoutRequest::new(ticket)
        )));
        SUCCESS
    })
}

/// Creates a typed Certify request and copies the certificate request as sensitive bytes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_certify(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_certificate_request_type: u8,
    certificate_request_type: u32,
    certificate_request_value: *const crate::kmipkit_byte_span_v1_t,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let request_type = match has_certificate_request_type {
            0 => None,
            1 => Some(certificate_request_type),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let request_value = try_ffi!(copy_optional_secret_span(certificate_request_value));
        let attributes = try_ffi!(copy_structure_handle(attributes, false));
        let masks = try_ffi!(copy_structure_handle(protection_storage_masks, false));
        let mut request = CertifyRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(request_type) = request_type {
            request = request.with_certificate_request_type(request_type);
        }
        if let Some(request_value) = request_value {
            request = request.with_certificate_request_value(request_value);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        if let Some(masks) = masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Certify(request)));
        SUCCESS
    })
}

/// Creates a typed Re-certify request and copies certificate request bytes as sensitive data.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_recertify(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    certificate_request_unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_certificate_request_type: u8,
    certificate_request_type: u32,
    certificate_request_value: *const crate::kmipkit_byte_span_v1_t,
    has_offset: u8,
    offset: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let request_identifier = try_ffi!(copy_optional_identifier(
            certificate_request_unique_identifier
        ));
        let request_type = match has_certificate_request_type {
            0 => None,
            1 => Some(certificate_request_type),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let request_value = try_ffi!(copy_optional_secret_span(certificate_request_value));
        let offset = match has_offset {
            0 => None,
            1 => Some(offset),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let attributes = try_ffi!(copy_structure_handle(attributes, false));
        let masks = try_ffi!(copy_structure_handle(protection_storage_masks, false));
        let mut request = RecertifyRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(request_identifier) = request_identifier {
            request = request.with_certificate_request_unique_identifier(request_identifier);
        }
        if let Some(request_type) = request_type {
            request = request.with_certificate_request_type(request_type);
        }
        if let Some(request_value) = request_value {
            request = request.with_certificate_request_value(request_value);
        }
        if let Some(offset) = offset {
            request = request.with_offset(offset);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        if let Some(masks) = masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Recertify(request)));
        SUCCESS
    })
}

/// Creates a typed Re-key Key Pair request using KMIP 2.1 §6.1.47, Table 310.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_rekey_key_pair(
    private_key_unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_offset: u8,
    offset: u32,
    common_attributes: *mut kmipkit_ttlv_structure_t,
    private_key_attributes: *mut kmipkit_ttlv_structure_t,
    public_key_attributes: *mut kmipkit_ttlv_structure_t,
    common_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    private_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    public_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(private_key_unique_identifier));
        let offset = match has_offset {
            0 => None,
            1 => Some(offset),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let common_attributes = try_ffi!(copy_attribute_set_handle(common_attributes, false));
        let private_key_attributes =
            try_ffi!(copy_attribute_set_handle(private_key_attributes, false));
        let public_key_attributes =
            try_ffi!(copy_attribute_set_handle(public_key_attributes, false));
        let common_masks = try_ffi!(copy_structure_handle(
            common_protection_storage_masks,
            false
        ));
        let private_masks = try_ffi!(copy_structure_handle(
            private_protection_storage_masks,
            false
        ));
        let public_masks = try_ffi!(copy_structure_handle(
            public_protection_storage_masks,
            false
        ));
        let mut request = RekeyKeyPairRequest::new();
        if let Some(value) = identifier {
            request = request.with_private_key_unique_identifier(value);
        }
        if let Some(value) = offset {
            request = request.with_offset(value);
        }
        if let Some(value) = common_attributes {
            request = request.with_common_attributes(value);
        }
        if let Some(value) = private_key_attributes {
            request = request.with_private_key_attributes(value);
        }
        if let Some(value) = public_key_attributes {
            request = request.with_public_key_attributes(value);
        }
        if let Some(value) = common_masks {
            request = request.with_common_protection_storage_masks(value);
        }
        if let Some(value) = private_masks {
            request = request.with_private_protection_storage_masks(value);
        }
        if let Some(value) = public_masks {
            request = request.with_public_protection_storage_masks(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::RekeyKeyPair(request)));
        SUCCESS
    })
}

/// Creates a typed Re-key request using the fields in §6.1.46, Table 305.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_rekey(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_offset: u8,
    offset: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let offset = match has_offset {
            0 => None,
            1 => Some(offset),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let attributes = try_ffi!(copy_structure_handle(attributes, false));
        let masks = try_ffi!(copy_structure_handle(protection_storage_masks, false));
        let mut request = RekeyRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(offset) = offset {
            request = request.with_offset(offset);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        if let Some(masks) = masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Rekey(request)));
        SUCCESS
    })
}

/// Creates a typed Re-Provision request from KMIP 2.1 §6.1.48, Table 313.
///
/// The optional Certificate Request and Certificate inputs are copied before this call returns.
/// At most one may be supplied; neither may be supplied when the server creates credentials.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_reprovision(
    certificate_request: *mut kmipkit_ttlv_structure_t,
    certificate: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        if !certificate_request.is_null() && !certificate.is_null() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let certificate_request = try_ffi!(copy_structure_handle(certificate_request, false));
        let certificate = try_ffi!(copy_structure_handle(certificate, false));
        let mut request = ReProvisionRequest::new();
        if let Some(value) = certificate_request {
            request = request.with_certificate_request(value);
        }
        if let Some(value) = certificate {
            request = request.with_certificate(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::ReProvision(request)));
        SUCCESS
    })
}

// The ordered Unique Identifier records may contain repeated identifiers.
// Derivation Parameters and Attributes are copied from generic TTLV structures;
// secret derivation byte strings are moved into zeroizing protocol storage.
fn derive_key_request_from_handles(
    object_type: u32,
    unique_identifiers: *const kmipkit_unique_identifier_v1_t,
    unique_identifier_count: u64,
    derivation_method: u32,
    derivation_parameters: *mut kmipkit_ttlv_structure_t,
    attributes: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<DeriveKeyRequest> {
    if unique_identifier_count == 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    if unique_identifier_count > MAX_DERIVE_KEY_SOURCE_IDENTIFIERS {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    if unique_identifiers.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    let Ok(count) = usize::try_from(unique_identifier_count) else {
        return Err(ERROR_RESOURCE_LIMIT);
    };
    // SAFETY: non-null and bounded above; the caller supplies `count` readable ABI records.
    let source_identifiers = unsafe { std::slice::from_raw_parts(unique_identifiers, count) };
    let mut identifiers = Vec::new();
    if identifiers.try_reserve_exact(count).is_err() {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    let mut total_identifier_bytes = 0_u64;
    for identifier in source_identifiers {
        let identifier = match copy_unique_identifier(ptr::from_ref(identifier)) {
            Ok(Some(identifier)) => identifier,
            Ok(None) => return Err(ERROR_INVALID_INPUT),
            Err(status) => return Err(status),
        };
        if let UniqueIdentifier::TextString(value) = &identifier {
            let Ok(length) = u64::try_from(value.len()) else {
                return Err(ERROR_RESOURCE_LIMIT);
            };
            total_identifier_bytes = match total_identifier_bytes.checked_add(length) {
                Some(total) if total <= MAX_FFI_INPUT_BYTES => total,
                _ => return Err(ERROR_RESOURCE_LIMIT),
            };
        }
        identifiers.push(identifier);
    }
    let parameters = derive_key_parameters_from_handle(derivation_parameters)?;
    let Some(attributes) = copy_attribute_set_handle(attributes, true)? else {
        return Err(ERROR_INVALID_INPUT);
    };
    Ok(DeriveKeyRequest::new(
        ObjectType::from_raw(object_type),
        identifiers,
        DerivationMethod::from_raw(derivation_method),
        parameters,
        attributes,
    ))
}

/// Creates a typed Derive Key request from KMIP 2.1 §6.1.14, Table 205.
///
/// The ordered Unique Identifier records may contain repeated identifiers.
/// Derivation Parameters and Attributes are copied from generic TTLV structures;
/// secret derivation byte strings are moved into zeroizing protocol storage.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_derive_key(
    object_type: u32,
    unique_identifiers: *const kmipkit_unique_identifier_v1_t,
    unique_identifier_count: u64,
    derivation_method: u32,
    derivation_parameters: *mut kmipkit_ttlv_structure_t,
    attributes: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = match derive_key_request_from_handles(
            object_type,
            unique_identifiers,
            unique_identifier_count,
            derivation_method,
            derivation_parameters,
            attributes,
        ) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        *out = match batch_item_handle(ClientRequest::DeriveKey(request)) {
            Ok(item) => item,
            Err(status) => return record_ffi_error(status),
        };
        SUCCESS
    })
}
/// Creates a typed Get Usage Allocation request from KMIP 2.1 §6.1.23, Table 232.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_get_usage_allocation(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    usage_limits_count: i64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let mut request = GetUsageAllocationRequest::new(usage_limits_count);
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::get_usage_allocation(
            request
        )));
        SUCCESS
    })
}

/// Creates a typed Obtain Lease request from KMIP 2.1 §6.1.35, Table 268.
/// A null identifier leaves the optional Table 268 field absent.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_obtain_lease(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let mut request = ObtainLeaseRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::obtain_lease(request)));
        SUCCESS
    })
}

/// Creates one owned identifier and its caller-known, local threshold context.
/// The threshold is retained only for pre-send request validation and is never encoded.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_split_key_reference_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    split_key_threshold: u32,
    out_reference: *mut *mut kmipkit_split_key_reference_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_reference);
        let Some(unique_identifier) = try_ffi!(copy_optional_identifier(unique_identifier)) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let reference =
            SplitKeyReference::new(unique_identifier).with_split_key_threshold(split_key_threshold);
        *out = try_ffi!(try_make_handle(
            Kind::SplitKeyReference,
            HandleValue::SplitKeyReference(reference),
        ));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 Join Split Key request from §6.1.27, Table 244.
/// Reference handles are borrowed for this call; their identifier order and repetitions are
/// copied into the request while their individual thresholds remain local validation context.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_join_split_key(
    object_type: u32,
    references: *mut *mut kmipkit_split_key_reference_t,
    reference_count: u64,
    has_secret_data_type: u8,
    secret_data_type: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let references = match copy_join_split_key_references(references, reference_count) {
            Ok(references) => references,
            Err(status) => return record_ffi_error(status),
        };
        if references.is_empty() {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let maximum_threshold = references
            .iter()
            .filter_map(SplitKeyReference::split_key_threshold)
            .max()
            .unwrap_or(0);
        if u64::try_from(references.len())
            .map_or(true, |count| count < u64::from(maximum_threshold))
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let secret_data_type = match has_secret_data_type {
            0 if secret_data_type == 0 => None,
            1 => Some(secret_data_type),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let attributes = match copy_attribute_set_handle(attributes, false) {
            Ok(attributes) => attributes,
            Err(status) => return record_ffi_error(status),
        };
        let protection_storage_masks = match copy_structure_handle(protection_storage_masks, false)
        {
            Ok(masks) => masks,
            Err(status) => return record_ffi_error(status),
        };
        let mut request = JoinSplitKeyRequest::new(ObjectType::from_raw(object_type), references);
        if let Some(value) = secret_data_type {
            request = request.with_secret_data_type(value);
        }
        if let Some(attributes) = attributes {
            request = request.with_attributes(attributes);
        }
        if let Some(masks) = protection_storage_masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::join_split_key(request)));
        SUCCESS
    })
}

fn import_request_from_handles(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    object_type: u32,
    has_replace_existing: u8,
    replace_existing: u8,
    has_key_wrap_type: u8,
    key_wrap_type: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    any_object: *mut kmipkit_ttlv_structure_t,
) -> FfiResult<ImportRequest> {
    let Some(unique_identifier) = copy_optional_identifier(unique_identifier)? else {
        return Err(ERROR_INVALID_INPUT);
    };
    let replace_existing = match has_replace_existing {
        0 if replace_existing == 0 => None,
        1 if replace_existing <= 1 => Some(replace_existing != 0),
        _ => return Err(ERROR_INVALID_INPUT),
    };
    let key_wrap_type = match has_key_wrap_type {
        0 if key_wrap_type == 0 => None,
        1 => Some(KeyWrapType::from_raw(key_wrap_type)),
        _ => return Err(ERROR_INVALID_INPUT),
    };
    let Some(attributes) = copy_attribute_set_handle(attributes, true)? else {
        return Err(ERROR_INVALID_INPUT);
    };
    if any_object.is_null() {
        return Err(ERROR_INVALID_INPUT);
    }
    let object_handle = reference_handle(any_object, Kind::Structure)?;
    let HandleValue::Structure(object_structure) = &object_handle.value else {
        return Err(ERROR_INVALID_INPUT);
    };
    let object_view = object_structure.view();
    if object_view.children().len() != 1 {
        return Err(ERROR_INVALID_INPUT);
    }
    let object = object_view.children()[0]
        .try_clone()
        .map_err(model_status)?;
    let mut request = ImportRequest::new(
        unique_identifier,
        ObjectType::from_raw(object_type),
        attributes,
        object,
    );
    if let Some(value) = replace_existing {
        request = request.with_replace_existing(value);
    }
    if let Some(value) = key_wrap_type {
        request = request.with_key_wrap_type(value);
    }
    Ok(request)
}

/// Creates a typed Import request from KMIP 2.1 §6.1.25, Table 238.
///
/// The Attributes and one-item Any Object structures are borrowed and copied
/// into zeroizing protocol-owned storage. Raw object and selector values are
/// preserved; no local unwrap or object transformation is performed.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_import(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    object_type: u32,
    has_replace_existing: u8,
    replace_existing: u8,
    has_key_wrap_type: u8,
    key_wrap_type: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    any_object: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = match import_request_from_handles(
            unique_identifier,
            object_type,
            has_replace_existing,
            replace_existing,
            has_key_wrap_type,
            key_wrap_type,
            attributes,
            any_object,
        ) {
            Ok(request) => request,
            Err(status) => return record_ffi_error(status),
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::import(request)));
        SUCCESS
    })
}

/// Creates the empty Get Constraints request from KMIP 2.1 §6.1.22, Table 229.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_get_constraints(
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        *out = try_ffi!(batch_item_handle(ClientRequest::GetConstraints(
            GetConstraintsRequest::new(),
        )));
        SUCCESS
    })
}

/// Creates a Set Constraints request from the required KMIP 2.1 §6.1.52 Constraints structure.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_set_constraints(
    constraints: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let constraints = try_ffi!(copy_required_structure_handle(constraints));
        *out = try_ffi!(batch_item_handle(ClientRequest::SetConstraints(
            SetConstraintsRequest::new(constraints),
        )));
        SUCCESS
    })
}

/// Creates a Set Defaults request from optional KMIP 2.1 §6.1.53 Table 328 Defaults Information.
/// A null structure omits the field; a valid empty structure remains explicitly present.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_set_defaults(
    defaults_information: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let defaults_information = try_ffi!(copy_structure_handle(defaults_information, false));
        let mut request = SetDefaultsRequest::new();
        if let Some(defaults_information) = defaults_information {
            request = request.with_defaults_information(defaults_information);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::SetDefaults(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 Export request from its optional Table 217 selectors.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_export(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_key_format_type: u8,
    key_format_type: u32,
    has_key_wrap_type: u8,
    key_wrap_type: u32,
    has_key_compression_type: u8,
    key_compression_type: u32,
    key_wrapping_specification: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let key_format_type = match has_key_format_type {
            0 => None,
            1 => Some(KeyFormatType::from_raw(key_format_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let key_wrap_type = match has_key_wrap_type {
            0 => None,
            1 => Some(KeyWrapType::from_raw(key_wrap_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let key_compression_type = match has_key_compression_type {
            0 => None,
            1 => Some(KeyCompressionType::from_raw(key_compression_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let wrapping_specification =
            try_ffi!(copy_structure_handle(key_wrapping_specification, false));

        let mut request = ExportRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(value) = key_format_type {
            request = request.with_key_format_type(value);
        }
        if let Some(value) = key_wrap_type {
            request = request.with_key_wrap_type(value);
        }
        if let Some(value) = key_compression_type {
            request = request.with_key_compression_type(value);
        }
        if let Some(value) = wrapping_specification {
            request = request.with_key_wrapping_specification(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Export(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 Get request from its optional Table 220 values.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_get(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    has_key_format_type: u8,
    key_format_type: u32,
    has_key_wrap_type: u8,
    key_wrap_type: u32,
    has_key_compression_type: u8,
    key_compression_type: u32,
    key_wrapping_specification: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let key_format_type = match has_key_format_type {
            0 => None,
            1 => Some(KeyFormatType::from_raw(key_format_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let key_wrap_type = match has_key_wrap_type {
            0 => None,
            1 => Some(KeyWrapType::from_raw(key_wrap_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let key_compression_type = match has_key_compression_type {
            0 => None,
            1 => Some(KeyCompressionType::from_raw(key_compression_type)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let wrapping_specification =
            try_ffi!(copy_structure_handle(key_wrapping_specification, false));

        let mut request = GetRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(value) = key_format_type {
            request = request.with_key_format_type(value);
        }
        if let Some(value) = key_wrap_type {
            request = request.with_key_wrap_type(value);
        }
        if let Some(value) = key_compression_type {
            request = request.with_key_compression_type(value);
        }
        if let Some(value) = wrapping_specification {
            request = request.with_key_wrapping_specification(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Get(request)));
        SUCCESS
    })
}

/// Creates a typed KMIP 2.1 Locate request with its required Table 247 Attributes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_locate(
    has_maximum_items: u8,
    maximum_items: i32,
    has_offset_items: u8,
    offset_items: i32,
    has_storage_status_mask: u8,
    storage_status_mask: u32,
    has_object_group_member: u8,
    object_group_member: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let maximum_items = match has_maximum_items {
            0 => None,
            1 => Some(maximum_items),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let offset_items = match has_offset_items {
            0 => None,
            1 => Some(offset_items),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let storage_status_mask = match has_storage_status_mask {
            0 => None,
            1 => Some(StorageStatusMask::from_raw(storage_status_mask)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let object_group_member = match has_object_group_member {
            0 => None,
            1 => Some(ObjectGroupMember::from_raw(object_group_member)),
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let Some(attributes) = try_ffi!(copy_attribute_set_handle(attributes, true)) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };

        let mut request = LocateRequest::new(attributes);
        if let Some(value) = maximum_items {
            request = request.with_maximum_items(value);
        }
        if let Some(value) = offset_items {
            request = request.with_offset_items(value);
        }
        if let Some(value) = storage_status_mask {
            request = request.with_storage_status_mask(value);
        }
        if let Some(value) = object_group_member {
            request = request.with_object_group_member(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Locate(request)));
        SUCCESS
    })
}

/// Creates an empty client-initiated Ping item from KMIP v2.1 §6.1.36.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_ping(
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        *out = try_ffi!(batch_item_handle(ClientRequest::ping()));
        SUCCESS
    })
}

/// Creates a typed client-initiated Query item from KMIP v2.1 §6.1.40.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_query(
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const crate::kmipkit_byte_span_v1_t,
    object_group_count: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let limits = CodecLimits::defaults();
        *out = try_ffi!(query_batch_item_with_limits(
            &limits,
            functions,
            function_count,
            has_object_groups,
            object_groups,
            object_group_count,
        ));
        SUCCESS
    })
}

/// Creates a typed client-initiated Query item after checking its payload budget.
/// The complete Request Message is checked again before transmission.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_query_with_limits(
    limits: *mut kmipkit_codec_limits_t,
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const crate::kmipkit_byte_span_v1_t,
    object_group_count: u64,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let limits = try_ffi!(codec_limits_from_handle(limits));
        *out = try_ffi!(query_batch_item_with_limits(
            &limits,
            functions,
            function_count,
            has_object_groups,
            object_groups,
            object_group_count,
        ));
        SUCCESS
    })
}

fn query_batch_item_with_limits(
    limits: &CodecLimits,
    functions: *const u32,
    function_count: u64,
    has_object_groups: u8,
    object_groups: *const crate::kmipkit_byte_span_v1_t,
    object_group_count: u64,
) -> FfiResult<*mut kmipkit_client_batch_item_t> {
    if has_object_groups > 1 || (has_object_groups == 0 && object_group_count != 0) {
        return Err(ERROR_INVALID_INPUT);
    }

    let function_count_usize = usize::try_from(function_count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    if function_count_usize == 0 {
        return Err(ERROR_INVALID_INPUT);
    }
    let group_count_usize =
        usize::try_from(object_group_count).map_err(|_| ERROR_RESOURCE_LIMIT)?;
    let mut estimated_elements = function_count_usize
        .checked_add(1)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let mut estimated_bytes = 8_usize
        .checked_add(
            function_count_usize
                .checked_mul(16)
                .ok_or(ERROR_RESOURCE_LIMIT)?,
        )
        .ok_or(ERROR_RESOURCE_LIMIT)?;

    let required_depth = if has_object_groups == 1 { 4 } else { 3 };
    if estimated_elements > limits.max_elements()
        || estimated_bytes > limits.max_message_bytes()
        || required_depth > limits.max_structure_depth()
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }

    let spans = if has_object_groups == 1 {
        estimated_elements = estimated_elements
            .checked_add(1)
            .and_then(|count| count.checked_add(group_count_usize))
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        if estimated_elements > limits.max_elements() || 4 > limits.max_structure_depth() {
            return Err(ERROR_RESOURCE_LIMIT);
        }
        Some(input_array(object_groups, object_group_count, u64::MAX)?)
    } else {
        None
    };

    if let Some(spans) = spans {
        estimated_bytes = estimated_bytes.checked_add(8).ok_or(ERROR_RESOURCE_LIMIT)?;
        let mut object_groups_value_length = 0_usize;
        for span in spans {
            u32::try_from(span.length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            let length = usize::try_from(span.length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            let padded_length = length
                .checked_add(7)
                .map(|value| value & !7)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
            let item_bytes = 8_usize
                .checked_add(padded_length)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
            object_groups_value_length = object_groups_value_length
                .checked_add(item_bytes)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
            u32::try_from(object_groups_value_length).map_err(|_| ERROR_RESOURCE_LIMIT)?;
            estimated_bytes = estimated_bytes
                .checked_add(item_bytes)
                .ok_or(ERROR_RESOURCE_LIMIT)?;
        }
    }

    u32::try_from(estimated_bytes.checked_sub(8).ok_or(ERROR_RESOURCE_LIMIT)?)
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;

    if estimated_elements > limits.max_elements() || estimated_bytes > limits.max_message_bytes() {
        return Err(ERROR_RESOURCE_LIMIT);
    }

    let raw_functions = input_array(functions, function_count, function_count)?;
    let mut query_functions = Vec::new();
    query_functions
        .try_reserve_exact(raw_functions.len())
        .map_err(|_| ERROR_RESOURCE_LIMIT)?;
    query_functions.extend(raw_functions.iter().copied().map(QueryFunction::from_raw));

    let groups = if let Some(spans) = spans {
        let mut values = Vec::new();
        values
            .try_reserve_exact(spans.len())
            .map_err(|_| ERROR_RESOURCE_LIMIT)?;
        for span in spans {
            let bytes = input_span(span.data, span.length, u64::MAX)?;
            let text = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
            let mut value = String::new();
            value
                .try_reserve_exact(text.len())
                .map_err(|_| ERROR_RESOURCE_LIMIT)?;
            value.push_str(text);
            values.push(value);
        }
        Some(values)
    } else {
        None
    };

    let mut request = QueryRequest::new(query_functions);
    if let Some(groups) = groups {
        request = request.with_object_groups(groups);
    }
    batch_item_handle(ClientRequest::query(request))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_hash(
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let Some(parameters) = parameters else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let data = try_ffi!(copy_operation_data(data));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let mut request = HashRequest::new(parameters);
        if let Some(data) = data {
            request = request.with_data(data);
        }
        request = with_cryptographic_framing(request, correlation, init_indicator, final_indicator);
        *out = try_ffi!(batch_item_handle(ClientRequest::Hash(request)));
        SUCCESS
    })
}

/// Creates a typed Encrypt request and copies caller-provided secret spans.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_encrypt(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    iv_counter_nonce: *const crate::kmipkit_byte_span_v1_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    authenticated_encryption_additional_data: *const crate::kmipkit_byte_span_v1_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let iv_counter_nonce = try_ffi!(copy_optional_secret_span(iv_counter_nonce));
        let correlation_value = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let additional_data = try_ffi!(copy_optional_secret_span(
            authenticated_encryption_additional_data
        ));

        let mut request = EncryptRequest::new(identifier, data);
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(value) = iv_counter_nonce {
            request = request.with_iv_counter_nonce(value);
        }
        if let Some(value) = correlation_value {
            request = request.with_correlation_value(value);
        }
        if let Some(value) = init_indicator {
            request = request.with_init_indicator(value);
        }
        if let Some(value) = final_indicator {
            request = request.with_final_indicator(value);
        }
        if let Some(value) = additional_data {
            request = request.with_authenticated_encryption_additional_data(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Encrypt(request)));
        SUCCESS
    })
}

/// Creates a typed Decrypt request and copies caller-provided secret spans.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_decrypt(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    iv_counter_nonce: *const crate::kmipkit_byte_span_v1_t,
    authenticated_encryption_tag: *const crate::kmipkit_byte_span_v1_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    authenticated_encryption_additional_data: *const crate::kmipkit_byte_span_v1_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let iv_counter_nonce = try_ffi!(copy_optional_secret_span(iv_counter_nonce));
        let authenticated_encryption_tag =
            try_ffi!(copy_optional_secret_span(authenticated_encryption_tag));
        let correlation_value = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let additional_data = try_ffi!(copy_optional_secret_span(
            authenticated_encryption_additional_data
        ));

        let mut request = DecryptRequest::new(identifier, data);
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(value) = iv_counter_nonce {
            request = request.with_iv_counter_nonce(value);
        }
        if let Some(value) = authenticated_encryption_tag {
            request = request.with_authenticated_encryption_tag(value);
        }
        if let Some(value) = correlation_value {
            request = request.with_correlation_value(value);
        }
        if let Some(value) = init_indicator {
            request = request.with_init_indicator(value);
        }
        if let Some(value) = final_indicator {
            request = request.with_final_indicator(value);
        }
        if let Some(value) = additional_data {
            request = request.with_authenticated_encryption_additional_data(value);
        }
        *out = try_ffi!(batch_item_handle(ClientRequest::Decrypt(request)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_mac(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let mut request = MacRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = data {
            request = request.with_data(data);
        }
        request = with_cryptographic_framing(request, correlation, init_indicator, final_indicator);
        *out = try_ffi!(batch_item_handle(ClientRequest::Mac(request)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_mac_verify(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    mac_data: *const crate::kmipkit_byte_span_v1_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let mac_data = try_ffi!(copy_optional_secret_span(mac_data));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let mut request = MacVerifyRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = data {
            request = request.with_data(data);
        }
        if let Some(mac_data) = mac_data {
            request = request.with_mac_data(mac_data);
        }
        request = with_cryptographic_framing(request, correlation, init_indicator, final_indicator);
        *out = try_ffi!(batch_item_handle(ClientRequest::MacVerify(request)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_sign(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    digested_data: *const crate::kmipkit_byte_span_v1_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let digested_data = try_ffi!(copy_optional_secret_span(digested_data));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let mut request = SignRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = data {
            request = request.with_data(data);
        }
        if let Some(digested_data) = digested_data {
            request = request.with_digested_data(digested_data);
        }
        request = with_cryptographic_framing(request, correlation, init_indicator, final_indicator);
        *out = try_ffi!(batch_item_handle(ClientRequest::Sign(request)));
        SUCCESS
    })
}

/// Creates a KMIP 2.1 §6.1.49 RNG Retrieve request with the caller's signed Data Length.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_rng_retrieve(
    data_length: i32,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        *out = try_ffi!(batch_item_handle(ClientRequest::rng_retrieve(
            RngRetrieveRequest::new(data_length),
        )));
        SUCCESS
    })
}

/// Creates a KMIP 2.1 §6.1.50 RNG Seed request from required secret seed bytes.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_rng_seed(
    data: *const crate::kmipkit_byte_span_v1_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let Some(data) = try_ffi!(copy_optional_secret_span(data)) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(batch_item_handle(ClientRequest::rng_seed(
            RngSeedRequest::new(data)
        )));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_signature_verify(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    parameters: *mut kmipkit_ttlv_structure_t,
    data: *mut kmipkit_ttlv_value_t,
    digested_data: *const crate::kmipkit_byte_span_v1_t,
    signature_data: *const crate::kmipkit_byte_span_v1_t,
    correlation_value: *const crate::kmipkit_byte_span_v1_t,
    init_indicator: u8,
    final_indicator: u8,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let identifier = try_ffi!(copy_optional_identifier(unique_identifier));
        let parameters = try_ffi!(copy_structure_handle(parameters, false));
        let data = try_ffi!(copy_operation_data(data));
        let digested_data = try_ffi!(copy_optional_secret_span(digested_data));
        let signature_data = try_ffi!(copy_optional_secret_span(signature_data));
        let correlation = try_ffi!(copy_optional_secret_span(correlation_value));
        let init_indicator = try_ffi!(optional_indicator(init_indicator));
        let final_indicator = try_ffi!(optional_indicator(final_indicator));
        let mut request = SignatureVerifyRequest::new();
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(parameters) = parameters {
            request = request.with_cryptographic_parameters(parameters);
        }
        if let Some(data) = data {
            request = request.with_data(data);
        }
        if let Some(digested_data) = digested_data {
            request = request.with_digested_data(digested_data);
        }
        if let Some(signature_data) = signature_data {
            request = request.with_signature_data(signature_data);
        }
        request = with_cryptographic_framing(request, correlation, init_indicator, final_indicator);
        *out = try_ffi!(batch_item_handle(ClientRequest::SignatureVerify(request)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create(
    object_type: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let attributes = try_ffi!(copy_attribute_set_handle(attributes, false));
        let Some(attributes) = attributes else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let masks = try_ffi!(copy_structure_handle(protection_storage_masks, false));
        let mut request = CreateRequest::new(ObjectType::from_raw(object_type), attributes);
        if let Some(masks) = masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Create(
                request,
            )))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create_key_pair(
    common_attributes: *mut kmipkit_ttlv_structure_t,
    private_key_attributes: *mut kmipkit_ttlv_structure_t,
    public_key_attributes: *mut kmipkit_ttlv_structure_t,
    common_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    private_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    public_protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let common_attributes = try_ffi!(copy_attribute_set_handle(common_attributes, false));
        let private_key_attributes =
            try_ffi!(copy_attribute_set_handle(private_key_attributes, false));
        let public_key_attributes =
            try_ffi!(copy_attribute_set_handle(public_key_attributes, false));
        let common_protection_storage_masks = try_ffi!(copy_structure_handle(
            common_protection_storage_masks,
            false
        ));
        let private_protection_storage_masks = try_ffi!(copy_structure_handle(
            private_protection_storage_masks,
            false
        ));
        let public_protection_storage_masks = try_ffi!(copy_structure_handle(
            public_protection_storage_masks,
            false
        ));
        let mut request = CreateKeyPairRequest::new();
        if let Some(attributes) = common_attributes {
            request = request.with_common_attributes(attributes);
        }
        if let Some(attributes) = private_key_attributes {
            request = request.with_private_key_attributes(attributes);
        }
        if let Some(attributes) = public_key_attributes {
            request = request.with_public_key_attributes(attributes);
        }
        if let Some(masks) = common_protection_storage_masks {
            request = request.with_common_protection_storage_masks(masks);
        }
        if let Some(masks) = private_protection_storage_masks {
            request = request.with_private_protection_storage_masks(masks);
        }
        if let Some(masks) = public_protection_storage_masks {
            request = request.with_public_protection_storage_masks(masks);
        }
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(
                ClientRequest::CreateKeyPair(request),
            ))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_create_split_key(
    object_type: u32,
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    split_key_parts: i32,
    split_key_threshold: i32,
    split_key_method: u32,
    has_prime_field_size: u8,
    prime_field_size_data: *const u8,
    prime_field_size_length: u64,
    attributes: *mut kmipkit_ttlv_structure_t,
    protection_storage_masks: *mut kmipkit_ttlv_structure_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let attributes = try_ffi!(copy_attribute_set_handle(attributes, false));
        let Some(attributes) = attributes else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let identifier = try_ffi!(copy_unique_identifier(unique_identifier));
        let prime_field_size = try_ffi!(copy_optional_bytes(
            prime_field_size_data,
            prime_field_size_length,
            has_prime_field_size,
        ));
        let masks = try_ffi!(copy_structure_handle(protection_storage_masks, false));
        let mut request = CreateSplitKeyRequest::new(
            ObjectType::from_raw(object_type),
            split_key_parts,
            split_key_threshold,
            SplitKeyMethod::from_raw(split_key_method),
            attributes,
        );
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(prime_field_size) = prime_field_size {
            request = request.with_prime_field_size(prime_field_size);
        }
        if let Some(masks) = masks {
            request = request.with_protection_storage_masks(masks);
        }
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(
                ClientRequest::CreateSplitKey(request),
            ))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_register_request_create(
    object_type: u32,
    attributes: *mut kmipkit_ttlv_structure_t,
    object: *mut kmipkit_ttlv_item_t,
    out_request: *mut *mut kmipkit_register_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let _ = read_handle!(attributes, Structure);
        let _ = read_handle!(object, Item);

        // Transfer both owners before the first fallible unwrap so an error cannot strand the
        // second consumed input in a language binding that has already disarmed both wrappers.
        let attributes_owner = try_ffi!(consume_owner(attributes, Kind::Structure));
        let object_owner = try_ffi!(consume_owner(object, Kind::Item));
        let attributes = try_ffi!(unwrap_payload(attributes_owner));
        let HandleValue::Structure(attributes) = attributes else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let object = try_ffi!(unwrap_payload(object_owner));
        let HandleValue::Item(object) = object else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };

        let attributes_view = attributes.view();
        let source_items = attributes_view.children();
        let mut items = Vec::new();
        if items.try_reserve_exact(source_items.len()).is_err() {
            return record_ffi_error(ERROR_RESOURCE_LIMIT);
        }
        for item in source_items {
            items.push(try_ffi!(item.try_clone().map_err(model_status)));
        }
        let attributes = match AttributeSet::try_new(items) {
            Ok(attributes) => attributes,
            Err(_error) => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let request = RegisterRequest::new(ObjectType::from_raw(object_type), attributes, object);
        *out = try_ffi!(try_make_handle(
            Kind::RegisterRequest,
            HandleValue::RegisterRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

/// Creates a raw-preserving Revocation Reason from KMIP 2.1 §11.48, Table 481.
///
/// `message_data` is the optional audit message nested inside Revocation Reason;
/// it is not a Request Payload sibling. The input bytes are copied.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_revocation_reason_create(
    raw_code: u32,
    has_message: u8,
    message_data: *const u8,
    message_length: u64,
    out_reason: *mut *mut kmipkit_revocation_reason_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_reason);
        let message = try_ffi!(copy_optional_bytes(
            message_data,
            message_length,
            has_message
        ));
        let message = match message {
            Some(bytes) => match String::from_utf8(bytes) {
                Ok(message) => Some(message),
                Err(_error) => return record_ffi_error(ERROR_INVALID_INPUT),
            },
            None => None,
        };
        *out = try_ffi!(try_make_handle(
            Kind::RevocationReason,
            HandleValue::RevocationReason {
                reason: RevocationReason::from_raw(raw_code),
                message,
            },
        ));
        SUCCESS
    })
}

/// Creates a typed Revoke request from KMIP 2.1 §6.1.44, Table 295.
///
/// The `reason` handle is consumed at function entry, including if later identifier
/// copying or date-flag validation fails. An absent identifier leaves ID Placeholder
/// use explicit; an optional date is checked when making the batch item.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_revoke_request_create(
    unique_identifier: *const kmipkit_unique_identifier_v1_t,
    reason: *mut kmipkit_revocation_reason_t,
    has_compromise_occurrence_date: u8,
    compromise_occurrence_date: i64,
    out_request: *mut *mut kmipkit_revoke_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let _ = read_handle!(reason, RevocationReason);
        let reason = try_ffi!(owned_payload(reason, Kind::RevocationReason));
        let identifier = try_ffi!(copy_unique_identifier(unique_identifier));
        let has_date = match has_compromise_occurrence_date {
            0 => false,
            1 => true,
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        let HandleValue::RevocationReason { reason, message } = reason else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let mut request = RevokeRequest::new(reason);
        if let Some(identifier) = identifier {
            request = request.with_unique_identifier(identifier);
        }
        if let Some(message) = message {
            request = request.with_revocation_message(message);
        }
        if has_date {
            request = request.with_compromise_occurrence_date(compromise_occurrence_date);
        }
        *out = try_ffi!(try_make_handle(
            Kind::RevokeRequest,
            HandleValue::RevokeRequest(Box::new(request)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_register_request_with_opaque_object_context(
    request: *mut kmipkit_register_request_t,
    context: u32,
    out_request: *mut *mut kmipkit_register_request_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_request);
        let _ = read_handle!(request, RegisterRequest);
        let request = try_ffi!(owned_payload(request, Kind::RegisterRequest));
        let HandleValue::RegisterRequest(request) = request else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let context = match context {
            1 => OpaqueObjectContext::Generic,
            2 => OpaqueObjectContext::ManagedCryptographicObject,
            _ => return record_ffi_error(ERROR_INVALID_INPUT),
        };
        *out = try_ffi!(try_make_handle(
            Kind::RegisterRequest,
            HandleValue::RegisterRequest(Box::new(request.with_opaque_object_context(context))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_register(
    request: *mut kmipkit_register_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let _ = read_handle!(request, RegisterRequest);
        let request = try_ffi!(owned_payload(request, Kind::RegisterRequest));
        let HandleValue::RegisterRequest(request) = request else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = match ClientBatchItem::register(*request) {
            Ok(item) => item,
            Err(error) => return record_ffi_error(protocol_status(&error)),
        };
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(item))
        ));
        SUCCESS
    })
}

/// Consumes a Revoke request and validates its SHALL NOT date constraint.
#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_revoke(
    request: *mut kmipkit_revoke_request_t,
    out_item: *mut *mut kmipkit_client_batch_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let request = try_ffi!(owned_payload(request, Kind::RevokeRequest));
        let HandleValue::RevokeRequest(request) = request else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if request.compromise_occurrence_date().is_some()
            && !matches!(request.revocation_reason().raw(), 2 | 3)
        {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(try_make_handle(
            Kind::BatchItem,
            HandleValue::BatchItem(Box::new(ClientBatchItem::new(ClientRequest::Revoke(
                *request,
            )))),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_count(
    item: *mut kmipkit_client_batch_item_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        let item = read_handle!(item, BatchItem);
        let HandleValue::BatchItem(item) = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(u64::try_from(item.extension_count()).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_identity_at(
    item: *mut kmipkit_client_batch_item_t,
    index: u64,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_identity);
        let item = read_handle!(item, BatchItem);
        let HandleValue::BatchItem(item) = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(index) = usize::try_from(index).ok() else {
            return SUCCESS;
        };
        let Some(identity) = item.extension_identity_at(index) else {
            return SUCCESS;
        };
        *out = try_ffi!(try_make_handle(
            Kind::Identity,
            HandleValue::Identity(identity)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_client_batch_item_extension_criticality_indicator_at(
    item: *mut kmipkit_client_batch_item_t,
    index: u64,
    out_criticality_indicator: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_criticality_indicator);
        let item = read_handle!(item, BatchItem);
        let HandleValue::BatchItem(item) = &item.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(index) = usize::try_from(index).ok() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(criticality_indicator) = item.extension_criticality_indicator_at(index) else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = u8::from(criticality_indicator);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_create(
    extension_name_data: *const u8,
    extension_name_length: u64,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_information);
        let name = try_ffi!(input_span(
            extension_name_data,
            extension_name_length,
            MAX_TEXT_BYTES
        ));
        let name = try_ffi!(std::str::from_utf8(name).map_err(|_| ERROR_INVALID_IDENTITY));
        let information =
            try_ffi!(extension::extension_information(name).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(
            Kind::Information,
            HandleValue::Information(information)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_tag(
    information: *mut kmipkit_extension_information_t,
    tag: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        information_transform(information, out_information, |info| {
            extension::with_tag(info, tag).map_err(|error| protocol_status(&error))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_type(
    information: *mut kmipkit_extension_information_t,
    item_type: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        information_transform(information, out_information, |info| {
            let item_type = item_type_from_u32(item_type)?;
            extension::with_type(info, item_type).map_err(|error| protocol_status(&error))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_enumeration(
    information: *mut kmipkit_extension_information_t,
    enumeration: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        information_transform(information, out_information, |info| {
            extension::with_enumeration(info, enumeration).map_err(|error| protocol_status(&error))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_attribute(
    information: *mut kmipkit_extension_information_t,
    attribute: u8,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        information_transform(information, out_information, |info| {
            if attribute > 1 {
                return Err(ERROR_INVALID_INPUT);
            }
            extension::with_attribute(info, attribute == 1).map_err(|error| protocol_status(&error))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_parent_structure_tag(
    information: *mut kmipkit_extension_information_t,
    parent_structure_tag: u32,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        information_transform(information, out_information, |info| {
            extension::with_parent_structure_tag(info, parent_structure_tag)
                .map_err(|error| protocol_status(&error))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_description(
    information: *mut kmipkit_extension_information_t,
    description_data: *const u8,
    description_length: u64,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_information);
        let source = try_ffi!(owned_payload(information, Kind::Information));
        let HandleValue::Information(source) = source else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let bytes = try_ffi!(input_span(
            description_data,
            description_length,
            MAX_TEXT_BYTES
        ));
        let description = try_ffi!(std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT));
        let information = try_ffi!(
            extension::with_description(source, description).map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Information,
            HandleValue::Information(information)
        ));
        SUCCESS
    })
}

fn information_transform(
    pointer: *mut kmipkit_extension_information_t,
    out_information: *mut *mut kmipkit_extension_information_t,
    transform: impl FnOnce(
        extension::ExtensionInformation,
    ) -> FfiResult<extension::ExtensionInformation>,
) -> i32 {
    let out = out_slot!(out_information);
    let information = try_ffi!(owned_payload(pointer, Kind::Information));
    let HandleValue::Information(information) = information else {
        return record_ffi_error(ERROR_INVALID_INPUT);
    };
    let information = try_ffi!(transform(information));
    *out = try_ffi!(try_make_handle(
        Kind::Information,
        HandleValue::Information(information)
    ));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_information_to_ttlv(
    information: *mut kmipkit_extension_information_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let information = read_handle!(information, Information);
        let HandleValue::Information(information) = &information.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let structure =
            try_ffi!(extension::to_ttlv(information.clone()).map_err(|e| protocol_status(&e)));
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_with_information(
    definition: *mut kmipkit_extension_definition_t,
    information: *mut kmipkit_extension_information_t,
    out_definition: *mut *mut kmipkit_extension_definition_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_definition);
        let information = read_handle!(information, Information);
        let definition = try_ffi!(owned_payload(definition, Kind::Definition));
        let HandleValue::Definition(definition) = definition else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let HandleValue::Information(information) = &information.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let definition = try_ffi!(
            extension::with_information(*definition, information.clone())
                .map_err(|e| protocol_status(&e))
        );
        *out = try_ffi!(try_make_handle(
            Kind::Definition,
            HandleValue::Definition(Box::new(definition)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_identity(
    definition: *mut kmipkit_extension_definition_t,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_identity);
        let definition = read_handle!(definition, Definition);
        let definition = try_ffi!(definition_from_handle(&definition));
        *out = try_ffi!(try_make_handle(
            Kind::Identity,
            HandleValue::Identity(extension::identity(definition)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_definition_information(
    definition: *mut kmipkit_extension_definition_t,
    out_information: *mut *mut kmipkit_extension_information_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_information);
        let definition = read_handle!(definition, Definition);
        let definition = try_ffi!(definition_from_handle(&definition));
        if let Some(information) = extension::information(definition) {
            *out = try_ffi!(try_make_handle(
                Kind::Information,
                HandleValue::Information(information)
            ));
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_is_recognized(
    recognition: *mut kmipkit_extension_recognition_t,
    out_recognized: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_recognized);
        let recognition = read_handle!(recognition, Recognition);
        let HandleValue::Recognition(recognition) = &recognition.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = u32::from(recognition.is_recognized());
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_validated_value(
    recognition: *mut kmipkit_extension_recognition_t,
    out_value: *mut *mut kmipkit_validated_extension_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let recognition_handle = read_handle!(recognition, Recognition);
        let HandleValue::Recognition(recognition_value) = &recognition_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        if recognition_value.validated_value().is_some() {
            let owner = try_ffi!(clone_handle_owner(recognition, Kind::Recognition));
            *out = try_ffi!(try_make_handle(
                Kind::Validated,
                HandleValue::ValidatedRef(owner)
            ));
        }
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_extension_recognition_generic_value(
    recognition: *mut kmipkit_extension_recognition_t,
    out_value: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let _ = read_handle!(recognition, Recognition);
        let owner = try_ffi!(clone_handle_owner(recognition, Kind::Recognition));
        if !matches!(owner.value, HandleValue::Recognition(_)) {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        let view = TtlvView::new(owner, ViewRoot::RecognitionGeneric, ViewTarget::Structure);
        *out = try_ffi!(try_make_handle(
            Kind::StructureView,
            HandleValue::StructureView(view)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_identity(
    value: *mut kmipkit_validated_extension_value_t,
    out_identity: *mut *mut kmipkit_extension_identity_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_identity);
        let value = read_handle!(value, Validated);
        let value = try_ffi!(validated_from_handle(&value));
        *out = try_ffi!(try_make_handle(
            Kind::Identity,
            HandleValue::Identity(extension::validated_extension_value_identity(value)),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_generic_value(
    value: *mut kmipkit_validated_extension_value_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let value_handle = read_handle!(value, Validated);
        let _ = try_ffi!(validated_from_handle(&value_handle));
        let owner = try_ffi!(clone_handle_owner(value, Kind::Validated));
        let structure = TtlvView::new(owner, ViewRoot::ValidatedGeneric, ViewTarget::Structure);
        *out = try_ffi!(try_make_handle(
            Kind::StructureView,
            HandleValue::StructureView(structure)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_validated_extension_value_value_at(
    value: *mut kmipkit_validated_extension_value_t,
    path: *mut kmipkit_ttlv_path_t,
    out_value: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let _value_handle = read_handle!(value, Validated);
        let path_handle = read_handle!(path, Path);
        let HandleValue::Path(path) = &path_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let owner = try_ffi!(clone_handle_owner(value, Kind::Validated));
        let structure = TtlvView::new(owner, ViewRoot::ValidatedGeneric, ViewTarget::Structure);
        let value_view = try_ffi!(structure.value_at_tags(path.tags()));
        *out = try_ffi!(try_make_handle(
            Kind::ValueView,
            HandleValue::ValueView(value_view)
        ));
        SUCCESS
    })
}

fn padded_length(value_length: u64) -> FfiResult<u64> {
    value_length
        .checked_add(7)
        .map(|value| value & !7)
        .ok_or(ERROR_RESOURCE_LIMIT)
}

fn structure_measure(structure: &kmipkit_ttlv::StructureView<'_>) -> FfiResult<(u64, u64, u64)> {
    let mut value_bytes = 0_u64;
    let mut elements = 0_u64;
    let mut max_depth = 1_u64;
    for item in structure.children() {
        let (payload_bytes, nested_elements, nested_depth) = item.with_value(measure_value)?;
        let encoded = 8_u64
            .checked_add(padded_length(payload_bytes)?)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        value_bytes = value_bytes
            .checked_add(encoded)
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        elements = elements
            .checked_add(1)
            .and_then(|total| total.checked_add(nested_elements))
            .ok_or(ERROR_RESOURCE_LIMIT)?;
        max_depth = max_depth.max(nested_depth.saturating_add(u64::from(nested_depth > 0)));
    }
    Ok((value_bytes, elements, max_depth))
}

fn measure_value(value: ValueView<'_>) -> FfiResult<(u64, u64, u64)> {
    match value {
        ValueView::Structure(structure) => structure_measure(&structure),
        ValueView::Integer(_) | ValueView::Enumeration(_) | ValueView::Interval(_) => Ok((4, 0, 0)),
        ValueView::LongInteger(_) | ValueView::DateTime(_) | ValueView::DateTimeExtended(_) => {
            Ok((8, 0, 0))
        }
        ValueView::Boolean(_) => Ok((8, 0, 0)),
        ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) => Ok((
            u64::try_from(bytes.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?,
            0,
            0,
        )),
        ValueView::TextString(text) => Ok((
            u64::try_from(text.len()).map_err(|_| ERROR_RESOURCE_LIMIT)?,
            0,
            0,
        )),
        _ => Err(ERROR_INVALID_SCHEMA),
    }
}

fn check_structure_limits(structure: &Structure, limits: &CodecLimits) -> FfiResult<(u64, u64)> {
    let (payload_bytes, nested_elements, depth) = structure_measure(&structure.view())?;
    let encoded_size = 8_u64
        .checked_add(payload_bytes)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let total_elements = 1_u64
        .checked_add(nested_elements)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if encoded_size > u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || total_elements
            > u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || depth > u64::try_from(limits.max_structure_depth()).map_err(|_| ERROR_RESOURCE_LIMIT)?
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok((encoded_size, total_elements))
}

fn check_item_limits(item: &Item, limits: &CodecLimits) -> FfiResult<(u64, u64)> {
    let (payload_bytes, nested_elements, depth) = item.with_value(measure_value)?;
    let encoded_size = 8_u64
        .checked_add(padded_length(payload_bytes)?)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    let total_elements = 1_u64
        .checked_add(nested_elements)
        .ok_or(ERROR_RESOURCE_LIMIT)?;
    if encoded_size > u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || total_elements
            > u64::try_from(limits.max_elements()).map_err(|_| ERROR_RESOURCE_LIMIT)?
        || depth > u64::try_from(limits.max_structure_depth()).map_err(|_| ERROR_RESOURCE_LIMIT)?
    {
        return Err(ERROR_RESOURCE_LIMIT);
    }
    Ok((encoded_size, total_elements))
}

fn codec_limits_from_handle(pointer: *mut kmipkit_codec_limits_t) -> FfiResult<Arc<CodecLimits>> {
    let handle = reference_handle(pointer, Kind::CodecLimits)?;
    match &handle.value {
        HandleValue::CodecLimits(limits) => Ok(Arc::clone(limits)),
        _ => Err(ERROR_INVALID_INPUT),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_create(
    raw: u32,
    out_raw_tag: *mut *mut kmipkit_raw_tag_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_raw_tag);
        let raw = try_ffi!(RawTag::new(raw).map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::RawTag, HandleValue::RawTag(raw)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_value(
    raw_tag: *mut kmipkit_raw_tag_t,
    out_raw: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_raw);
        let raw_tag = read_handle!(raw_tag, RawTag);
        let HandleValue::RawTag(raw_tag) = &raw_tag.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = raw_tag.raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_raw_tag_try_checked(
    raw_tag: *mut kmipkit_raw_tag_t,
    out_tag: *mut *mut kmipkit_tag_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_tag);
        let raw_tag = read_handle!(raw_tag, RawTag);
        let HandleValue::RawTag(raw_tag) = &raw_tag.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let tag = try_ffi!(raw_tag.try_checked().map_err(model_status));
        *out = try_ffi!(try_make_handle(Kind::Tag, HandleValue::Tag(tag)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_tag_value(tag: *mut kmipkit_tag_t, out_raw: *mut u32) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_raw);
        let tag = read_handle!(tag, Tag);
        let HandleValue::Tag(tag) = &tag.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = tag.raw();
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_defaults(
    out_limits: *mut *mut kmipkit_codec_limits_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_limits);
        *out = try_ffi!(try_make_handle(
            Kind::CodecLimits,
            HandleValue::CodecLimits(Arc::new(CodecLimits::defaults())),
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_create(
    max_message_bytes: u64,
    max_structure_depth: u64,
    max_elements: u64,
    out_limits: *mut *mut kmipkit_codec_limits_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_limits);
        let max_message_bytes =
            try_ffi!(usize::try_from(max_message_bytes).map_err(|_| ERROR_RESOURCE_LIMIT));
        let max_structure_depth =
            try_ffi!(usize::try_from(max_structure_depth).map_err(|_| ERROR_RESOURCE_LIMIT));
        let max_elements =
            try_ffi!(usize::try_from(max_elements).map_err(|_| ERROR_RESOURCE_LIMIT));
        let limits = try_ffi!(
            CodecLimits::new(max_message_bytes, max_structure_depth, max_elements)
                .map_err(|_| ERROR_RESOURCE_LIMIT)
        );
        *out = try_ffi!(try_make_handle(
            Kind::CodecLimits,
            HandleValue::CodecLimits(Arc::new(limits)),
        ));
        SUCCESS
    })
}

fn codec_limit_value(
    pointer: *mut kmipkit_codec_limits_t,
    out_value: *mut u64,
    selector: fn(&CodecLimits) -> usize,
) -> i32 {
    let output = scalar_out!(out_value);
    let limits = try_ffi!(codec_limits_from_handle(pointer));
    *output = try_ffi!(u64::try_from(selector(&limits)).map_err(|_| ERROR_RESOURCE_LIMIT));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_message_bytes(
    limits: *mut kmipkit_codec_limits_t,
    max_message_bytes: *mut u64,
) -> i32 {
    ffi_entrypoint(|| codec_limit_value(limits, max_message_bytes, CodecLimits::max_message_bytes))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_structure_depth(
    limits: *mut kmipkit_codec_limits_t,
    max_structure_depth: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        codec_limit_value(
            limits,
            max_structure_depth,
            CodecLimits::max_structure_depth,
        )
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_codec_limits_max_elements(
    limits: *mut kmipkit_codec_limits_t,
    max_elements: *mut u64,
) -> i32 {
    ffi_entrypoint(|| codec_limit_value(limits, max_elements, CodecLimits::max_elements))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_integer(
    value: i32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::integer(value))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_long_integer(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::long_integer(value))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_enumeration(
    value: u32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::enumeration(value))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_boolean(
    value: u8,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        if value > 1 {
            return record_ffi_error(ERROR_INVALID_INPUT);
        }
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::boolean(value == 1))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_date_time(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::date_time(value))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_interval(
    value: u32,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::interval(value))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_date_time_extended(
    value: i64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::date_time_extended(value)),
        ));
        SUCCESS
    })
}

fn variable_value(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
    build: fn(&[u8]) -> FfiResult<Value>,
) -> i32 {
    let out = out_slot!(out_value);
    let limits = try_ffi!(codec_limits_from_handle(limits));
    let maximum =
        try_ffi!(u64::try_from(limits.max_message_bytes()).map_err(|_| ERROR_RESOURCE_LIMIT));
    let bytes = try_ffi!(input_span(value, value_length, maximum));
    let value = try_ffi!(build(bytes));
    *out = try_ffi!(try_make_handle(Kind::Value, HandleValue::Value(value)));
    SUCCESS
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_big_integer(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        variable_value(limits, value, value_length, out_value, |bytes| {
            Ok(Value::big_integer(bytes.to_vec()))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_text_string(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        variable_value(limits, value, value_length, out_value, |bytes| {
            let text = std::str::from_utf8(bytes).map_err(|_| ERROR_INVALID_INPUT)?;
            Ok(Value::text_string(text.to_owned()))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_byte_string(
    limits: *mut kmipkit_codec_limits_t,
    value: *const u8,
    value_length: u64,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        variable_value(limits, value, value_length, out_value, |bytes| {
            Ok(Value::byte_string(bytes.to_vec()))
        })
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_structure(
    structure: *mut kmipkit_ttlv_structure_t,
    limits: *mut kmipkit_codec_limits_t,
    out_value: *mut *mut kmipkit_ttlv_value_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let limits = try_ffi!(codec_limits_from_handle(limits));
        let structure = try_ffi!(owned_payload(structure, Kind::Structure));
        let HandleValue::Structure(structure) = structure else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        try_ffi!(check_structure_limits(&structure, &limits));
        *out = try_ffi!(try_make_handle(
            Kind::Value,
            HandleValue::Value(Value::structure(structure))
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_create(
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(Structure::new())
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_create(
    tag: *mut kmipkit_tag_t,
    value: *mut kmipkit_ttlv_value_t,
    limits: *mut kmipkit_codec_limits_t,
    out_item: *mut *mut kmipkit_ttlv_item_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let tag_handle = read_handle!(tag, Tag);
        let HandleValue::Tag(tag) = &tag_handle.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let limits = try_ffi!(codec_limits_from_handle(limits));
        let value = try_ffi!(owned_payload(value, Kind::Value));
        let HandleValue::Value(value) = value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item = try_ffi!(Item::new(*tag, value).map_err(model_status));
        try_ffi!(check_item_limits(&item, &limits));
        *out = try_ffi!(try_make_handle(Kind::Item, HandleValue::Item(item)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_with_item(
    structure: *mut kmipkit_ttlv_structure_t,
    item: *mut kmipkit_ttlv_item_t,
    limits: *mut kmipkit_codec_limits_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let limits = try_ffi!(codec_limits_from_handle(limits));
        let _ = read_handle!(structure, Structure);
        let _ = read_handle!(item, Item);
        let structure = try_ffi!(consume_owner(structure, Kind::Structure));
        let item = try_ffi!(consume_owner(item, Kind::Item));
        let structure = try_ffi!(unwrap_payload(structure));
        let item = try_ffi!(unwrap_payload(item));
        let (HandleValue::Structure(mut structure), HandleValue::Item(item)) = (structure, item)
        else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        try_ffi!(structure.try_push(item).map_err(model_status));
        try_ffi!(check_structure_limits(&structure, &limits));
        *out = try_ffi!(try_make_handle(
            Kind::Structure,
            HandleValue::Structure(structure)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view(
    structure: *mut kmipkit_ttlv_structure_t,
    out_view: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_view);
        let _ = read_handle!(structure, Structure);
        let owner = try_ffi!(clone_handle_owner(structure, Kind::Structure));
        let view = TtlvView::new(owner, ViewRoot::Structure, ViewTarget::Structure);
        *out = try_ffi!(try_make_handle(
            Kind::StructureView,
            HandleValue::StructureView(view)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_item_count(
    view: *mut kmipkit_ttlv_structure_view_t,
    out_count: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_count);
        let view = read_handle!(view, StructureView);
        let HandleValue::StructureView(view) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let count = try_ffi!(view.with_structure(|structure| structure.children().len()));
        *out = try_ffi!(u64::try_from(count).map_err(|_| ERROR_RESOURCE_LIMIT));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_structure_view_item_at(
    view: *mut kmipkit_ttlv_structure_view_t,
    index: u64,
    out_item: *mut *mut kmipkit_ttlv_item_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_item);
        let view = read_handle!(view, StructureView);
        let HandleValue::StructureView(view) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let index = try_ffi!(usize::try_from(index).map_err(|_| ERROR_INVALID_INPUT));
        let item = try_ffi!(view.item_at(index));
        *out = try_ffi!(try_make_handle(Kind::ItemView, HandleValue::ItemView(item)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_tag(
    view: *mut kmipkit_ttlv_item_view_t,
    out_tag: *mut *mut kmipkit_tag_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_tag);
        let view = read_handle!(view, ItemView);
        let HandleValue::ItemView(item) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let tag = try_ffi!(item.with_item(Item::tag));
        *out = try_ffi!(try_make_handle(Kind::Tag, HandleValue::Tag(tag)));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_type(
    view: *mut kmipkit_ttlv_item_view_t,
    out_item_type: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_item_type);
        let view = read_handle!(view, ItemView);
        let HandleValue::ItemView(item) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let item_type = try_ffi!(item.with_item(Item::item_type));
        *out = item_type_code(item_type);
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_item_view_value(
    view: *mut kmipkit_ttlv_item_view_t,
    out_value: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_value);
        let view = read_handle!(view, ItemView);
        let HandleValue::ItemView(item) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let value = try_ffi!(item.item_value());
        *out = try_ffi!(try_make_handle(
            Kind::ValueView,
            HandleValue::ValueView(value)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view(
    value: *mut kmipkit_ttlv_value_t,
    out_view: *mut *mut kmipkit_ttlv_value_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_view);
        let _ = read_handle!(value, Value);
        let owner = try_ffi!(clone_handle_owner(value, Kind::Value));
        let view = TtlvView::new(owner, ViewRoot::Value, ViewTarget::Value);
        *out = try_ffi!(try_make_handle(
            Kind::ValueView,
            HandleValue::ValueView(view)
        ));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_type(
    view: *mut kmipkit_ttlv_value_view_t,
    out_item_type: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_item_type);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(value.with_value(|value| Ok(value_view_type_code(&value))));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_structure(
    view: *mut kmipkit_ttlv_value_view_t,
    out_structure: *mut *mut kmipkit_ttlv_structure_view_t,
) -> i32 {
    ffi_entrypoint(|| {
        let out = out_slot!(out_structure);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let structure = try_ffi!(value.structure_from_value());
        *out = try_ffi!(try_make_handle(
            Kind::StructureView,
            HandleValue::StructureView(structure)
        ));
        SUCCESS
    })
}

fn extract_value<T: Copy>(
    value: &TtlvView,
    extract: impl FnOnce(ValueView<'_>) -> Option<T>,
) -> FfiResult<T> {
    value.with_value(|value| extract(value).ok_or(ERROR_INVALID_INPUT))
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_integer(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::Integer(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_long_integer(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::LongInteger(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_enumeration(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::Enumeration(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_boolean(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = u8::from(try_ffi!(extract_value(value, |value| match value {
            ValueView::Boolean(v) => Some(*v),
            _ => None,
        })));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_date_time(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::DateTime(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_interval(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut u32,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::Interval(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_date_time_extended(
    view: *mut kmipkit_ttlv_value_view_t,
    out_value: *mut i64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_value);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::DateTimeExtended(v) => Some(*v),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_byte_length(
    view: *mut kmipkit_ttlv_value_view_t,
    out_length: *mut u64,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_length);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) =>
                u64::try_from(bytes.len()).ok(),
            ValueView::TextString(text) => u64::try_from(text.len()).ok(),
            _ => None,
        }));
        SUCCESS
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn kmipkit_ttlv_value_view_byte_at(
    view: *mut kmipkit_ttlv_value_view_t,
    index: u64,
    out_byte: *mut u8,
) -> i32 {
    ffi_entrypoint(|| {
        let out = scalar_out!(out_byte);
        let view = read_handle!(view, ValueView);
        let HandleValue::ValueView(value) = &view.value else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        let Some(index) = usize::try_from(index).ok() else {
            return record_ffi_error(ERROR_INVALID_INPUT);
        };
        *out = try_ffi!(extract_value(value, |value| match value {
            ValueView::BigInteger(bytes) | ValueView::ByteString(bytes) =>
                bytes.get(index).copied(),
            ValueView::TextString(text) => text.as_bytes().get(index).copied(),
            _ => None,
        }));
        SUCCESS
    })
}
