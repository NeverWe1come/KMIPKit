//! Synchronous typed KMIP client execution foundation for `KMIPKit`.
//!
//! The typed request enum supports explicit Discover Versions, Activate,
//! Archive, Certify, Destroy, Recover, Create, Create Key Pair, Create Split Key, Add
//! Attribute, Adjust Attribute, Delete Attribute, Get Attributes, Get
//! Attribute List, Modify Attribute, Set Attribute, Query, and Ping operations.
//! Separate methods execute Poll, Cancel, Process, and Query Asynchronous
//! Requests. The synchronous [`Client`] is constructed from an immutable client
//! configuration and validated production transport configuration; callers
//! cannot inject an arbitrary transport or submit raw KMIP bytes. Timeout
//! overrides are available through the options-bearing typed methods.
//!
//! [`ClientBatchOutcome::response`] returns a [`ClientResponseView`] with a
//! common operation-result accessor and typed accessors for each response.
//!
//! The following example prepares a typed batch without sending a request:
//!
//! ```
//! use kmipkit_client::{ClientBatch, ClientBatchItem, ClientRequest};
//!
//! let request = ClientRequest::discover_versions();
//! let batch = ClientBatch::new(ClientBatchItem::new(request));
//! assert_eq!(batch.items().len(), 1);
//! ```
//!
//! See the repository guide at `docs/user-guide/en/client-execution.md` for
//! resource limits, redaction, the low-level transport contract, and current
//! implementation boundaries.
#![doc = include_str!("../../../docs/user-guide/en/production-transports.md")]
#![doc = include_str!("../../../docs/user-guide/es/transportes-produccion.md")]
#![forbid(unsafe_code)]

mod error;
pub mod extension_registry;

#[cfg(test)]
#[path = "../tests/fixtures/extensions/extension_fixtures.generated.rs"]
pub(crate) mod extension_fixtures;

#[cfg(test)]
#[path = "../tests/support/fake_transport.rs"]
pub(crate) mod fake_transport_test_support;

#[cfg(test)]
#[path = "../tests/unit/extension_registry_test_support.rs"]
mod extension_registry_test_support;

#[cfg(test)]
#[path = "../tests/unit/extension_execution.rs"]
mod extension_execution_tests;

mod execute;

#[cfg(test)]
#[path = "../tests/unit/profile_fixture_replay_tests.rs"]
mod profile_fixture_replay_tests;

/// Typed opaque PKCS#11 forwarding models.
pub use kmipkit_protocol::{Pkcs11Error, Pkcs11Request, Pkcs11Response};
mod usage_allocation;

pub use usage_allocation::{UsageAllocationContext, UsageAllocationContextError};

#[cfg(test)]
#[path = "../tests/unit/execution_tests.rs"]
mod execution_tests;

#[cfg(test)]
#[path = "../tests/unit/option_tests.rs"]
mod option_tests;

#[cfg(test)]
#[path = "../tests/unit/response_boundary_tests.rs"]
mod response_boundary_tests;

#[cfg(test)]
#[path = "../tests/unit/request_time_stamp_tests.rs"]
mod request_time_stamp_tests;

#[cfg(test)]
#[path = "../tests/unit/codec_limits_identity_tests.rs"]
mod codec_limits_identity_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_lifecycle_tests.rs"]
mod execute_lifecycle_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_test_support.rs"]
mod execute_test_support;

#[cfg(test)]
#[path = "../tests/unit/asynchronous_execution_test_support.rs"]
mod asynchronous_execution_test_support;

#[cfg(test)]
#[path = "../tests/unit/poll_execution_tests.rs"]
mod poll_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/cancel_execution_tests.rs"]
mod cancel_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/create_execution_tests.rs"]
mod create_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/create_key_pair_execution_tests.rs"]
mod create_key_pair_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/create_split_key_execution_tests.rs"]
mod create_split_key_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/join_split_key_execution_tests.rs"]
mod join_split_key_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/activate_execution_tests.rs"]
mod activate_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/archive_execution_tests.rs"]
mod archive_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/certify_execution_tests.rs"]
mod certify_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/check_execution_tests.rs"]
mod check_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/obtain_lease_execution_tests.rs"]
mod obtain_lease_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/destroy_execution_tests.rs"]
mod destroy_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/recover_execution_tests.rs"]
mod recover_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_multipart_execution_tests.rs"]
mod encrypt_decrypt_multipart_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_id_placeholder_execution_tests.rs"]
mod encrypt_decrypt_id_placeholder_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_fixture_execution_tests.rs"]
mod encrypt_decrypt_fixture_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_outcome_execution_tests.rs"]
mod encrypt_decrypt_outcome_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_redaction_execution_tests.rs"]
mod encrypt_decrypt_redaction_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_data_roundtrip_property_tests.rs"]
mod encrypt_decrypt_data_roundtrip_property_tests;

#[cfg(test)]
#[path = "../tests/unit/lifecycle_redaction_tests.rs"]
mod lifecycle_redaction_tests;

#[cfg(test)]
#[path = "../tests/unit/lifecycle_execution_tests.rs"]
mod lifecycle_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/process_execution_tests.rs"]
mod process_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/ping_execution_tests.rs"]
mod ping_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/rng_retrieve_execution_tests.rs"]
mod rng_retrieve_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/rng_seed_execution_tests.rs"]
mod rng_seed_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/query_execution_tests.rs"]
mod query_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/query_ping_redaction_tests.rs"]
mod query_ping_redaction_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_execution_tests.rs"]
mod query_async_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/execute_boundary_tests.rs"]
mod execute_boundary_tests;

#[cfg(test)]
#[path = "../tests/unit/attestation_indicator_tests.rs"]
mod attestation_indicator_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_read_execution_tests.rs"]
mod attribute_read_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/get_constraints_execution_tests.rs"]
mod get_constraints_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/get_usage_allocation_execution_tests.rs"]
mod get_usage_allocation_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/set_constraints_execution_tests.rs"]
mod set_constraints_execution_tests;
#[cfg(test)]
#[path = "../tests/unit/set_defaults_execution_tests.rs"]
mod set_defaults_execution_tests;
#[cfg(test)]
#[path = "../tests/unit/usage_allocation_guard_tests.rs"]
mod usage_allocation_guard_tests;

#[cfg(test)]
#[path = "../tests/unit/policy_controls_guide_examples.rs"]
mod policy_controls_guide_examples;

#[cfg(test)]
#[path = "../tests/unit/attribute_mutation_execution_tests.rs"]
mod attribute_mutation_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_execution_contract_tests.rs"]
mod attribute_execution_contract_tests;

#[cfg(test)]
#[path = "../tests/unit/recertify_execution_tests.rs"]
mod recertify_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/rekey_execution_tests.rs"]
mod rekey_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/rekey_key_pair_execution_tests.rs"]
mod rekey_key_pair_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/reprovision_execution_tests.rs"]
mod reprovision_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/register_execution_tests.rs"]
mod register_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/revoke_execution_tests.rs"]
mod revoke_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/object_read_execution_tests.rs"]
mod object_read_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/transport_contract_tests.rs"]
mod transport_contract_tests;

#[cfg(test)]
#[path = "../tests/unit/profile_preflight_tests.rs"]
mod profile_preflight_tests;

pub use error::{ClientCauseCategory, ClientError, ClientErrorCategory, ClientErrorResponseTtlv};
pub use execute::{
    Client, ClientBatch, ClientBatchItem, ClientBatchItemResponse, ClientBatchOutcome,
    ClientBatchResponse, ClientMessageExtension, ClientOperation, ClientOperationOutcome,
    ClientRequest, ClientResponseView, PendingOutcome,
};
pub use kmipkit_protocol::{
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode,
    ProfileValidationSelection,
};
pub use kmipkit_transport::RequestOptions;

#[cfg(test)]
#[path = "../tests/unit/interop_execution_tests.rs"]
mod interop_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/log_execution_tests.rs"]
mod log_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/validate_execution_tests.rs"]
mod validate_execution_tests;

#[cfg(test)]
#[path = "../tests/unit/pkcs11_execution_tests.rs"]
mod pkcs11_execution_tests;
