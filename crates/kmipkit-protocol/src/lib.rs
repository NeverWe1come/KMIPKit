//! KMIP 2.1 protocol models and validation for `KMIPKit`.
#![forbid(unsafe_code)]

mod attribute_policy {
    include!("generated/attribute_policy.rs");
}

mod activate;
mod add_attribute;
mod adjust_attribute;
mod archive;
mod asynchronous;
pub mod attribute;
mod attribute_reference;
mod attribute_types_generated;
mod cancel;
mod create;
mod create_key_pair;
mod create_split_key;
mod credential;
mod cryptographic_operation;
mod cryptographic_parameters;
mod decrypt;
mod delete_attribute;
mod destroy;
mod discover_versions;
mod encrypt;
mod error;
pub mod extension;
mod get_attribute_list;
mod get_attributes;
mod hash;
mod lifecycle;
mod mac;
mod mac_verify;
mod message;
mod modify_attribute;
mod multipart;
mod operation_data;
mod ping;
mod poll;
mod process;
mod query;
mod query_async_requests;
mod recover;
mod result;
mod set_attribute;
mod sign;
mod signature_verify;

pub use activate::{ActivateError, ActivateRequest, ActivateResponse};
pub use add_attribute::{AddAttributeError, AddAttributeRequest, AddAttributeResponse};
pub use adjust_attribute::{
    AdjustAttributeError, AdjustAttributeRequest, AdjustAttributeResponse, AdjustmentType,
};
pub use archive::{ArchiveError, ArchiveRequest, ArchiveResponse};
pub use asynchronous::AsynchronousOperationError;
pub use attribute::{AttributeSet, AttributeSetError, CurrentAttribute, NewAttribute};
pub use attribute_reference::AttributeReference;
pub use cancel::{CancelRequest, CancelResponse, CancellationResult};
pub use create::{CreateError, CreateRequest, CreateResponse, ObjectType, UniqueIdentifier};
pub use create_key_pair::{CreateKeyPairError, CreateKeyPairRequest, CreateKeyPairResponse};
pub use create_split_key::{
    CreateSplitKeyError, CreateSplitKeyRequest, CreateSplitKeyResponse, SplitKeyMethod,
};
pub use credential::{
    AttestationCredential, Authentication, Credential, CredentialType, CredentialValidationError,
    CredentialValidationErrorKind, CredentialValue, CredentialView, DeviceCredential,
    HashedPasswordCredential, Nonce, OneTimePasswordCredential, OpaqueTtlv, SecretBytes,
    SecretText, TicketCredential, UsernameAndPasswordCredential,
};
pub use cryptographic_operation::{
    CryptographicAlgorithm, CryptographicOperationError, CryptographicOperationErrorKind,
    DigitalSignatureAlgorithm, HashingAlgorithm, ValidityIndicator, VerificationResponseContext,
};
pub use decrypt::{DecryptError, DecryptRequest, DecryptResponse};
pub use delete_attribute::{DeleteAttributeError, DeleteAttributeRequest, DeleteAttributeResponse};
pub use destroy::{DestroyError, DestroyRequest, DestroyResponse};
pub use discover_versions::{
    DiscoverVersionsError, DiscoverVersionsRequest, DiscoverVersionsResponse,
};
pub use encrypt::{EncryptError, EncryptRequest, EncryptResponse};
pub use error::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
pub use get_attribute_list::{
    GetAttributeListError, GetAttributeListRequest, GetAttributeListResponse,
};
pub use get_attributes::{GetAttributesError, GetAttributesRequest, GetAttributesResponse};
pub use hash::{HashError, HashRequest, HashResponse};
pub use mac::{MacError, MacRequest, MacResponse};
pub use mac_verify::{MacVerifyError, MacVerifyRequest, MacVerifyResponse};
pub use message::{
    MessageExtensionView, MessageValidationError, MessageValidationErrorKind, ProtocolVersion,
    RequestBatchItemView, RequestHeaderView, RequestMessage, ResponseBatchItemView,
    ResponseHeaderView, ResponseMessage,
};
pub use modify_attribute::{ModifyAttributeError, ModifyAttributeRequest, ModifyAttributeResponse};
pub use operation_data::OperationData;
pub use ping::{PingError, PingRequest, PingResponse};
pub use poll::{PollRequest, PollResponse};
pub use process::{ProcessRequest, ProcessResponse};
pub use query::{QueryError, QueryFunction, QueryRequest, QueryResponse, QueryResponseField};
pub use query_async_requests::{QueryAsyncRequestsRequest, QueryAsyncRequestsResponse};
pub use recover::{RecoverError, RecoverRequest, RecoverResponse};
pub use result::{
    KmipOperationResult, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};
pub use set_attribute::{SetAttributeError, SetAttributeRequest, SetAttributeResponse};
pub use sign::{SignError, SignRequest, SignResponse};
pub use signature_verify::{SignatureVerifyError, SignatureVerifyRequest, SignatureVerifyResponse};

#[cfg(test)]
#[path = "../tests/unit/discover_versions_tests.rs"]
mod discover_versions_tests;

#[cfg(test)]
#[path = "../tests/unit/create_tests.rs"]
mod create_tests;

#[cfg(test)]
#[path = "../tests/unit/activate_operation_tests.rs"]
mod activate_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/archive_operation_tests.rs"]
mod archive_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/destroy_operation_tests.rs"]
mod destroy_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/recover_operation_tests.rs"]
mod recover_operation_tests;

#[cfg(test)]
#[path = "../tests/support/operation_test_support.rs"]
mod operation_test_support;

#[cfg(test)]
#[path = "../tests/support/cryptographic_operation_test_support.rs"]
mod cryptographic_operation_test_support;

#[cfg(test)]
#[path = "../tests/unit/operation_data_tests.rs"]
mod operation_data_tests;

#[cfg(test)]
#[path = "../tests/unit/cryptographic_parameters_tests.rs"]
mod cryptographic_parameters_tests;

#[cfg(test)]
#[path = "../tests/unit/cryptographic_operation_contract_tests.rs"]
mod cryptographic_operation_contract_tests;

#[cfg(test)]
#[path = "../tests/unit/hash_operation_tests.rs"]
mod hash_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/mac_operation_tests.rs"]
mod mac_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/sign_operation_tests.rs"]
mod sign_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/mac_verify_operation_tests.rs"]
mod mac_verify_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/signature_verify_operation_tests.rs"]
mod signature_verify_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_tests.rs"]
mod encrypt_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_parameter_structure_tests.rs"]
mod encrypt_parameter_structure_tests;

#[cfg(test)]
#[path = "../tests/unit/decrypt_tests.rs"]
mod decrypt_tests;

#[cfg(test)]
#[path = "../tests/unit/multipart_validation_tests.rs"]
mod multipart_validation_tests;

#[cfg(test)]
#[path = "../tests/unit/operation_failure_tests.rs"]
mod operation_failure_tests;

#[cfg(test)]
#[path = "../tests/unit/common_operation_result_tests.rs"]
mod common_operation_result_tests;

#[cfg(test)]
#[path = "../tests/unit/pending_encrypt_response_shape_tests.rs"]
mod pending_encrypt_response_shape_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_response_tests.rs"]
mod encrypt_response_tests;

#[cfg(test)]
#[path = "../tests/unit/decrypt_response_tests.rs"]
mod decrypt_response_tests;

#[cfg(test)]
#[path = "../tests/unit/encrypt_decrypt_redaction_tests.rs"]
mod encrypt_decrypt_redaction_tests;

#[cfg(test)]
#[path = "../tests/unit/malformed_crypto_payload_tests.rs"]
mod malformed_crypto_payload_tests;

#[cfg(test)]
#[path = "../tests/unit/crypto_robustness_property_tests.rs"]
mod crypto_robustness_property_tests;

#[cfg(test)]
#[path = "../tests/unit/create_key_pair_tests.rs"]
mod create_key_pair_tests;

#[cfg(test)]
#[path = "../tests/unit/create_split_key_tests.rs"]
mod create_split_key_tests;

#[cfg(test)]
#[path = "../tests/support/async_operation_fixtures.rs"]
mod async_operation_fixtures;

#[cfg(test)]
#[path = "../tests/support/query_ping_fixtures.rs"]
mod query_ping_fixtures;

#[cfg(test)]
#[path = "../tests/unit/ping_operation_tests.rs"]
mod ping_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/query_operation_tests.rs"]
mod query_operation_tests;

#[cfg(test)]
#[path = "../tests/support/lifecycle_fixtures.rs"]
mod lifecycle_fixtures;

#[cfg(test)]
#[path = "../tests/unit/lifecycle_fixtures_tests.rs"]
mod lifecycle_fixtures_tests;

#[cfg(test)]
#[path = "../tests/unit/asynchronous_tests.rs"]
mod asynchronous_tests;

#[cfg(test)]
#[path = "../tests/unit/poll_tests.rs"]
mod poll_tests;

#[cfg(test)]
#[path = "../tests/unit/cancel_tests.rs"]
mod cancel_tests;

#[cfg(test)]
#[path = "../tests/unit/process_tests.rs"]
mod process_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_requests_tests.rs"]
mod query_async_requests_tests;

#[cfg(test)]
#[path = "../tests/unit/query_async_response_tests.rs"]
mod query_async_response_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_tests.rs"]
mod attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/vendor_attribute_tests.rs"]
mod vendor_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/get_attributes_tests.rs"]
mod get_attributes_tests;

#[cfg(test)]
#[path = "../tests/unit/get_attribute_list_tests.rs"]
mod get_attribute_list_tests;

#[cfg(test)]
#[path = "../tests/unit/add_attribute_tests.rs"]
mod add_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/adjust_attribute_tests.rs"]
mod adjust_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/delete_attribute_tests.rs"]
mod delete_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/modify_attribute_tests.rs"]
mod modify_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/set_attribute_tests.rs"]
mod set_attribute_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_roundtrip_tests.rs"]
mod attribute_roundtrip_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_limits_tests.rs"]
mod attribute_limits_tests;

#[cfg(test)]
#[path = "../tests/unit/attribute_coverage_contract_tests.rs"]
mod attribute_coverage_contract_tests;
