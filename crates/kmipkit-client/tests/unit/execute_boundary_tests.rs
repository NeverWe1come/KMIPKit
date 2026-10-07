//! Red-only tests for the private execute-boundary source checker.
//!
//! Fixture files are inert Rust source snippets loaded as text. They are not
//! compiled, executed, or read from OASIS sources. T011 replaces these
//! deliberately incomplete test-only checker candidates with the approved
//! pinned Rust-AST audit; T017 wires that production audit into CI.
//! The low-level caller-owned transport exception belongs to
//! `kmipkit-transport`; client source may call `exchange` only from
//! `Client::exchange_operation`, reached only from closed typed request entry points.
//! The audit is syntactic: it walks the production module graph under `src`,
//! accepts only `cfg(test)`/`cfg(not(test))`, and rejects unknown attributes,
//! imported protected names, custom macros, and every macro outside its small
//! standard-library whitelist. It does not expand macros or resolve types; the
//! whitelist's token trees are checked for protected boundary identifiers.
//!
//! Traceability: `KMIPKIT-0007-FR-001`, `-FR-003`, `-FR-011`, `-FR-012`,
//! `-FR-015`, `-FR-017`; `KMIPKIT-0007-SC-004`, `-SC-007`, `-SC-008`;
//! ADR-0012, ADR-0014, OD-005.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::parse::{Parse, ParseStream};
use syn::visit::{self, Visit};
use syn::{
    Attribute, Expr, ExprCall, ExprMethodCall, Item, ItemMod, Macro, Meta, Pat, Type, UseTree,
};

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
        source: include_str!("../../tests/fixtures/execute_boundary/valid_execute.rs"),
        probe: "OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "canonical_vec_macro",
        path: "tests/fixtures/execute_boundary/canonical_vec_macro.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/canonical_vec_macro.rs"),
        probe: "vec![0u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "generic_item_input",
        path: "tests/fixtures/execute_boundary/generic_item_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/generic_item_input.rs"),
        probe: "kmipkit_ttlv::Item",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "ttlv_extern_crate_alias",
        path: "tests/fixtures/execute_boundary/ttlv_extern_crate_alias.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/ttlv_extern_crate_alias.rs"),
        probe: "pub fn accept(value: tt::Item)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "ttlv_cargo_alias_type",
        path: "tests/fixtures/execute_boundary/ttlv_cargo_alias_type.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/ttlv_cargo_alias_type.rs"),
        probe: "pub fn accept(value: ttlv::Item)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_body_input",
        path: "tests/fixtures/execute_boundary/raw_body_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/raw_body_input.rs"),
        probe: "body: &[u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_structure_input",
        path: "tests/fixtures/execute_boundary/public_structure_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_structure_input.rs"),
        probe: "kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_registry_value_validation",
        path: "tests/fixtures/execute_boundary/approved_registry_value_validation.rs",
        source: "pub fn validate_extension_value(registry: &ClientExtensionRegistry, identity: ExtensionIdentity, value: kmipkit_ttlv::Structure, limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<RegisteredExtensionValue, ClientError> { loop {} }",
        probe: "value: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "registry_value_validation_wrong_module",
        path: "tests/fixtures/execute_boundary/registry_value_validation_wrong_module.rs",
        source: "pub fn validate_extension_value(registry: &ClientExtensionRegistry, identity: ExtensionIdentity, value: kmipkit_ttlv::Structure, limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<RegisteredExtensionValue, ClientError> { loop {} }",
        probe: "value: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "registry_value_validation_wrong_argument",
        path: "tests/fixtures/execute_boundary/registry_value_validation_wrong_argument.rs",
        source: "pub fn validate_extension_value(registry: &ClientExtensionRegistry, identity: ExtensionIdentity, payload: kmipkit_ttlv::Structure, limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<RegisteredExtensionValue, ClientError> { loop {} }",
        probe: "payload: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "registry_value_validation_wrong_output",
        path: "tests/fixtures/execute_boundary/registry_value_validation_wrong_output.rs",
        source: "pub fn validate_extension_value(registry: &ClientExtensionRegistry, identity: ExtensionIdentity, value: kmipkit_ttlv::Structure, limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<kmipkit_ttlv::Item, ClientError> { loop {} }",
        probe: "kmipkit_ttlv::Item",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_registry_inspect",
        path: "tests/fixtures/execute_boundary/approved_registry_inspect.rs",
        source: "pub fn inspect(registry: &ClientExtensionRegistry, vendor_identifier: &str, value: kmipkit_ttlv::Structure, codec_limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<ExtensionRecognition, ClientError> { loop {} }",
        probe: "value: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "registry_inspect_wrong_module",
        path: "tests/fixtures/execute_boundary/registry_inspect_wrong_module.rs",
        source: "pub fn inspect(registry: &ClientExtensionRegistry, vendor_identifier: &str, value: kmipkit_ttlv::Structure, codec_limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<ExtensionRecognition, ClientError> { loop {} }",
        probe: "value: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "registry_inspect_wrong_argument",
        path: "tests/fixtures/execute_boundary/registry_inspect_wrong_argument.rs",
        source: "pub fn inspect(registry: &ClientExtensionRegistry, vendor_identifier: &str, payload: kmipkit_ttlv::Structure, codec_limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<ExtensionRecognition, ClientError> { loop {} }",
        probe: "payload: kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "registry_inspect_wrong_output",
        path: "tests/fixtures/execute_boundary/registry_inspect_wrong_output.rs",
        source: "pub fn inspect(registry: &ClientExtensionRegistry, vendor_identifier: &str, value: kmipkit_ttlv::Structure, codec_limits: &kmipkit_ttlv::codec::CodecLimits) -> Result<kmipkit_ttlv::Structure, ClientError> { loop {} }",
        probe: "Result<kmipkit_ttlv::Structure, ClientError>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_registry_generic_accessor",
        path: "tests/fixtures/execute_boundary/approved_registry_generic_accessor.rs",
        source: "pub fn generic_value(recognition: &ExtensionRecognition) -> &kmipkit_ttlv::Structure { loop {} }",
        probe: "-> &kmipkit_ttlv::Structure",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "registry_generic_accessor_wrong_input",
        path: "tests/fixtures/execute_boundary/registry_generic_accessor_wrong_input.rs",
        source: "pub fn generic_value(recognition: ExtensionRecognition) -> &kmipkit_ttlv::Structure { loop {} }",
        probe: "recognition: ExtensionRecognition",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "registry_generic_accessor_wrong_output",
        path: "tests/fixtures/execute_boundary/registry_generic_accessor_wrong_output.rs",
        source: "pub fn generic_value(recognition: &ExtensionRecognition) -> kmipkit_ttlv::Item { loop {} }",
        probe: "-> kmipkit_ttlv::Item",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_structure_view_input",
        path: "tests/fixtures/execute_boundary/public_structure_view_input.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/public_structure_view_input.rs"
        ),
        probe: "StructureView<'_>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_structure_view_output",
        path: "tests/fixtures/execute_boundary/public_structure_view_output.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/public_structure_view_output.rs"
        ),
        probe: "-> kmipkit_ttlv::StructureView<'static>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_tag_input",
        path: "tests/fixtures/execute_boundary/public_tag_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_tag_input.rs"),
        probe: "kmipkit_ttlv::Tag",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_item_type_input",
        path: "tests/fixtures/execute_boundary/public_item_type_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_item_type_input.rs"),
        probe: "kmipkit_ttlv::ItemType",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_value_view_input",
        path: "tests/fixtures/execute_boundary/public_value_view_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_value_view_input.rs"),
        probe: "kmipkit_ttlv::ValueView",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "structure_view_wrong_callback",
        path: "tests/fixtures/execute_boundary/structure_view_wrong_callback.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/structure_view_wrong_callback.rs"
        ),
        probe: "fn with_view",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_extension_view_callback",
        path: "tests/fixtures/execute_boundary/approved_extension_view_callback.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/approved_extension_view_callback.rs"
        ),
        probe: "fn with_ttlv",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "approved_async_outcome_callbacks",
        path: "tests/fixtures/execute_boundary/approved_async_outcome_callbacks.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/approved_async_outcome_callbacks.rs"
        ),
        probe: "fn with_asynchronous_correlation_value",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "async_outcome_callback_escape",
        path: "tests/fixtures/execute_boundary/async_outcome_callback_escape.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/async_outcome_callback_escape.rs"
        ),
        probe: "pub fn correlation_bytes",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "owned_vec_bytes_input",
        path: "tests/fixtures/execute_boundary/owned_vec_bytes_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/owned_vec_bytes_input.rs"),
        probe: "body: Vec<u8>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "boxed_byte_slice_input",
        path: "tests/fixtures/execute_boundary/boxed_byte_slice_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/boxed_byte_slice_input.rs"),
        probe: "Box<[u8]>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "fixed_array_bytes_input",
        path: "tests/fixtures/execute_boundary/fixed_array_bytes_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/fixed_array_bytes_input.rs"),
        probe: "body: [u8; 32]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "exact_unique_batch_id_setter",
        path: "tests/fixtures/execute_boundary/exact_unique_batch_id_setter.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/exact_unique_batch_id_setter.rs"
        ),
        probe: "with_unique_batch_item_id",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "other_owned_bytes_setter",
        path: "tests/fixtures/execute_boundary/other_owned_bytes_setter.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/other_owned_bytes_setter.rs"),
        probe: "with_bytes",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_enum_input",
        path: "tests/fixtures/execute_boundary/public_enum_input.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_enum_input.rs"),
        probe: "pub enum RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_type_alias",
        path: "tests/fixtures/execute_boundary/public_type_alias.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_type_alias.rs"),
        probe: "pub type RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "item_reexport_alias",
        path: "tests/fixtures/execute_boundary/item_reexport_alias.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/item_reexport_alias.rs"),
        probe: "Item as RequestInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_trait_transport_input",
        path: "tests/fixtures/execute_boundary/public_trait_transport_input.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/public_trait_transport_input.rs"
        ),
        probe: "trait TransportFactory",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_writer",
        path: "tests/fixtures/execute_boundary/public_writer.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_writer.rs"),
        probe: "pub fn encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_exchange_outside_execute",
        path: "tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/raw_exchange_outside_execute.rs"
        ),
        probe: "transport.exchange(request, max_response_bytes)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "raw_exchange_ufcs_outside_execute",
        path: "tests/fixtures/execute_boundary/raw_exchange_ufcs_outside_execute.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/raw_exchange_ufcs_outside_execute.rs"
        ),
        probe: "Transport::exchange(&mut self.transport, request, max_response_bytes)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "qself_exchange_method_item",
        path: "tests/fixtures/execute_boundary/qself_exchange_method_item.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/qself_exchange_method_item.rs"),
        probe: "<dyn Transport>::exchange",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "macro_hidden_second_exchange",
        path: "tests/fixtures/execute_boundary/macro_hidden_second_exchange.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/macro_hidden_second_exchange.rs"
        ),
        probe: "vec![self.transport.exchange(&[], 1)]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "nested_macro_in_whitelisted_macro",
        path: "tests/fixtures/execute_boundary/nested_macro_in_whitelisted_macro.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/nested_macro_in_whitelisted_macro.rs"
        ),
        probe: "matches!(hidden!(), _)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "matches_guard_logical_not",
        path: "tests/fixtures/execute_boundary/matches_guard_logical_not.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/matches_guard_logical_not.rs"),
        probe: "matches!(value, Some(item) if !(item == 0))",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "public_conversion_hooks",
        path: "tests/fixtures/execute_boundary/public_conversion_hooks.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/public_conversion_hooks.rs"),
        probe: "T: Into<ClientRequest>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "client_request_conversion_trait_impls",
        path: "tests/fixtures/execute_boundary/client_request_conversion_trait_impls.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/client_request_conversion_trait_impls.rs"
        ),
        probe: "impl From<Vec<u8>> for ClientRequest",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "client_batch_conversion_trait_impl",
        path: "tests/fixtures/execute_boundary/client_batch_conversion_trait_impl.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/client_batch_conversion_trait_impl.rs"
        ),
        probe: "impl From<Vec<u8>> for ClientBatch",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "client_batch_item_conversion_trait_impl",
        path: "tests/fixtures/execute_boundary/client_batch_item_conversion_trait_impl.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/client_batch_item_conversion_trait_impl.rs"
        ),
        probe: "impl TryFrom<Structure> for ClientBatchItem",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "client_batch_trait_argument_conversion_impl",
        path: "tests/fixtures/execute_boundary/client_batch_trait_argument_conversion_impl.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/client_batch_trait_argument_conversion_impl.rs"
        ),
        probe: "impl Into<ClientBatch> for CallerInput",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_impl_trait_conversion_input",
        path: "tests/fixtures/execute_boundary/public_impl_trait_conversion_input.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/public_impl_trait_conversion_input.rs"
        ),
        probe: "request: impl Into<ClientRequest>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_error_validation_source",
        path: "tests/fixtures/execute_boundary/approved_error_validation_source.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/approved_error_validation_source.rs"
        ),
        probe: "pub fn validation<E>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "approved_batch_response_iter_output",
        path: "tests/fixtures/execute_boundary/approved_batch_response_iter_output.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/approved_batch_response_iter_output.rs"
        ),
        probe: "impl ExactSizeIterator<Item = &ClientBatchItemResponse>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "unbounded_public_type_parameter",
        path: "tests/fixtures/execute_boundary/unbounded_public_type_parameter.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/unbounded_public_type_parameter.rs"
        ),
        probe: "pub fn execute<T>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "nongeneric_bytes_input_output",
        path: "tests/fixtures/execute_boundary/nongeneric_bytes_input_output.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/nongeneric_bytes_input_output.rs"
        ),
        probe: "bytes::Bytes",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_qself_mint",
        path: "tests/fixtures/execute_boundary/permit_qself_mint.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/permit_qself_mint.rs"),
        probe: "<OperationEncodingPermit>::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_constructor_derives",
        path: "tests/fixtures/execute_boundary/permit_constructor_derives.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/permit_constructor_derives.rs"),
        probe: "derive(Default, Clone, Copy)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_constructor_qualified_derives",
        path: "tests/fixtures/execute_boundary/permit_constructor_qualified_derives.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/permit_constructor_qualified_derives.rs"
        ),
        probe: "derive(core::default::Default, core::clone::Clone)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_constructor_trait_impls",
        path: "tests/fixtures/execute_boundary/permit_constructor_trait_impls.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/permit_constructor_trait_impls.rs"
        ),
        probe: "impl Default for OperationEncodingPermit",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "counterfeit_exception_types",
        path: "tests/fixtures/execute_boundary/counterfeit_exception_types.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/counterfeit_exception_types.rs"
        ),
        probe: "with_ttlv",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "approved_batch_from_items_iterator",
        path: "tests/fixtures/execute_boundary/approved_batch_from_items_iterator.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/approved_batch_from_items_iterator.rs"
        ),
        probe: "from_items(items: impl IntoIterator",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Accept,
    },
    Fixture {
        id: "client_raw_body_execute",
        path: "tests/fixtures/execute_boundary/client_raw_body_execute.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/client_raw_body_execute.rs"),
        probe: "self.transport.exchange(caller_body)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "public_client_transport_injection",
        path: "tests/fixtures/execute_boundary/public_client_transport_injection.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/public_client_transport_injection.rs"
        ),
        probe: "pub fn with_transport<T: Transport>",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "facade_transport_reexport",
        path: "tests/fixtures/execute_boundary/facade_transport_reexport.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/facade_transport_reexport.rs"),
        probe: "pub use kmipkit_transport::Transport",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "missing_permit",
        path: "tests/fixtures/execute_boundary/missing_permit.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/missing_permit.rs"),
        probe: "self.writer.encode(request)",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_permit",
        path: "tests/fixtures/execute_boundary/duplicate_permit.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/duplicate_permit.rs"),
        probe: "let second = OperationEncodingPermit::mint()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_outside_execute",
        path: "tests/fixtures/execute_boundary/permit_outside_execute.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/permit_outside_execute.rs"),
        probe: "fn prepare_permit()",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "duplicate_writer",
        path: "tests/fixtures/execute_boundary/duplicate_writer.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/duplicate_writer.rs"),
        probe: "self.writer.encode(request, permit);\n        self.writer.encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_outside_execute",
        path: "tests/fixtures/execute_boundary/writer_outside_execute.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/writer_outside_execute.rs"),
        probe: "fn submit",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_alias",
        path: "tests/fixtures/execute_boundary/writer_alias.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/writer_alias.rs"),
        probe: "as renamed_encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "writer_reexport",
        path: "tests/fixtures/execute_boundary/writer_reexport.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/writer_reexport.rs"),
        probe: "pub use crate::private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "macro_token_tree",
        path: "tests/fixtures/execute_boundary/macro_token_tree.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/macro_token_tree.rs"),
        probe: "macro_rules! hidden_writer_call",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "qualified_custom_vec_macro",
        path: "tests/fixtures/execute_boundary/qualified_custom_vec_macro.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/qualified_custom_vec_macro.rs"),
        probe: "untrusted::vec![0u8]",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "glob_writer_import_with_direct_permit",
        path: "tests/fixtures/execute_boundary/glob_writer_import_with_direct_permit.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/glob_writer_import_with_direct_permit.rs"
        ),
        probe: "use crate::private_wire_writer::*",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "permit_mint_and_helper_struct_literals",
        path: "tests/fixtures/execute_boundary/permit_mint_and_helper_struct_literals.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/permit_mint_and_helper_struct_literals.rs"
        ),
        probe: "fn helper() -> Self",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "include_bypass",
        path: "tests/fixtures/execute_boundary/include_bypass.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/include_bypass.rs"),
        probe: "include!(\"included_writer.rs\")",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "included_writer_source",
        path: "tests/fixtures/execute_boundary/included_writer.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/included_writer.rs"),
        probe: "private_wire_writer::encode",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "generated_source",
        path: "tests/fixtures/execute_boundary/generated_source.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/generated_source.rs"),
        probe: "OUT_DIR",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "conditional_compilation",
        path: "tests/fixtures/execute_boundary/conditional_compilation.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/conditional_compilation.rs"),
        probe: "#[cfg(any(",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_request_time_stamp",
        path: "tests/fixtures/execute_boundary/log_request_time_stamp.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/log_request_time_stamp.rs"),
        probe: "request_time_stamp = ?request_time_stamp",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "log_asynchronous_correlation_value",
        path: "tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs",
        source: include_str!(
            "../../tests/fixtures/execute_boundary/log_asynchronous_correlation_value.rs"
        ),
        probe: "asynchronous_correlation_value = ?value",
        coverage: SourceCoverage::CandidateInspected,
        expected: ExpectedDecision::Reject,
    },
    Fixture {
        id: "uninspected_construct",
        path: "tests/fixtures/execute_boundary/uninspected_construct.rs",
        source: include_str!("../../tests/fixtures/execute_boundary/uninspected_construct.rs"),
        probe: "opaque_compiler_construct!",
        coverage: SourceCoverage::Uninspected,
        expected: ExpectedDecision::Reject,
    },
];

const EXPECTED_FIXTURE_IDS: &[&str] = &[
    "valid_execute",
    "canonical_vec_macro",
    "generic_item_input",
    "ttlv_extern_crate_alias",
    "ttlv_cargo_alias_type",
    "raw_body_input",
    "public_structure_input",
    "approved_registry_value_validation",
    "registry_value_validation_wrong_module",
    "registry_value_validation_wrong_argument",
    "registry_value_validation_wrong_output",
    "approved_registry_inspect",
    "registry_inspect_wrong_module",
    "registry_inspect_wrong_argument",
    "registry_inspect_wrong_output",
    "approved_registry_generic_accessor",
    "registry_generic_accessor_wrong_input",
    "registry_generic_accessor_wrong_output",
    "public_structure_view_input",
    "public_structure_view_output",
    "public_tag_input",
    "public_item_type_input",
    "public_value_view_input",
    "structure_view_wrong_callback",
    "approved_extension_view_callback",
    "approved_async_outcome_callbacks",
    "async_outcome_callback_escape",
    "owned_vec_bytes_input",
    "boxed_byte_slice_input",
    "fixed_array_bytes_input",
    "exact_unique_batch_id_setter",
    "other_owned_bytes_setter",
    "public_enum_input",
    "public_type_alias",
    "item_reexport_alias",
    "public_trait_transport_input",
    "public_writer",
    "raw_exchange_outside_execute",
    "raw_exchange_ufcs_outside_execute",
    "qself_exchange_method_item",
    "macro_hidden_second_exchange",
    "nested_macro_in_whitelisted_macro",
    "matches_guard_logical_not",
    "public_conversion_hooks",
    "client_request_conversion_trait_impls",
    "client_batch_conversion_trait_impl",
    "client_batch_item_conversion_trait_impl",
    "client_batch_trait_argument_conversion_impl",
    "public_impl_trait_conversion_input",
    "approved_error_validation_source",
    "approved_batch_response_iter_output",
    "unbounded_public_type_parameter",
    "nongeneric_bytes_input_output",
    "permit_qself_mint",
    "permit_constructor_derives",
    "permit_constructor_qualified_derives",
    "permit_constructor_trait_impls",
    "counterfeit_exception_types",
    "approved_batch_from_items_iterator",
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
    "qualified_custom_vec_macro",
    "glob_writer_import_with_direct_permit",
    "permit_mint_and_helper_struct_literals",
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

#[derive(Default)]
struct BoundaryAudit {
    rejection_reasons: Vec<String>,
    permit_calls: usize,
    writer_calls: usize,
    misplaced_permit_calls: usize,
    misplaced_writer_calls: usize,
    misplaced_exchange_calls: usize,
    exchange_calls: usize,
    repeatable_execution_depth: usize,
    sensitive_calls_in_repeatable_context: usize,
    permit_struct_constructions: usize,
    misplaced_permit_struct_constructions: usize,
    current_impl_type: Option<String>,
    current_function: Option<String>,
    current_source_path: Option<PathBuf>,
    current_inline_modules: Vec<String>,
    allowed_call_target: bool,
    logged_sensitive_value: bool,
}

impl BoundaryAudit {
    fn visit_source_file(&mut self, file: &syn::File, source_path: &Path) {
        let previous_source_path = self.current_source_path.replace(source_path.to_path_buf());
        let previous_inline_modules = std::mem::take(&mut self.current_inline_modules);
        visit::visit_file(self, file);
        self.current_source_path = previous_source_path;
        self.current_inline_modules = previous_inline_modules;
    }

    fn is_execute_root_scope(&self) -> bool {
        self.is_source_root(Path::new("execute.rs"))
    }

    fn is_source_root(&self, source_path: &Path) -> bool {
        self.current_source_path.as_deref() == Some(source_path)
            && self.current_inline_modules.is_empty()
    }

    fn reject(&mut self, reason: impl Into<String>) {
        self.rejection_reasons.push(reason.into());
    }

    fn record_repeatable_sensitive_call(&mut self) {
        if self.repeatable_execution_depth > 0 {
            self.sensitive_calls_in_repeatable_context += 1;
            self.reject(
                "writer, permit, or transport call appears in repeatable execution context",
            );
        }
    }

    fn is_rejected(&self) -> bool {
        !self.rejection_reasons.is_empty()
    }

    fn check_attributes(&mut self, attributes: &[Attribute]) {
        for attribute in attributes {
            let path = attribute.path();
            let name = path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            if path.is_ident("cfg") {
                if !supported_cfg(attribute) {
                    self.reject(format!("unsupported cfg attribute: {name}"));
                }
            } else if path.is_ident("cfg_attr") {
                self.reject("cfg_attr is unsupported");
            } else if path.is_ident("derive") {
                if !supported_derive(attribute) {
                    self.reject(format!("unsupported derive: {name}"));
                }
            } else if path.is_ident("path")
                || path.is_ident("doc")
                || path.is_ident("allow")
                || path.is_ident("warn")
                || path.is_ident("deny")
                || path.is_ident("forbid")
                || path.is_ident("must_use")
                || path.is_ident("non_exhaustive")
                || path.is_ident("inline")
            {
                // These built-in attributes do not inject Rust source or macro code.
            } else {
                // Unknown attributes may invoke unreviewed procedural macros.
                self.reject(format!("unknown attribute: {name}"));
            }
        }
    }

    fn check_public_signature(&mut self, signature: &syn::Signature, impl_type: Option<&str>) {
        let mut finder = PublicTypeFinder::default();
        let execute_root_scope = self.is_execute_root_scope();
        let error_root_scope = self.is_source_root(Path::new("error.rs"));
        let unique_id_setter =
            is_exact_unique_batch_id_setter(execute_root_scope, impl_type, signature);
        let extension_view_callback =
            is_exact_extension_view_callback(execute_root_scope, impl_type, signature);
        let async_outcome_callback =
            is_exact_async_outcome_callback(execute_root_scope, impl_type, signature);
        let batch_from_items = is_exact_batch_from_items(execute_root_scope, impl_type, signature);
        let error_validation_source =
            is_exact_error_validation_source(error_root_scope, impl_type, signature);
        let batch_response_iterator =
            is_exact_batch_response_iterator(execute_root_scope, impl_type, signature);
        let registry_value_validation = is_exact_registry_value_validation(
            self.is_source_root(Path::new("extension_registry.rs")),
            impl_type,
            signature,
        );
        let approved_generic_signature =
            extension_view_callback || async_outcome_callback || error_validation_source;
        if has_public_type_or_const_generics(signature) && !approved_generic_signature {
            finder
                .violations
                .insert(PublicTypeViolation::CallerConversion);
        }
        if !error_validation_source {
            finder.visit_generics(&signature.generics);
        }
        for (index, argument) in signature.inputs.iter().enumerate() {
            if let syn::FnArg::Typed(argument) = argument {
                if (unique_id_setter || extension_view_callback || async_outcome_callback)
                    && index == 1
                    || batch_from_items && index == 0
                    || registry_value_validation && index == 2
                {
                    continue;
                }
                finder.visit_type(&argument.ty);
            }
        }
        let mut output_finder = PublicTypeFinder::default();
        if !batch_response_iterator && let syn::ReturnType::Type(_, output) = &signature.output {
            output_finder.visit_type(output);
        }
        if finder.has_forbidden_type() || output_finder.has_forbidden_output_type() {
            self.reject(format!(
                "public signature exposes a forbidden type (input={:?}, output={:?})",
                finder.violations, output_finder.violations
            ));
        }
    }

    fn finish_fixture(&self) -> Result<(), CandidateRejection> {
        if self.is_rejected()
            || self.misplaced_permit_calls != 0
            || self.misplaced_writer_calls != 0
            || self.misplaced_exchange_calls != 0
            || self.sensitive_calls_in_repeatable_context != 0
            || self.misplaced_permit_struct_constructions != 0
            || self.exchange_calls != 1
            || self.permit_calls != self.writer_calls
            || self.permit_calls > 1
        {
            Err(CandidateRejection::BoundaryViolation)
        } else {
            Ok(())
        }
    }
}

impl<'ast> Visit<'ast> for BoundaryAudit {
    fn visit_attribute(&mut self, attribute: &'ast Attribute) {
        self.check_attributes(std::slice::from_ref(attribute));
        visit::visit_attribute(self, attribute);
    }

    fn visit_item_mod(&mut self, module: &'ast ItemMod) {
        // External module discovery is performed separately from AST policy.
        self.check_attributes(&module.attrs);
        if module.content.is_some() && !is_test_cfg(&module.attrs) {
            let previous_depth = self.current_inline_modules.len();
            self.current_inline_modules.push(module.ident.to_string());
            visit::visit_item_mod(self, module);
            self.current_inline_modules.truncate(previous_depth);
        }
    }

    fn visit_item_macro(&mut self, item: &'ast syn::ItemMacro) {
        // Declarative/procedural item macros have opaque expansion behavior.
        if is_sensitive_log_macro(&item.mac.path)
            && contains_logged_sensitive_value(&item.mac.tokens.to_string())
        {
            self.logged_sensitive_value = true;
        }
        self.reject("item macro has opaque expansion");
    }

    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        self.check_attributes(&function.attrs);
        if is_test_cfg(&function.attrs) {
            return;
        }
        if matches!(function.vis, syn::Visibility::Public(_))
            && function.sig.ident.to_string().starts_with("encode")
        {
            self.reject("public writer function");
        }
        if matches!(function.vis, syn::Visibility::Public(_)) {
            self.check_public_signature(&function.sig, None);
        }
        let previous = self
            .current_function
            .replace(function.sig.ident.to_string());
        visit::visit_item_fn(self, function);
        self.current_function = previous;
    }

    fn visit_impl_item_fn(&mut self, function: &'ast syn::ImplItemFn) {
        self.check_attributes(&function.attrs);
        if is_test_cfg(&function.attrs) {
            return;
        }
        if matches!(function.vis, syn::Visibility::Public(_)) {
            let impl_type = self.current_impl_type.clone();
            self.check_public_signature(&function.sig, impl_type.as_deref());
        }
        let previous = self
            .current_function
            .replace(function.sig.ident.to_string());
        visit::visit_impl_item_fn(self, function);
        self.current_function = previous;
    }

    fn visit_item_impl(&mut self, implementation: &'ast syn::ItemImpl) {
        self.check_attributes(&implementation.attrs);
        if is_request_model_conversion_impl(implementation) {
            self.reject(
                "conversion trait implementation exposes a closed request model to caller-defined input",
            );
        }
        if implementation.trait_.is_some() {
            let mut permit_references = PermitReferenceFinder::default();
            permit_references.visit_item_impl(implementation);
            if permit_references.found {
                self.reject("OperationEncodingPermit cannot participate in a trait implementation");
            }
        }
        let previous = std::mem::replace(
            &mut self.current_impl_type,
            type_path_name(&implementation.self_ty),
        );
        visit::visit_item_impl(self, implementation);
        self.current_impl_type = previous;
    }

    fn visit_item_struct(&mut self, structure: &'ast syn::ItemStruct) {
        self.check_attributes(&structure.attrs);
        if is_test_cfg(&structure.attrs) {
            return;
        }
        if structure.ident == "OperationEncodingPermit"
            && structure.attrs.iter().any(permit_enabling_derive)
        {
            self.reject("OperationEncodingPermit cannot derive Default, Clone, or Copy");
        }
        if matches!(structure.vis, syn::Visibility::Public(_)) {
            for field in &structure.fields {
                if matches!(field.vis, syn::Visibility::Public(_)) {
                    let mut finder = PublicTypeFinder::default();
                    finder.visit_type(&field.ty);
                    if finder.has_forbidden_type() {
                        self.reject("public struct field exposes forbidden input");
                    }
                }
            }
        }
        visit::visit_item_struct(self, structure);
    }

    fn visit_item_enum(&mut self, enumeration: &'ast syn::ItemEnum) {
        self.check_attributes(&enumeration.attrs);
        if is_test_cfg(&enumeration.attrs) {
            return;
        }
        if matches!(enumeration.vis, syn::Visibility::Public(_)) {
            for field in enumeration
                .variants
                .iter()
                .flat_map(|variant| variant.fields.iter())
            {
                let mut finder = PublicTypeFinder::default();
                finder.visit_type(&field.ty);
                if finder.has_forbidden_type() {
                    self.reject("public enum variant exposes forbidden input");
                }
            }
        }
        visit::visit_item_enum(self, enumeration);
    }

    fn visit_item_type(&mut self, alias: &'ast syn::ItemType) {
        self.check_attributes(&alias.attrs);
        if !is_test_cfg(&alias.attrs) {
            // Type aliases are not resolved by this syntactic audit and could
            // otherwise conceal a raw Item or byte-slice boundary.
            self.reject("type alias is not inspected by the syntactic boundary audit");
        }
        visit::visit_item_type(self, alias);
    }

    fn visit_item_union(&mut self, union: &'ast syn::ItemUnion) {
        self.check_attributes(&union.attrs);
        if !is_test_cfg(&union.attrs) {
            let mut finder = PublicTypeFinder::default();
            for field in &union.fields.named {
                finder.visit_type(&field.ty);
            }
            if finder.has_forbidden_type() {
                self.reject("union exposes forbidden input");
            }
        }
        visit::visit_item_union(self, union);
    }

    fn visit_item_trait(&mut self, trait_item: &'ast syn::ItemTrait) {
        self.check_attributes(&trait_item.attrs);
        if is_test_cfg(&trait_item.attrs) {
            return;
        }
        if matches!(trait_item.vis, syn::Visibility::Public(_)) {
            let mut finder = PublicTypeFinder::default();
            for bound in &trait_item.supertraits {
                finder.visit_type_param_bound(bound);
            }
            if finder.has_forbidden_type() {
                self.reject("public trait exposes a forbidden bound");
            }
            for member in &trait_item.items {
                match member {
                    syn::TraitItem::Fn(function) if !is_test_cfg(&function.attrs) => {
                        self.check_attributes(&function.attrs);
                        self.check_public_signature(&function.sig, None);
                    }
                    syn::TraitItem::Type(associated) => {
                        let mut finder = PublicTypeFinder::default();
                        finder.visit_generics(&associated.generics);
                        for bound in &associated.bounds {
                            finder.visit_type_param_bound(bound);
                        }
                        if let Some((_, default)) = &associated.default {
                            finder.visit_type(default);
                        }
                        if finder.has_forbidden_type() {
                            self.reject("public trait associated type exposes forbidden input");
                        }
                    }
                    syn::TraitItem::Const(constant) => {
                        let mut finder = PublicTypeFinder::default();
                        finder.visit_type(&constant.ty);
                        if finder.has_forbidden_type() {
                            self.reject("public trait associated constant exposes forbidden type");
                        }
                    }
                    _ => {}
                }
            }
        }
        visit::visit_item_trait(self, trait_item);
    }

    fn visit_item_use(&mut self, item_use: &'ast syn::ItemUse) {
        self.check_attributes(&item_use.attrs);
        if use_tree_imports_client_execute(&item_use.tree) {
            self.reject("execute function imports are not resolved by the boundary audit");
        }
        if is_test_cfg(&item_use.attrs) {
            return;
        }
        if matches!(item_use.vis, syn::Visibility::Public(_))
            && (use_tree_has_name(&item_use.tree, "Transport")
                || use_tree_has_name(&item_use.tree, "Item")
                || use_tree_has_name(&item_use.tree, "encode")
                || use_tree_has_name(&item_use.tree, "encode_for_execute"))
        {
            self.reject("public transport or writer reexport");
        }
        if use_tree_has_glob(&item_use.tree) {
            self.reject("glob imports are not inspected by the boundary audit");
        }
        if use_tree_has_name(&item_use.tree, "renamed_encode")
            || use_tree_has_rename(&item_use.tree)
            || (use_tree_has_name(&item_use.tree, "private_wire_writer")
                && use_tree_has_name(&item_use.tree, "encode"))
            || [
                "OperationEncodingPermit",
                "mint",
                "encode",
                "encode_for_execute",
                "encode_raw",
                "write",
                "matches",
                "vec",
                "Clone",
                "Copy",
                "Debug",
                "Default",
                "Eq",
                "PartialEq",
                "Hash",
            ]
            .iter()
            .any(|protected| use_tree_has_name(&item_use.tree, protected))
        {
            self.reject("writer alias or protected import");
        }
        visit::visit_item_use(self, item_use);
    }

    fn visit_item_extern_crate(&mut self, item: &'ast syn::ItemExternCrate) {
        self.check_attributes(&item.attrs);
        if !is_test_cfg(&item.attrs) && item.rename.is_some() {
            self.reject("extern crate aliases are not resolved by the boundary audit");
        }
        visit::visit_item_extern_crate(self, item);
    }

    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = call.func.as_ref() {
            let segments = path
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>();
            if segments.last().is_some_and(|segment| segment == "execute")
                && (path.qself.is_some()
                    || segments
                        .iter()
                        .any(|segment| matches!(segment.as_str(), "Client" | "Self")))
            {
                self.reject("qualified execute calls must not bypass the boundary audit");
            }
            if segments.last().is_some_and(|segment| segment == "exchange") {
                self.reject(
                    "transport exchange call target must use the canonical method call in Client::exchange_operation",
                );
            }
            if is_permit_mint(&segments) {
                self.permit_calls += 1;
                self.record_repeatable_sensitive_call();
                if self.current_impl_type.as_deref() != Some("Client")
                    || self.current_function.as_deref() != Some("exchange_operation")
                {
                    self.misplaced_permit_calls += 1;
                }
            } else if path_mentions_permit(&segments) {
                self.reject("OperationEncodingPermit is constructed outside mint");
            }
            if is_writer_target(&segments) {
                self.writer_calls += 1;
                self.record_repeatable_sensitive_call();
                if !is_writer_call(&segments) {
                    self.reject("writer function called without its canonical private path");
                }
                if self.current_impl_type.as_deref() != Some("Client")
                    || self.current_function.as_deref() != Some("exchange_operation")
                {
                    self.misplaced_writer_calls += 1;
                }
            }
        }
        let previous = self.allowed_call_target;
        self.allowed_call_target = true;
        self.visit_expr(&call.func);
        self.allowed_call_target = previous;
        for argument in &call.args {
            self.visit_expr(argument);
        }
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if method == "execute" {
            self.reject("execute method calls are not allowed in production client sources");
        }
        if matches!(
            method.as_str(),
            "encode" | "encode_for_execute" | "encode_raw"
        ) {
            self.writer_calls += 1;
            self.record_repeatable_sensitive_call();
            if !is_private_writer_receiver(&call.receiver) {
                self.reject("writer method called through a noncanonical receiver");
            }
            if self.current_impl_type.as_deref() != Some("Client")
                || self.current_function.as_deref() != Some("exchange_operation")
            {
                self.misplaced_writer_calls += 1;
            }
        }
        if method == "exchange" {
            self.exchange_calls += 1;
            self.record_repeatable_sensitive_call();
            if self.current_impl_type.as_deref() != Some("Client")
                || self.current_function.as_deref() != Some("exchange_operation")
                || !is_client_transport_receiver(&call.receiver)
            {
                self.misplaced_exchange_calls += 1;
            }
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_loop(&mut self, expression: &'ast syn::ExprLoop) {
        self.repeatable_execution_depth += 1;
        visit::visit_expr_loop(self, expression);
        self.repeatable_execution_depth -= 1;
    }

    fn visit_expr_while(&mut self, expression: &'ast syn::ExprWhile) {
        self.repeatable_execution_depth += 1;
        visit::visit_expr_while(self, expression);
        self.repeatable_execution_depth -= 1;
    }

    fn visit_expr_for_loop(&mut self, expression: &'ast syn::ExprForLoop) {
        self.repeatable_execution_depth += 1;
        visit::visit_expr_for_loop(self, expression);
        self.repeatable_execution_depth -= 1;
    }

    fn visit_expr_closure(&mut self, expression: &'ast syn::ExprClosure) {
        self.repeatable_execution_depth += 1;
        visit::visit_expr_closure(self, expression);
        self.repeatable_execution_depth -= 1;
    }

    fn visit_expr_struct(&mut self, expression: &'ast syn::ExprStruct) {
        let segments = expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let is_self_literal = expression.path.is_ident("Self");
        let is_permit_impl = self.current_impl_type.as_deref() == Some("OperationEncodingPermit");
        if path_mentions_permit(&segments) || (is_permit_impl && is_self_literal) {
            self.permit_struct_constructions += 1;
            let is_mint_construction = is_permit_impl
                && is_self_literal
                && self.current_function.as_deref() == Some("mint");
            if !is_mint_construction {
                self.misplaced_permit_struct_constructions += 1;
                self.reject("OperationEncodingPermit construction is allowed only in mint");
            }
        }
        visit::visit_expr_struct(self, expression);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if expression
            .qself
            .as_ref()
            .is_some_and(|qself| type_mentions_permit(&qself.ty))
        {
            self.reject("qualified permit paths are not allowed");
        }
        let segments = expression
            .path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        if segments.last().is_some_and(|segment| segment == "execute")
            && (expression.qself.is_some()
                || segments
                    .iter()
                    .any(|segment| matches!(segment.as_str(), "Client" | "Self")))
        {
            self.reject("qualified execute calls must not bypass the boundary audit");
        }
        if segments.last().is_some_and(|segment| segment == "exchange") {
            self.reject(format!(
                "transport exchange path must use the canonical method call in Client::exchange_operation: {segments:?}"
            ));
        }
        if (path_mentions_permit(&segments) || is_writer_target(&segments))
            && !self.allowed_call_target
        {
            self.reject(format!(
                "writer or permit function used as an alias: {segments:?}"
            ));
        }
        visit::visit_expr_path(self, expression);
    }

    fn visit_macro(&mut self, macro_call: &'ast syn::Macro) {
        let tokens = macro_call.tokens.to_string();
        if is_sensitive_log_macro(&macro_call.path) && contains_logged_sensitive_value(&tokens) {
            self.logged_sensitive_value = true;
            self.reject("sensitive request metadata appears in a logger macro");
        }
        if !(macro_call.path.is_ident("write")
            || macro_call.path.is_ident("matches")
            || macro_call.path.is_ident("vec"))
        {
            let path = macro_call
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            self.reject(format!("unsupported macro: {path}"));
        } else if allowed_macro_contains_forbidden_content(macro_call) {
            self.reject(
                "nested macro or execute invocation appears in an allowed macro token tree",
            );
        }
        if contains_protected_macro_tokens(&tokens) {
            self.reject("protected boundary tokens appear in opaque macro input");
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
enum PublicTypeViolation {
    GenericTtlv,
    OwnedBytes,
    RawBody,
    TransportBound,
    CallerConversion,
}

#[derive(Default)]
struct PublicTypeFinder {
    violations: HashSet<PublicTypeViolation>,
}

impl PublicTypeFinder {
    fn has_forbidden_type(&self) -> bool {
        !self.violations.is_empty()
    }

    fn has_forbidden_output_type(&self) -> bool {
        self.violations.iter().any(|violation| {
            matches!(
                violation,
                PublicTypeViolation::GenericTtlv
                    | PublicTypeViolation::OwnedBytes
                    | PublicTypeViolation::TransportBound
                    | PublicTypeViolation::CallerConversion
            )
        })
    }
}

impl<'ast> Visit<'ast> for PublicTypeFinder {
    fn visit_type_param_bound(&mut self, bound: &'ast syn::TypeParamBound) {
        if let syn::TypeParamBound::Trait(trait_bound) = bound {
            self.violations
                .insert(PublicTypeViolation::CallerConversion);
            if trait_bound
                .path
                .segments
                .iter()
                .any(|segment| segment.ident == "Transport")
            {
                self.violations.insert(PublicTypeViolation::TransportBound);
            }
        }
        visit::visit_type_param_bound(self, bound);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if is_generic_ttlv_path(&path.path) {
            self.violations.insert(PublicTypeViolation::GenericTtlv);
        }
        if is_owned_byte_container_path(path) {
            self.violations.insert(PublicTypeViolation::OwnedBytes);
        }
        if path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "Transport")
        {
            self.violations.insert(PublicTypeViolation::TransportBound);
        }
        visit::visit_type_path(self, path);
    }

    fn visit_type_reference(&mut self, reference: &'ast syn::TypeReference) {
        if let Type::Slice(slice) = reference.elem.as_ref()
            && matches!(slice.elem.as_ref(), Type::Path(path) if path.path.is_ident("u8"))
        {
            self.violations.insert(PublicTypeViolation::RawBody);
        }
        visit::visit_type_reference(self, reference);
    }

    fn visit_type_array(&mut self, array: &'ast syn::TypeArray) {
        if is_u8(&array.elem) {
            self.violations.insert(PublicTypeViolation::OwnedBytes);
        }
        visit::visit_type_array(self, array);
    }
}

fn is_generic_ttlv_path(path: &syn::Path) -> bool {
    const GENERIC_TTLV_TYPES: &[&str] = &[
        "Item",
        "Structure",
        "StructureView",
        "Tag",
        "RawTag",
        "ItemType",
        "Value",
        "ValueView",
    ];
    let Some(last) = path.segments.last() else {
        return false;
    };
    let name = last.ident.to_string();
    // Dependency renames and crate aliases change the qualifier, so the public
    // boundary must reject these generic model names regardless of path prefix.
    GENERIC_TTLV_TYPES.contains(&name.as_str())
}

fn is_exact_extension_view_callback(
    execute_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !execute_root_scope
        || impl_type != Some("ClientMessageExtension")
        || signature.ident != "with_ttlv"
        || signature.generics.params.len() != 1
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 2
    {
        return false;
    }

    let generic_is_plain_result = matches!(
        signature.generics.params.first(),
        Some(syn::GenericParam::Type(parameter))
            if parameter.ident == "R"
                && parameter.attrs.is_empty()
                && parameter.colon_token.is_none()
                && parameter.bounds.is_empty()
                && parameter.eq_token.is_none()
                && parameter.default.is_none()
    );
    let mut inputs = signature.inputs.iter();
    let receiver_is_shared_self = matches!(
        inputs.next(),
        Some(syn::FnArg::Receiver(receiver))
            if receiver.reference.as_ref().is_some_and(|(_, lifetime)| lifetime.is_none())
                && receiver.mutability.is_none()
                && receiver.colon_token.is_none()
                && receiver.attrs.is_empty()
    );
    let callback_is_exact = matches!(
        inputs.next(),
        Some(syn::FnArg::Typed(argument))
            if matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern)
                if pattern.ident == "callback"
                    && pattern.by_ref.is_none()
                    && pattern.mutability.is_none()
                    && pattern.subpat.is_none()
                    && pattern.attrs.is_empty())
                && argument.attrs.is_empty()
                && is_exact_extension_view_callback_type(&argument.ty)
    );
    let returns_result = matches!(
        &signature.output,
        syn::ReturnType::Type(_, output) if is_plain_type_path(output, "R")
    );

    generic_is_plain_result && receiver_is_shared_self && callback_is_exact && returns_result
}

fn is_exact_extension_view_callback_type(ty: &Type) -> bool {
    let Type::ImplTrait(implementation_trait) = ty else {
        return false;
    };
    if implementation_trait.bounds.len() != 1 {
        return false;
    }
    let Some(syn::TypeParamBound::Trait(trait_bound)) = implementation_trait.bounds.first() else {
        return false;
    };
    if !matches!(&trait_bound.modifier, syn::TraitBoundModifier::None) {
        return false;
    }
    if trait_bound.path.leading_colon.is_some() {
        return false;
    }
    let Some(lifetimes) = &trait_bound.lifetimes else {
        return false;
    };
    let mut lifetime_parameters = lifetimes.lifetimes.iter();
    let higher_ranked_lifetime_is_plain_a = matches!(
        (lifetime_parameters.next(), lifetime_parameters.next()),
        (Some(syn::GenericParam::Lifetime(parameter)), None)
            if parameter.lifetime.ident == "a"
                && parameter.attrs.is_empty()
                && parameter.colon_token.is_none()
                && parameter.bounds.is_empty()
    );
    let mut path_segments = trait_bound.path.segments.iter();
    let Some(segment) = path_segments.next() else {
        return false;
    };
    if path_segments.next().is_some() {
        return false;
    }
    let syn::PathArguments::Parenthesized(arguments) = &segment.arguments else {
        return false;
    };
    let mut callback_inputs = arguments.inputs.iter();
    let callback_input_is_structure_view = matches!(
        (callback_inputs.next(), callback_inputs.next()),
        (Some(Type::Path(path)), None)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0].ident == "StructureView"
                && matches!(&path.path.segments[0].arguments,
                    syn::PathArguments::AngleBracketed(arguments)
                        if arguments.args.len() == 1
                            && matches!(arguments.args.first(),
                                Some(syn::GenericArgument::Lifetime(lifetime))
                                    if lifetime.ident == "a"))
    );
    let callback_output_is_result = matches!(
        &arguments.output,
        syn::ReturnType::Type(_, output) if is_plain_type_path(output, "R")
    );

    higher_ranked_lifetime_is_plain_a
        && segment.ident == "FnOnce"
        && callback_input_is_structure_view
        && callback_output_is_result
}

fn is_exact_async_outcome_callback(
    execute_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !execute_root_scope
        || impl_type != Some("ClientOperationOutcome")
        || !matches!(
            signature.ident.to_string().as_str(),
            "with_asynchronous_correlation_value" | "with_cancel_echo" | "with_response_payload"
        )
        || signature.generics.params.len() != 1
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 2
    {
        return false;
    }

    let generic_is_plain_result = matches!(
        signature.generics.params.first(),
        Some(syn::GenericParam::Type(parameter))
            if parameter.ident == "R"
                && parameter.attrs.is_empty()
                && parameter.colon_token.is_none()
                && parameter.bounds.is_empty()
                && parameter.eq_token.is_none()
                && parameter.default.is_none()
    );
    let mut inputs = signature.inputs.iter();
    let receiver_is_shared_self = matches!(
        inputs.next(),
        Some(syn::FnArg::Receiver(receiver))
            if receiver.reference.as_ref().is_some_and(|(_, lifetime)| lifetime.is_none())
                && receiver.mutability.is_none()
                && receiver.colon_token.is_none()
                && receiver.attrs.is_empty()
    );
    let callback_is_exact = matches!(
        inputs.next(),
        Some(syn::FnArg::Typed(argument))
            if matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern)
                if pattern.ident == "callback"
                    && pattern.by_ref.is_none()
                    && pattern.mutability.is_none()
                    && pattern.subpat.is_none()
                    && pattern.attrs.is_empty())
                && argument.attrs.is_empty()
                && is_exact_async_outcome_callback_type(
                    &argument.ty,
                    signature.ident == "with_response_payload",
                )
    );
    let returns_option_of_result = matches!(
        &signature.output,
        syn::ReturnType::Type(_, output) if is_exact_option_generic_r(output)
    );

    generic_is_plain_result
        && receiver_is_shared_self
        && callback_is_exact
        && returns_option_of_result
}

fn is_exact_async_outcome_callback_type(ty: &Type, payload: bool) -> bool {
    let Type::ImplTrait(implementation_trait) = ty else {
        return false;
    };
    if implementation_trait.bounds.len() != 1 {
        return false;
    }
    let Some(syn::TypeParamBound::Trait(trait_bound)) = implementation_trait.bounds.first() else {
        return false;
    };
    if !matches!(&trait_bound.modifier, syn::TraitBoundModifier::None)
        || trait_bound.path.leading_colon.is_some()
    {
        return false;
    }
    let Some(lifetimes) = &trait_bound.lifetimes else {
        return false;
    };
    let lifetime_is_plain_a = matches!(
        (lifetimes.lifetimes.iter().next(), lifetimes.lifetimes.iter().nth(1)),
        (Some(syn::GenericParam::Lifetime(parameter)), None)
            if parameter.lifetime.ident == "a"
                && parameter.attrs.is_empty()
                && parameter.colon_token.is_none()
                && parameter.bounds.is_empty()
    );
    let mut path_segments = trait_bound.path.segments.iter();
    let Some(segment) = path_segments.next() else {
        return false;
    };
    if path_segments.next().is_some() || segment.ident != "FnOnce" {
        return false;
    }
    let syn::PathArguments::Parenthesized(arguments) = &segment.arguments else {
        return false;
    };
    let mut callback_inputs = arguments.inputs.iter();
    let Some(callback_input) = callback_inputs.next() else {
        return false;
    };
    if callback_inputs.next().is_some() {
        return false;
    }
    let callback_input_is_exact = if payload {
        matches!(callback_input, Type::Path(path)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0].ident == "StructureView"
                && matches!(&path.path.segments[0].arguments,
                    syn::PathArguments::AngleBracketed(arguments)
                        if arguments.args.len() == 1
                            && matches!(arguments.args.first(),
                                Some(syn::GenericArgument::Lifetime(lifetime))
                                    if lifetime.ident == "a")))
    } else {
        matches!(callback_input, Type::Reference(reference)
            if reference.lifetime.as_ref().is_some_and(|lifetime| lifetime.ident == "a")
                && reference.mutability.is_none()
                && matches!(reference.elem.as_ref(), Type::Slice(slice)
                    if is_u8(&slice.elem)))
    };
    let callback_output_is_result = matches!(
        &arguments.output,
        syn::ReturnType::Type(_, output) if is_plain_type_path(output, "R")
    );

    lifetime_is_plain_a && callback_input_is_exact && callback_output_is_result
}

fn is_exact_option_generic_r(ty: &Type) -> bool {
    matches!(ty, Type::Path(path)
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0].ident == "Option"
            && matches!(&path.path.segments[0].arguments,
                syn::PathArguments::AngleBracketed(arguments)
                    if arguments.args.len() == 1
                        && matches!(arguments.args.first(),
                            Some(syn::GenericArgument::Type(Type::Path(result)))
                                if result.qself.is_none()
                                    && result.path.leading_colon.is_none()
                                    && result.path.segments.len() == 1
                                    && result.path.segments[0].ident == "R"
                                    && matches!(&result.path.segments[0].arguments,
                                        syn::PathArguments::None))))
}

fn is_plain_type_path(ty: &Type, expected: &str) -> bool {
    matches!(ty, Type::Path(path)
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0].ident == expected
            && matches!(&path.path.segments[0].arguments, syn::PathArguments::None))
}

fn is_owned_byte_container(ty: &Type) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    is_owned_byte_container_path(path)
}

fn is_owned_byte_container_path(path: &syn::TypePath) -> bool {
    let Some(segment) = path.path.segments.last() else {
        return false;
    };
    if matches!(segment.ident.to_string().as_str(), "Bytes" | "BytesMut") {
        return true;
    }
    if !matches!(
        segment.ident.to_string().as_str(),
        "Vec" | "Box" | "Cow" | "Arc" | "Rc" | "Bytes" | "BytesMut"
    ) {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    arguments.args.iter().any(|argument| {
        let syn::GenericArgument::Type(argument_type) = argument else {
            return false;
        };
        is_byte_slice(argument_type)
            || is_u8(argument_type)
            || is_owned_byte_container(argument_type)
    })
}

fn is_byte_slice(ty: &Type) -> bool {
    matches!(ty, Type::Slice(slice) if is_u8(&slice.elem))
}

fn is_u8(ty: &Type) -> bool {
    matches!(ty, Type::Path(path) if path.path.is_ident("u8"))
}

fn is_exact_unique_batch_id_setter(
    execute_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !execute_root_scope
        || impl_type != Some("ClientBatchItem")
        || signature.ident != "with_unique_batch_item_id"
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 2
    {
        return false;
    }
    let mut inputs = signature.inputs.iter();
    let Some(syn::FnArg::Receiver(receiver)) = inputs.next() else {
        return false;
    };
    if receiver.reference.is_some() || receiver.colon_token.is_some() {
        return false;
    }
    let Some(syn::FnArg::Typed(argument)) = inputs.next() else {
        return false;
    };
    let correct_name =
        matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern) if pattern.ident == "id");
    let correct_input = matches!(argument.ty.as_ref(), Type::Path(path)
        if path.qself.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0].ident == "Vec"
            && matches!(&path.path.segments[0].arguments, syn::PathArguments::AngleBracketed(arguments)
                if arguments.args.len() == 1
                    && matches!(arguments.args.first(), Some(syn::GenericArgument::Type(ty)) if is_u8(ty))));
    let correct_output = matches!(&signature.output, syn::ReturnType::Type(_, ty)
        if matches!(ty.as_ref(), Type::Path(path) if path.path.is_ident("Self")));
    correct_name && correct_input && correct_output
}

fn is_exact_batch_from_items(
    execute_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !execute_root_scope
        || impl_type != Some("ClientBatch")
        || signature.ident != "from_items"
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 1
        || !matches!(&signature.output, syn::ReturnType::Type(_, ty) if is_plain_type_path(ty, "Self"))
    {
        return false;
    }
    let Some(syn::FnArg::Typed(argument)) = signature.inputs.first() else {
        return false;
    };
    let input_name = matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern)
        if pattern.ident == "items"
            && pattern.by_ref.is_none()
            && pattern.mutability.is_none()
            && pattern.subpat.is_none()
            && pattern.attrs.is_empty());
    let Type::ImplTrait(implementation_trait) = argument.ty.as_ref() else {
        return false;
    };
    if implementation_trait.bounds.len() != 1 {
        return false;
    }
    let Some(syn::TypeParamBound::Trait(trait_bound)) = implementation_trait.bounds.first() else {
        return false;
    };
    if !matches!(&trait_bound.modifier, syn::TraitBoundModifier::None)
        || trait_bound.lifetimes.is_some()
        || trait_bound.path.leading_colon.is_some()
        || trait_bound.path.segments.len() != 1
    {
        return false;
    }
    let segment = &trait_bound.path.segments[0];
    if segment.ident != "IntoIterator" {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return false;
    };
    if arguments.args.len() != 1 {
        return false;
    }
    let Some(syn::GenericArgument::AssocType(binding)) = arguments.args.first() else {
        return false;
    };
    let input_type = binding.ident == "Item"
        && binding.generics.is_none()
        && is_plain_type_path(&binding.ty, "ClientBatchItem");

    input_name && input_type
}

fn has_public_type_or_const_generics(signature: &syn::Signature) -> bool {
    signature.generics.params.iter().any(|parameter| {
        matches!(
            parameter,
            syn::GenericParam::Type(_) | syn::GenericParam::Const(_)
        )
    })
}

fn is_exact_error_validation_source(
    error_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !error_root_scope
        || impl_type != Some("ClientError")
        || signature.ident != "validation"
        || signature.generics.params.len() != 1
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 3
        || !matches!(&signature.output, syn::ReturnType::Type(_, ty) if is_plain_type_path(ty, "Self"))
    {
        return false;
    }

    let generic_is_plain_e = matches!(
        signature.generics.params.first(),
        Some(syn::GenericParam::Type(parameter))
            if parameter.ident == "E"
                && parameter.attrs.is_empty()
                && parameter.colon_token.is_none()
                && parameter.bounds.is_empty()
                && parameter.eq_token.is_none()
                && parameter.default.is_none()
    );
    let where_clause_is_exact_error_bound = signature.generics.where_clause.as_ref().is_some_and(
        |where_clause| {
            if where_clause.predicates.len() != 1 {
                return false;
            }
            let Some(syn::WherePredicate::Type(predicate)) = where_clause.predicates.first() else {
                return false;
            };
            if predicate.lifetimes.is_some()
                || !is_plain_type_path(&predicate.bounded_ty, "E")
                || predicate.bounds.len() != 2
            {
                return false;
            }
            let mut bounds = predicate.bounds.iter();
            let error_bound = matches!(
                (bounds.next(), bounds.next()),
                (
                    Some(syn::TypeParamBound::Trait(trait_bound)),
                    Some(syn::TypeParamBound::Lifetime(lifetime))
                ) if matches!(&trait_bound.modifier, syn::TraitBoundModifier::None)
                    && trait_bound.lifetimes.is_none()
                    && trait_bound.path.leading_colon.is_none()
                    && trait_bound.path.segments.len() == 1
                    && trait_bound.path.segments[0].ident == "Error"
                    && matches!(&trait_bound.path.segments[0].arguments, syn::PathArguments::None)
                    && lifetime.ident == "static"
            );
            error_bound
        },
    );

    let mut inputs = signature.inputs.iter();
    let cause_is_exact = matches!(
        inputs.next(),
        Some(syn::FnArg::Typed(argument))
            if is_named_typed_argument(argument, "cause")
                && is_plain_type_path(&argument.ty, "ClientCauseCategory")
    );
    let delivery_is_exact = matches!(
        inputs.next(),
        Some(syn::FnArg::Typed(argument))
            if is_named_typed_argument(argument, "delivery_state")
                && is_plain_type_path(&argument.ty, "RequestDeliveryState")
    );
    let source_is_exact = matches!(
        inputs.next(),
        Some(syn::FnArg::Typed(argument))
            if is_named_typed_argument(argument, "source")
                && is_plain_type_path(&argument.ty, "E")
    );

    generic_is_plain_e
        && where_clause_is_exact_error_bound
        && cause_is_exact
        && delivery_is_exact
        && source_is_exact
}

fn is_named_typed_argument(argument: &syn::PatType, expected_name: &str) -> bool {
    argument.attrs.is_empty()
        && matches!(argument.pat.as_ref(), syn::Pat::Ident(pattern)
            if pattern.ident == expected_name
                && pattern.by_ref.is_none()
                && pattern.mutability.is_none()
                && pattern.subpat.is_none()
                && pattern.attrs.is_empty())
}

fn is_exact_registry_value_validation(
    registry_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !registry_root_scope
        || impl_type.is_some()
        || signature.ident != "validate_extension_value"
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 4
        || !is_result_with_types(&signature.output, "RegisteredExtensionValue", "ClientError")
    {
        return false;
    }

    let mut inputs = signature.inputs.iter();
    matches!(
        (inputs.next(), inputs.next(), inputs.next(), inputs.next()),
        (
            Some(syn::FnArg::Typed(registry)),
            Some(syn::FnArg::Typed(identity)),
            Some(syn::FnArg::Typed(value)),
            Some(syn::FnArg::Typed(limits)),
        ) if is_named_typed_argument(registry, "registry")
            && is_shared_reference_to(&registry.ty, &["ClientExtensionRegistry"])
            && is_named_typed_argument(identity, "identity")
            && is_path_type(&identity.ty, &["ExtensionIdentity"])
            && is_named_typed_argument(value, "value")
            && is_path_type(&value.ty, &["kmipkit_ttlv", "Structure"])
            && is_named_typed_argument(limits, "limits")
            && is_shared_reference_to(
                &limits.ty,
                &["kmipkit_ttlv", "codec", "CodecLimits"],
            )
    )
}

fn is_result_with_types(output: &syn::ReturnType, ok_type: &str, error_type: &str) -> bool {
    let syn::ReturnType::Type(_, output) = output else {
        return false;
    };
    let Type::Path(path) = output.as_ref() else {
        return false;
    };
    if path.qself.is_some()
        || path.path.leading_colon.is_some()
        || path.path.segments.len() != 1
        || path.path.segments[0].ident != "Result"
    {
        return false;
    }
    let syn::PathArguments::AngleBracketed(arguments) = &path.path.segments[0].arguments else {
        return false;
    };
    if arguments.args.len() != 2 {
        return false;
    }
    matches!(
        (arguments.args.first(), arguments.args.last()),
        (
            Some(syn::GenericArgument::Type(ok)),
            Some(syn::GenericArgument::Type(error)),
        ) if is_path_type(ok, &[ok_type]) && is_path_type(error, &[error_type])
    )
}

fn is_shared_reference_to(ty: &Type, expected_path: &[&str]) -> bool {
    matches!(ty, Type::Reference(reference)
        if reference.mutability.is_none()
            && reference.lifetime.is_none()
            && is_path_type(&reference.elem, expected_path))
}

fn is_path_type(ty: &Type, expected_path: &[&str]) -> bool {
    let Type::Path(path) = ty else {
        return false;
    };
    if path.qself.is_some()
        || path.path.leading_colon.is_some()
        || path.path.segments.len() != expected_path.len()
    {
        return false;
    }
    path.path
        .segments
        .iter()
        .zip(expected_path)
        .all(|(segment, expected)| {
            segment.ident == *expected && matches!(segment.arguments, syn::PathArguments::None)
        })
}

fn is_exact_batch_response_iterator(
    execute_root_scope: bool,
    impl_type: Option<&str>,
    signature: &syn::Signature,
) -> bool {
    if !execute_root_scope
        || impl_type != Some("ClientBatchResponse")
        || signature.ident != "iter"
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.asyncness.is_some()
        || signature.constness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || signature.inputs.len() != 1
        || !matches!(signature.inputs.first(), Some(syn::FnArg::Receiver(receiver))
            if receiver.reference.as_ref().is_some_and(|(_, lifetime)| lifetime.is_none())
                && receiver.mutability.is_none()
                && receiver.colon_token.is_none()
                && receiver.attrs.is_empty())
    {
        return false;
    }
    let syn::ReturnType::Type(_, output) = &signature.output else {
        return false;
    };
    let Type::ImplTrait(implementation_trait) = output.as_ref() else {
        return false;
    };
    if implementation_trait.bounds.len() != 2 {
        return false;
    }
    let mut bounds = implementation_trait.bounds.iter();
    let iterator_bound_is_exact = matches!(bounds.next(),
        Some(syn::TypeParamBound::Trait(trait_bound))
            if matches!(&trait_bound.modifier, syn::TraitBoundModifier::None)
                && trait_bound.lifetimes.is_none()
                && trait_bound.path.leading_colon.is_none()
                && trait_bound.path.segments.len() == 1
                && trait_bound.path.segments[0].ident == "ExactSizeIterator"
                && exact_iterator_item_type(&trait_bound.path.segments[0].arguments));
    let opaque_lifetime_is_exact = matches!(
        bounds.next(),
        Some(syn::TypeParamBound::Lifetime(lifetime)) if lifetime.ident == "_"
    );
    iterator_bound_is_exact && opaque_lifetime_is_exact
}

fn exact_iterator_item_type(arguments: &syn::PathArguments) -> bool {
    let syn::PathArguments::AngleBracketed(arguments) = arguments else {
        return false;
    };
    if arguments.args.len() != 1 {
        return false;
    }
    let Some(syn::GenericArgument::AssocType(binding)) = arguments.args.first() else {
        return false;
    };
    binding.ident == "Item"
        && binding.generics.is_none()
        && matches!(&binding.ty, Type::Reference(reference)
            if reference.lifetime.is_none()
                && reference.mutability.is_none()
                && is_plain_type_path(&reference.elem, "ClientBatchItemResponse"))
}

fn supported_derive(attribute: &Attribute) -> bool {
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    let derives = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        list.tokens.clone(),
    );
    derives.is_ok_and(|derives| {
        !derives.is_empty()
            && derives.iter().all(|derive| {
                derive.segments.len() == 1
                    && matches!(
                        derive.segments[0].ident.to_string().as_str(),
                        "Clone" | "Copy" | "Debug" | "Default" | "Eq" | "PartialEq" | "Hash"
                    )
            })
    })
}

fn type_path_name(ty: &Type) -> Option<String> {
    match ty {
        Type::Path(path) if path.qself.is_none() => Some(
            path.path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect::<Vec<_>>()
                .join("::"),
        ),
        _ => None,
    }
}

const CLOSED_REQUEST_MODEL_NAMES: &[&str] = &["ClientRequest", "ClientBatch", "ClientBatchItem"];

fn is_request_model_conversion_impl(implementation: &syn::ItemImpl) -> bool {
    let Some((_, trait_path, _)) = &implementation.trait_ else {
        return false;
    };
    let Some(trait_segment) = trait_path.segments.last() else {
        return false;
    };
    if !matches!(
        trait_segment.ident.to_string().as_str(),
        "From" | "TryFrom" | "Into" | "TryInto"
    ) {
        return false;
    }

    let mut finder = RequestModelReferenceFinder::default();
    finder.visit_type(&implementation.self_ty);
    finder.visit_path(trait_path);
    finder.found
}

#[derive(Default)]
struct RequestModelReferenceFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for RequestModelReferenceFinder {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path.segments.iter().any(|segment| {
            let name = segment.ident.to_string();
            CLOSED_REQUEST_MODEL_NAMES.contains(&name.as_str())
        }) {
            self.found = true;
        }
        visit::visit_path(self, path);
    }
}

#[derive(Default)]
struct PermitReferenceFinder {
    found: bool,
}

impl<'ast> Visit<'ast> for PermitReferenceFinder {
    fn visit_path(&mut self, path: &'ast syn::Path) {
        if path
            .segments
            .iter()
            .any(|segment| segment.ident == "OperationEncodingPermit")
        {
            self.found = true;
        }
        visit::visit_path(self, path);
    }

    fn visit_type_path(&mut self, path: &'ast syn::TypePath) {
        if path
            .path
            .segments
            .iter()
            .any(|segment| segment.ident == "OperationEncodingPermit")
        {
            self.found = true;
        }
        visit::visit_type_path(self, path);
    }
}

fn type_mentions_permit(ty: &Type) -> bool {
    let mut finder = PermitReferenceFinder::default();
    finder.visit_type(ty);
    finder.found
}

fn permit_enabling_derive(attribute: &Attribute) -> bool {
    if !attribute.path().is_ident("derive") {
        return false;
    }
    let Meta::List(list) = &attribute.meta else {
        return false;
    };
    let derives = syn::parse::Parser::parse2(
        syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
        list.tokens.clone(),
    );
    derives.is_ok_and(|derives| {
        derives.iter().any(|derive| {
            let Some(final_segment) = derive.segments.last() else {
                return false;
            };
            ["Default", "Clone", "Copy"]
                .iter()
                .any(|name| final_segment.ident == *name)
        })
    })
}

fn is_permit_mint(segments: &[String]) -> bool {
    matches!(
        segments,
        [permit, mint] if permit == "OperationEncodingPermit" && mint == "mint"
    )
}

fn path_mentions_permit(segments: &[String]) -> bool {
    segments
        .iter()
        .any(|segment| segment == "OperationEncodingPermit")
}

fn is_writer_target(segments: &[String]) -> bool {
    matches!(
        segments.last().map(String::as_str),
        Some("encode" | "encode_for_execute" | "encode_raw")
    )
}

fn is_writer_call(segments: &[String]) -> bool {
    matches!(
        segments,
        [module, function]
            if module == "private_wire_writer" && function == "encode_for_execute"
    )
}

fn is_private_writer_receiver(receiver: &Expr) -> bool {
    matches!(
        receiver,
        Expr::Field(field)
            if matches!(field.base.as_ref(), Expr::Path(path) if path.path.is_ident("self"))
                && matches!(&field.member, syn::Member::Named(name) if name == "writer")
    )
}

fn is_client_transport_receiver(receiver: &Expr) -> bool {
    matches!(
        receiver,
        Expr::Field(field)
            if matches!(field.base.as_ref(), Expr::Path(path) if path.path.is_ident("self"))
                && matches!(&field.member, syn::Member::Named(name) if name == "transport")
    )
}

fn use_tree_has_name(tree: &UseTree, expected: &str) -> bool {
    match tree {
        UseTree::Path(path) => path.ident == expected || use_tree_has_name(&path.tree, expected),
        UseTree::Name(name) => name.ident == expected,
        UseTree::Rename(rename) => rename.ident == expected || rename.rename == expected,
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|tree| use_tree_has_name(tree, expected)),
        UseTree::Glob(_) => false,
    }
}

fn use_tree_imports_client_execute(tree: &UseTree) -> bool {
    fn visit(tree: &UseTree, prefix: &mut Vec<String>) -> bool {
        match tree {
            UseTree::Path(path) => {
                prefix.push(path.ident.to_string());
                let imported = visit(&path.tree, prefix);
                prefix.pop();
                imported
            }
            UseTree::Name(name) => {
                prefix.push(name.ident.to_string());
                let imported = prefix.len() >= 2
                    && prefix.last().is_some_and(|segment| segment == "execute")
                    && prefix[..prefix.len() - 1]
                        .iter()
                        .any(|segment| matches!(segment.as_str(), "Client" | "Self"));
                prefix.pop();
                imported
            }
            UseTree::Rename(rename) => {
                prefix.push(rename.ident.to_string());
                let imported = prefix.len() >= 2
                    && prefix.last().is_some_and(|segment| segment == "execute")
                    && prefix[..prefix.len() - 1]
                        .iter()
                        .any(|segment| matches!(segment.as_str(), "Client" | "Self"));
                prefix.pop();
                imported
            }
            UseTree::Group(group) => group.items.iter().any(|item| visit(item, prefix)),
            UseTree::Glob(_) => false,
        }
    }

    visit(tree, &mut Vec::new())
}

fn use_tree_has_rename(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => use_tree_has_rename(&path.tree),
        UseTree::Rename(_) => true,
        UseTree::Group(group) => group.items.iter().any(use_tree_has_rename),
        UseTree::Name(_) | UseTree::Glob(_) => false,
    }
}

fn use_tree_has_glob(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(path) => use_tree_has_glob(&path.tree),
        UseTree::Group(group) => group.items.iter().any(use_tree_has_glob),
        UseTree::Glob(_) => true,
        UseTree::Name(_) | UseTree::Rename(_) => false,
    }
}

fn is_sensitive_log_macro(path: &syn::Path) -> bool {
    let segments = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();
    segments.iter().any(|segment| {
        matches!(
            segment.as_str(),
            "trace" | "debug" | "info" | "warn" | "error"
        )
    }) && segments.iter().any(|segment| {
        matches!(
            segment.as_str(),
            "log" | "tracing" | "trace" | "debug" | "info" | "warn" | "error"
        )
    })
}

fn contains_logged_sensitive_value(tokens: &str) -> bool {
    let normalized = tokens.to_ascii_lowercase();
    [
        "time_stamp",
        "timestamp",
        "asynchronous_correlation_value",
        "correlation_value",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn contains_protected_macro_tokens(tokens: &str) -> bool {
    let normalized = tokens.to_ascii_lowercase();
    [
        "operationencodingpermit",
        "private_wire_writer",
        "encode",
        "exchange",
        "mint",
        "asynchronous_correlation_value",
        "time_stamp",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

struct MatchesMacroInput {
    expression: Expr,
    pattern: Pat,
    guard: Option<Expr>,
}

impl Parse for MatchesMacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let expression = input.parse::<Expr>()?;
        input.parse::<syn::Token![,]>()?;
        let pattern = input.call(Pat::parse_multi_with_leading_vert)?;
        let guard = if input.peek(syn::Token![if]) {
            input.parse::<syn::Token![if]>()?;
            Some(input.parse::<Expr>()?)
        } else {
            None
        };
        if !input.is_empty() {
            return Err(input.error("unexpected tokens in matches! input"));
        }
        Ok(Self {
            expression,
            pattern,
            guard,
        })
    }
}

struct VecMacroInput {
    expressions: Vec<Expr>,
}

impl Parse for VecMacroInput {
    fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        if input.is_empty() {
            return Ok(Self {
                expressions: Vec::new(),
            });
        }
        let first = input.parse::<Expr>()?;
        let mut expressions = vec![first];
        if input.peek(syn::Token![;]) {
            input.parse::<syn::Token![;]>()?;
            expressions.push(input.parse::<Expr>()?);
        } else {
            while input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
                if input.is_empty() {
                    break;
                }
                expressions.push(input.parse::<Expr>()?);
            }
        }
        if !input.is_empty() {
            return Err(input.error("unexpected tokens in vec! input"));
        }
        Ok(Self { expressions })
    }
}

#[derive(Default)]
struct MacroBoundaryFinder {
    forbidden_content_found: bool,
}

impl<'ast> Visit<'ast> for MacroBoundaryFinder {
    fn visit_macro(&mut self, _macro_call: &'ast Macro) {
        self.forbidden_content_found = true;
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if call.method == "execute" {
            self.forbidden_content_found = true;
        }
        visit::visit_expr_method_call(self, call);
    }

    fn visit_expr_path(&mut self, expression: &'ast syn::ExprPath) {
        if expression
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "execute")
        {
            self.forbidden_content_found = true;
        }
        visit::visit_expr_path(self, expression);
    }
}

fn allowed_macro_contains_forbidden_content(macro_call: &Macro) -> bool {
    let Some(name) = macro_call
        .path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
    else {
        return true;
    };
    match name.as_str() {
        "matches" => {
            syn::parse2::<MatchesMacroInput>(macro_call.tokens.clone()).map_or(true, |input| {
                let mut finder = MacroBoundaryFinder::default();
                finder.visit_expr(&input.expression);
                finder.visit_pat(&input.pattern);
                if let Some(guard) = &input.guard {
                    finder.visit_expr(guard);
                }
                finder.forbidden_content_found
            })
        }
        "vec" => syn::parse2::<VecMacroInput>(macro_call.tokens.clone()).map_or(true, |input| {
            let mut finder = MacroBoundaryFinder::default();
            for expression in &input.expressions {
                finder.visit_expr(expression);
            }
            finder.forbidden_content_found
        }),
        "write" => {
            let parser = syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated;
            syn::parse::Parser::parse2(parser, macro_call.tokens.clone()).map_or(
                true,
                |expressions| {
                    if expressions.len() < 2 {
                        return true;
                    }
                    let mut finder = MacroBoundaryFinder::default();
                    for expression in &expressions {
                        finder.visit_expr(expression);
                    }
                    finder.forbidden_content_found
                },
            )
        }
        _ => true,
    }
}

fn audit_source(source: &str) -> Result<BoundaryAudit, CandidateRejection> {
    audit_source_at(source, Path::new("fixture.rs"))
}

fn audit_source_at(source: &str, source_path: &Path) -> Result<BoundaryAudit, CandidateRejection> {
    let parsed = syn::parse_file(source).map_err(|_| CandidateRejection::UninspectedSource)?;
    let mut audit = BoundaryAudit::default();
    audit.visit_source_file(&parsed, source_path);
    audit.finish_fixture().map(|()| audit)
}

const CANONICAL_EXECUTE_EXCHANGE_FIXTURE: &str =
    "impl Client { fn exchange_operation(&mut self) { self.transport.exchange(&[], 1); } }";

fn candidate_check_fixture(fixture: &Fixture) -> Result<(), CandidateRejection> {
    if fixture.coverage == SourceCoverage::Uninspected {
        return Err(CandidateRejection::UninspectedSource);
    }
    // Most fixtures isolate one construct, so supply the canonical execute
    // exchange context required by the complete-source candidate checker.
    let complete_source = if fixture.source.contains(".exchange(") {
        fixture.source.to_owned()
    } else {
        format!("{}\n{CANONICAL_EXECUTE_EXCHANGE_FIXTURE}", fixture.source)
    };
    let source_path = if matches!(
        fixture.id,
        "exact_unique_batch_id_setter"
            | "approved_extension_view_callback"
            | "approved_async_outcome_callbacks"
            | "approved_batch_from_items_iterator"
            | "approved_batch_response_iter_output"
            | "counterfeit_exception_types"
    ) {
        Path::new("execute.rs")
    } else if fixture.id == "approved_error_validation_source" {
        Path::new("error.rs")
    } else if matches!(
        fixture.id,
        "approved_registry_value_validation"
            | "approved_registry_inspect"
            | "registry_inspect_wrong_argument"
            | "registry_inspect_wrong_output"
            | "approved_registry_generic_accessor"
            | "registry_generic_accessor_wrong_input"
            | "registry_generic_accessor_wrong_output"
    ) {
        Path::new("extension_registry.rs")
    } else {
        Path::new("fixture.rs")
    };
    audit_source_at(&complete_source, source_path).map(|_| ())
}

fn candidate_check_inventory(inventory: &[&Fixture]) -> Result<(), CandidateRejection> {
    for fixture in inventory {
        if fixture.coverage == SourceCoverage::Uninspected {
            return Err(CandidateRejection::UninspectedSource);
        }
        candidate_check_fixture(fixture)?;
    }
    Ok(())
}

pub(super) fn audit_callsite_for_test(source: &str) -> bool {
    let Ok(parsed) = syn::parse_file(source) else {
        return true;
    };
    let mut audit = BoundaryAudit::default();
    audit.visit_file(&parsed);
    audit.logged_sensitive_value
}

#[derive(Default)]
struct ProductionSources {
    files: Vec<(PathBuf, String)>,
    production_paths: HashSet<PathBuf>,
    test_only_paths: HashSet<PathBuf>,
    rejection_reasons: Vec<String>,
    rejected: bool,
}

fn production_sources() -> ProductionSources {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let Some(src) = manifest.join("src").canonicalize().ok() else {
        return ProductionSources {
            rejected: true,
            ..ProductionSources::default()
        };
    };
    let mut inventory = ProductionSources::default();
    let root = src.join("lib.rs");
    discover_module_file(&root, &src, &mut inventory);

    let mut all_rust_files = Vec::new();
    if collect_rust_files(&src, &mut all_rust_files).is_err() {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push("unable to enumerate src Rust files".to_owned());
    }
    for path in all_rust_files {
        let Ok(path) = path.canonicalize() else {
            inventory.rejected = true;
            continue;
        };
        if !inventory.production_paths.contains(&path) && !inventory.test_only_paths.contains(&path)
        {
            inventory.rejected = true;
            inventory
                .rejection_reasons
                .push(format!("unclassified source: {path:?}"));
        }
    }
    inventory
}

fn collect_rust_files(directory: &Path, output: &mut Vec<PathBuf>) -> std::io::Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, output)?;
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            output.push(path);
        }
    }
    Ok(())
}

fn discover_module_file(path: &Path, src: &Path, inventory: &mut ProductionSources) {
    let Ok(canonical) = path.canonicalize() else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("missing module file: {path:?}"));
        return;
    };
    if !canonical.starts_with(src) {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("module escapes src: {canonical:?}"));
        return;
    }
    if !inventory.production_paths.insert(canonical.clone()) {
        // Rust may not compile one source file into multiple modules here.
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("duplicate module source: {canonical:?}"));
        return;
    }
    let Ok(source) = fs::read_to_string(&canonical) else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("unreadable module source: {canonical:?}"));
        return;
    };
    let Ok(parsed) = syn::parse_file(&source) else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("unparseable module source: {canonical:?}"));
        return;
    };
    let child_base = module_base(&canonical);
    let Some(source_path) = canonical.strip_prefix(src).ok() else {
        inventory.rejected = true;
        inventory
            .rejection_reasons
            .push(format!("module source escapes src: {canonical:?}"));
        return;
    };
    let mut audit = BoundaryAudit::default();
    audit.visit_source_file(&parsed, source_path);
    inventory.rejected |= audit.is_rejected();
    if audit.is_rejected() {
        inventory.rejection_reasons.push(format!(
            "unsupported or unsafe AST construct in {canonical:?}: {:?}",
            audit.rejection_reasons
        ));
    }
    inventory.files.push((canonical.clone(), source));
    discover_items(
        &parsed.items,
        &child_base,
        canonical.parent().unwrap_or(src),
        src,
        inventory,
    );
}

fn discover_items(
    items: &[Item],
    base: &Path,
    attribute_base: &Path,
    src: &Path,
    inventory: &mut ProductionSources,
) {
    for item in items {
        let Item::Mod(module) = item else {
            continue;
        };
        if module.content.is_some() {
            if is_test_cfg(&module.attrs) {
                continue;
            }
            let inline_base = base.join(module.ident.to_string());
            discover_items(
                module
                    .content
                    .as_ref()
                    .map(|(_, items)| items.as_slice())
                    .unwrap_or_default(),
                &inline_base,
                &inline_base,
                src,
                inventory,
            );
            continue;
        }
        let Some(path) = module_file_path(module, base, attribute_base) else {
            inventory.rejected = true;
            inventory
                .rejection_reasons
                .push(format!("unresolved module: {}", module.ident));
            continue;
        };
        if is_test_cfg(&module.attrs) {
            if let Ok(path) = path.canonicalize() {
                inventory.test_only_paths.insert(path);
            } else {
                inventory.rejected = true;
                inventory
                    .rejection_reasons
                    .push(format!("missing test-only module: {path:?}"));
            }
        } else {
            discover_module_file(&path, src, inventory);
        }
    }
}

fn module_file_path(module: &ItemMod, base: &Path, attribute_base: &Path) -> Option<PathBuf> {
    for attribute in &module.attrs {
        if attribute.path().is_ident("path") {
            let Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            let Expr::Lit(value) = &value.value else {
                return None;
            };
            let syn::Lit::Str(value) = &value.lit else {
                return None;
            };
            return Some(attribute_base.join(value.value()));
        }
    }
    let flat = base.join(format!("{}.rs", module.ident));
    if flat.exists() {
        return Some(flat);
    }
    let nested = base.join(module.ident.to_string()).join("mod.rs");
    nested.exists().then_some(nested)
}

fn module_base(file: &Path) -> PathBuf {
    let parent = file.parent().unwrap_or_else(|| Path::new("."));
    if matches!(
        file.file_name().and_then(|name| name.to_str()),
        Some("lib.rs" | "main.rs" | "mod.rs")
    ) {
        parent.to_path_buf()
    } else {
        parent.join(file.file_stem().unwrap_or_default())
    }
}

fn is_test_cfg(attributes: &[Attribute]) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("cfg")
            && matches!(&attribute.meta, Meta::List(list) if list.tokens.to_string() == "test")
    })
}

fn supported_cfg(attribute: &Attribute) -> bool {
    matches!(
        &attribute.meta,
        Meta::List(list) if matches!(list.tokens.to_string().as_str(), "test" | "not (test)")
    )
}

#[test]
fn query_request_is_dropped_before_exchange_and_not_retained_by_the_client() {
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/execute.rs");
    let source = fs::read_to_string(source_path).expect("the client source is readable");
    let syntax = syn::parse_file(&source).expect("the client source parses");
    let body = syntax
        .items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(implementation) => Some(implementation.items.iter()),
            _ => None,
        })
        .flatten()
        .find_map(|item| match item {
            syn::ImplItem::Fn(function) if function.sig.ident == "execute_query_async_requests" => {
                Some(&function.block)
            }
            _ => None,
        })
        .expect("the Query execution method exists");
    let request_drop = body.stmts.iter().position(|statement| match statement {
        syn::Stmt::Expr(Expr::Call(call), _) => {
            let drops_request = matches!(
                call.func.as_ref(),
                Expr::Path(path)
                    if path.path.segments.last().is_some_and(|segment| segment.ident == "drop")
            );
            drops_request
                && call.args.len() == 1
                && matches!(&call.args[0], Expr::Path(path) if path.path.is_ident("request"))
        }
        _ => false,
    });
    let exchange = body.stmts.iter().position(|statement| {
        matches!(
            statement,
            syn::Stmt::Expr(Expr::MethodCall(call), _)
                if call.method == "execute_async_request"
        )
    });

    assert!(
        matches!((request_drop, exchange), (Some(drop), Some(exchange)) if drop < exchange),
        "the consumed Query request and filter values must be dropped before the client exchange"
    );
}

#[test]
fn production_source_inventory_is_complete_and_execute_owns_the_only_writer_permit_pair() {
    let inventory = production_sources();
    assert!(
        !inventory.rejected,
        "production AST inventory failed closed: reasons={:?}, production={:?}, test_only={:?}",
        inventory.rejection_reasons, inventory.production_paths, inventory.test_only_paths
    );
    let paths = inventory
        .files
        .iter()
        .filter_map(|(path, _)| path.file_name().and_then(|name| name.to_str()))
        .collect::<HashSet<_>>();
    for expected in ["lib.rs", "error.rs", "execute.rs", "wire_encoder.rs"] {
        assert!(
            paths.contains(expected),
            "production source missing from audit: {expected}"
        );
    }

    let source_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .canonicalize()
        .expect("the client source root exists");
    let mut audit = BoundaryAudit::default();
    for (path, source) in &inventory.files {
        let parsed = syn::parse_file(source).expect("the discovered production source parsed");
        let relative_path = path
            .strip_prefix(&source_root)
            .expect("production source belongs to the client src root");
        audit.visit_source_file(&parsed, relative_path);
    }
    assert!(
        !audit.is_rejected(),
        "production Rust source contains an uninspected boundary"
    );
    assert_eq!(
        audit.permit_calls, 1,
        "expected one permit mint in production"
    );
    assert_eq!(
        audit.writer_calls, 1,
        "expected one private writer call in production"
    );
    assert_eq!(audit.misplaced_permit_calls, 0);
    assert_eq!(audit.misplaced_writer_calls, 0);
    assert_eq!(audit.exchange_calls, 1, "expected one transport exchange");
    assert_eq!(audit.misplaced_exchange_calls, 0);
    assert_eq!(audit.permit_struct_constructions, 1);
    assert_eq!(audit.misplaced_permit_struct_constructions, 0);
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
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "generic_item_input",
        "raw_body_input",
        "public_structure_input",
        "public_structure_view_input",
        "public_structure_view_output",
        "public_tag_input",
        "public_item_type_input",
        "public_value_view_input",
        "structure_view_wrong_callback",
        "async_outcome_callback_escape",
        "owned_vec_bytes_input",
        "boxed_byte_slice_input",
        "fixed_array_bytes_input",
        "other_owned_bytes_setter",
        "public_enum_input",
        "public_type_alias",
        "item_reexport_alias",
        "public_trait_transport_input",
    ]);

    assert!(
        accepted.is_empty(),
        "accepted forbidden fixtures: {accepted:?}"
    );
}

#[test]
fn registry_validation_generic_input_is_the_only_exact_exception() {
    assert_eq!(
        candidate_check_fixture(fixture("approved_registry_value_validation")),
        Ok(()),
        "the registry may validate a generic subtree only through its exact sealed-value API"
    );

    let rejected = accepted_ids_for_rejected_fixtures(&[
        "registry_value_validation_wrong_module",
        "registry_value_validation_wrong_argument",
        "registry_value_validation_wrong_output",
    ]);
    assert!(
        rejected.is_empty(),
        "accepted non-exact registry validation signatures: {rejected:?}"
    );
}

#[test]
fn registry_inspection_and_generic_accessors_are_the_only_inbound_ttlv_surface() {
    assert_eq!(
        candidate_check_fixture(fixture("approved_registry_inspect")),
        Ok(()),
        "the client may accept a generic subtree only at its exact inspection boundary"
    );
    assert_eq!(
        candidate_check_fixture(fixture("approved_registry_generic_accessor")),
        Ok(()),
        "the recognition result may expose its unchanged generic subtree by borrow"
    );

    let rejected = accepted_ids_for_rejected_fixtures(&[
        "registry_inspect_wrong_module",
        "registry_inspect_wrong_argument",
        "registry_inspect_wrong_output",
        "registry_generic_accessor_wrong_input",
        "registry_generic_accessor_wrong_output",
    ]);
    assert!(
        rejected.is_empty(),
        "accepted non-exact inbound TTLV signatures: {rejected:?}"
    );
}

#[test]
fn extern_crate_alias_cannot_hide_a_generic_ttlv_type() {
    assert_eq!(
        candidate_check_fixture(fixture("ttlv_extern_crate_alias")),
        Err(CandidateRejection::BoundaryViolation),
        "the syntactic boundary audit must not accept crate aliases that hide generic TTLV types"
    );
}

#[test]
fn cargo_dependency_alias_cannot_hide_a_generic_ttlv_type() {
    assert_eq!(
        candidate_check_fixture(fixture("ttlv_cargo_alias_type")),
        Err(CandidateRejection::BoundaryViolation),
        "the syntactic boundary audit must reject protected TTLV type names behind unknown crate aliases"
    );
}

#[test]
fn low_level_exception_does_not_bypass_the_typed_client_boundary() {
    assert_eq!(
        candidate_check_fixture(fixture("raw_exchange_outside_execute")),
        Err(CandidateRejection::BoundaryViolation),
        "client source may exchange only from Client::exchange_operation"
    );
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "raw_exchange_ufcs_outside_execute",
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
fn standard_macros_cannot_hide_a_second_transport_exchange() {
    assert_eq!(
        candidate_check_fixture(fixture("macro_hidden_second_exchange")),
        Err(CandidateRejection::BoundaryViolation),
        "a standard macro must not hide an exchange call from the source audit"
    );
}

#[test]
fn allowed_macro_tokens_cannot_contain_nested_macro_invocations() {
    assert_eq!(
        candidate_check_fixture(fixture("nested_macro_in_whitelisted_macro")),
        Err(CandidateRejection::BoundaryViolation),
        "a whitelisted macro must not hide an expansion-bearing nested macro"
    );
}

#[test]
fn logical_not_in_a_matches_guard_remains_allowed() {
    assert_eq!(
        candidate_check_fixture(fixture("matches_guard_logical_not")),
        Ok(()),
        "a logical negation in a matches guard is not a nested macro invocation"
    );
}

#[test]
fn public_signatures_reject_caller_defined_conversion_hooks() {
    assert_eq!(
        candidate_check_fixture(fixture("public_conversion_hooks")),
        Err(CandidateRejection::BoundaryViolation),
        "public generic conversion hooks must not accept caller-defined request conversions"
    );
}

#[test]
fn client_request_rejects_inbound_conversion_trait_implementations() {
    assert_eq!(
        candidate_check_fixture(fixture("client_request_conversion_trait_impls")),
        Err(CandidateRejection::BoundaryViolation),
        "From/TryFrom implementations must not add caller-defined ClientRequest conversions"
    );
}

#[test]
fn client_batch_rejects_inbound_conversion_trait_implementations() {
    assert_eq!(
        candidate_check_fixture(fixture("client_batch_conversion_trait_impl")),
        Err(CandidateRejection::BoundaryViolation),
        "From implementations must not admit caller-defined ClientBatch inputs"
    );
}

#[test]
fn client_batch_item_rejects_inbound_conversion_trait_implementations() {
    assert_eq!(
        candidate_check_fixture(fixture("client_batch_item_conversion_trait_impl")),
        Err(CandidateRejection::BoundaryViolation),
        "TryFrom implementations must not admit caller-defined ClientBatchItem inputs"
    );
}

#[test]
fn request_model_conversion_trait_arguments_are_checked() {
    assert_eq!(
        candidate_check_fixture(fixture("client_batch_trait_argument_conversion_impl")),
        Err(CandidateRejection::BoundaryViolation),
        "conversion implementations must be rejected when a request model occurs in trait arguments"
    );
}

#[test]
fn public_signatures_reject_impl_trait_conversion_inputs() {
    assert_eq!(
        candidate_check_fixture(fixture("public_impl_trait_conversion_input")),
        Err(CandidateRejection::BoundaryViolation),
        "caller-controlled input impl Trait hooks must stay outside the closed request API"
    );
}

#[test]
fn public_signatures_reject_unbounded_type_parameters() {
    assert_eq!(
        candidate_check_fixture(fixture("unbounded_public_type_parameter")),
        Err(CandidateRejection::BoundaryViolation),
        "an unbounded public type parameter can bypass the closed request boundary"
    );
}

#[test]
fn nongeneric_bytes_types_are_rejected_as_public_inputs_and_outputs() {
    assert_eq!(
        candidate_check_fixture(fixture("nongeneric_bytes_input_output")),
        Err(CandidateRejection::BoundaryViolation),
        "bytes::Bytes and bytes::BytesMut are owned raw byte containers even without type arguments"
    );
}

#[test]
fn permit_constructors_cannot_escape_the_single_mint_path() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "permit_qself_mint",
        "permit_constructor_derives",
        "permit_constructor_qualified_derives",
        "permit_constructor_trait_impls",
    ]);
    assert!(
        accepted.is_empty(),
        "accepted permit constructor bypasses: {accepted:?}"
    );
}

#[test]
fn exception_signatures_are_confined_to_the_execute_root_module() {
    assert_eq!(
        candidate_check_fixture(fixture("counterfeit_exception_types")),
        Err(CandidateRejection::BoundaryViolation),
        "same-named counterfeit types must not borrow the execute.rs API exceptions"
    );
}

#[test]
fn approved_batch_from_items_iterator_remains_allowed() {
    assert_eq!(
        candidate_check_fixture(fixture("approved_batch_from_items_iterator")),
        Ok(()),
        "the exact ClientBatch::from_items iterator input remains approved in execute.rs root"
    );
}

#[test]
fn approved_validation_source_factory_remains_allowed() {
    assert_eq!(
        candidate_check_fixture(fixture("approved_error_validation_source")),
        Ok(()),
        "the exact source-discarding validation factory remains allowed in error.rs root"
    );
}

#[test]
fn approved_batch_response_iterator_remains_allowed() {
    assert_eq!(
        candidate_check_fixture(fixture("approved_batch_response_iter_output")),
        Ok(()),
        "the exact opaque iterator output remains allowed in execute.rs root"
    );
}

#[test]
fn ufcs_transport_exchange_call_outside_execute_is_rejected() {
    assert_eq!(
        candidate_check_fixture(fixture("raw_exchange_ufcs_outside_execute")),
        Err(CandidateRejection::BoundaryViolation),
        "UFCS must not bypass the canonical Client::execute exchange boundary"
    );
}

#[test]
fn qself_transport_exchange_method_items_are_rejected() {
    assert_eq!(
        candidate_check_fixture(fixture("qself_exchange_method_item")),
        Err(CandidateRejection::BoundaryViolation),
        "a single-segment qself exchange method item must not evade the sole callsite audit"
    );
}

#[test]
fn exactly_one_permit_and_writer_call_are_inside_the_shared_exchange_operation() {
    assert_eq!(
        candidate_check_fixture(fixture("valid_execute")),
        Ok(()),
        "one permit mint and one writer call inside Client::exchange_operation is allowed"
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
fn canonical_macros_and_unique_batch_identifier_setter_remain_allowed() {
    for id in [
        "valid_execute",
        "canonical_vec_macro",
        "exact_unique_batch_id_setter",
        "approved_extension_view_callback",
        "approved_async_outcome_callbacks",
    ] {
        assert_eq!(
            candidate_check_fixture(fixture(id)),
            Ok(()),
            "approved positive-control fixture should be accepted: {id}"
        );
    }
}

#[test]
fn permit_self_literals_are_restricted_to_the_mint_constructor() {
    assert_eq!(
        candidate_check_fixture(fixture("permit_mint_and_helper_struct_literals")),
        Err(CandidateRejection::BoundaryViolation),
        "a Self literal in a permit helper must be rejected even when mint is valid"
    );
}

#[test]
fn complete_execute_audit_requires_exactly_one_canonical_exchange() {
    assert!(matches!(
        audit_source(fixture("canonical_vec_macro").source),
        Err(CandidateRejection::BoundaryViolation)
    ));
    let two_exchanges =
        format!("{CANONICAL_EXECUTE_EXCHANGE_FIXTURE}\n{CANONICAL_EXECUTE_EXCHANGE_FIXTURE}");
    assert!(matches!(
        audit_source(&two_exchanges),
        Err(CandidateRejection::BoundaryViolation)
    ));
}

#[test]
fn complete_execute_audit_rejects_repeated_or_recursive_exchange_control_flow() {
    let repeated =
        "impl Client { fn execute(&mut self) { loop { self.transport.exchange(&[], 1); } } }";
    assert!(
        matches!(
            audit_source(repeated),
            Err(CandidateRejection::BoundaryViolation)
        ),
        "a single exchange AST node inside a loop can still run more than once"
    );

    let recursive = "impl Client { fn execute(&mut self) { if true { self.execute(); } self.transport.exchange(&[], 1); } }";
    assert!(
        matches!(
            audit_source(recursive),
            Err(CandidateRejection::BoundaryViolation)
        ),
        "recursive execute can perform another exchange through one counted callsite"
    );

    let qualified = "impl Client { fn execute(&mut self) { if true { <Client>::execute(self); } self.transport.exchange(&[], 1); } }";
    assert!(
        matches!(
            audit_source(qualified),
            Err(CandidateRejection::BoundaryViolation)
        ),
        "qualified Client::execute calls must not bypass the recursion audit"
    );

    let self_qualified = "impl Client { fn execute(&mut self) { if true { <Self>::execute(self); } self.transport.exchange(&[], 1); } }";
    assert!(
        matches!(
            audit_source(self_qualified),
            Err(CandidateRejection::BoundaryViolation)
        ),
        "qualified Self::execute calls must not bypass the recursion audit"
    );
}

#[test]
fn complete_execute_audit_rejects_recursive_calls_inside_allowed_macros() {
    let macro_calls = [
        "impl Client { fn execute(&mut self) { let _ = vec![self.execute()]; self.transport.exchange(&[], 1); } }",
        "impl Client { fn execute(&mut self) { let _ = matches!(true, _ if self.execute()); self.transport.exchange(&[], 1); } }",
        "impl Client { fn execute(&mut self) { let _ = write!(std::io::sink(), \"{}\", self.execute()); self.transport.exchange(&[], 1); } }",
    ];

    for source in macro_calls {
        assert!(
            matches!(
                audit_source(source),
                Err(CandidateRejection::BoundaryViolation)
            ),
            "recursive execute calls inside supported macro expressions must be audited: {source}"
        );
    }
}

#[test]
fn aliases_macros_includes_generated_and_cfg_sources_cannot_bypass_the_audit() {
    let accepted = accepted_ids_for_rejected_fixtures(&[
        "writer_alias",
        "writer_reexport",
        "macro_token_tree",
        "qualified_custom_vec_macro",
        "glob_writer_import_with_direct_permit",
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
                "valid_execute"
                    | "canonical_vec_macro"
                    | "exact_unique_batch_id_setter"
                    | "approved_extension_view_callback"
                    | "approved_async_outcome_callbacks"
                    | "approved_batch_from_items_iterator"
                    | "approved_error_validation_source"
                    | "approved_batch_response_iter_output"
                    | "approved_registry_value_validation"
                    | "approved_registry_inspect"
                    | "approved_registry_generic_accessor"
                    | "matches_guard_logical_not"
            )),
            ExpectedDecision::Reject => assert!(!matches!(
                fixture.id,
                "valid_execute"
                    | "canonical_vec_macro"
                    | "exact_unique_batch_id_setter"
                    | "approved_extension_view_callback"
                    | "approved_async_outcome_callbacks"
                    | "approved_error_validation_source"
                    | "approved_batch_response_iter_output"
                    | "approved_registry_inspect"
                    | "approved_registry_generic_accessor"
                    | "matches_guard_logical_not"
            )),
        }
    }
}
