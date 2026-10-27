//! KMIP 2.1 protocol models and validation for `KMIPKit`.
#![forbid(unsafe_code)]

#[cfg(test)]
#[path = "../tests/unit/interop_operation_tests.rs"]
mod interop_operation_tests;

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
mod certify;
mod check;
mod create;
mod create_key_pair;
mod create_split_key;
mod credential;
mod cryptographic_operation;
mod cryptographic_parameters;
mod decrypt;
mod delegated_login;
mod delete_attribute;
mod derive_key;
mod destroy;
mod diagnostics;
mod discover_versions;
mod encrypt;
mod error;
mod export;
pub mod extension;
mod get;
mod get_attribute_list;
mod get_attributes;
mod get_constraints;
mod get_usage_allocation;
mod hash;
mod import;
mod interop;
mod join_split_key;
mod lifecycle;
mod locate;
mod log;
mod login;
mod logout;
mod mac;
mod mac_verify;
mod message;
mod modify_attribute;
mod multipart;
mod obtain_lease;
mod operation_data;
mod ping;
mod pkcs11;
mod poll;
mod process;
mod profile;
/// Generated source and evidence metadata for catalogued client profiles.
pub mod profiles_generated;
mod query;
mod query_async_requests;
mod recertify;
mod recover;
mod register;
mod rekey;
mod rekey_key_pair;
mod reprovision;
mod result;
mod revoke;
mod rng_retrieve;
mod rng_seed;
mod set_attribute;
mod set_constraints;
#[cfg(test)]
#[path = "../tests/unit/set_constraints_tests.rs"]
mod set_constraints_tests;
mod set_defaults;
#[cfg(test)]
#[path = "../tests/unit/set_defaults_tests.rs"]
mod set_defaults_tests;
mod sign;
mod signature_verify;
mod ticket;
mod validate;

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
pub use certify::{CertifyError, CertifyRequest, CertifyResponse};
pub use check::{CheckError, CheckRequest, CheckResponse};
pub use create::{CreateError, CreateRequest, CreateResponse, ObjectType, UniqueIdentifier};
pub use create_key_pair::{CreateKeyPairError, CreateKeyPairRequest, CreateKeyPairResponse};
pub use create_split_key::{
    CreateSplitKeyError, CreateSplitKeyRequest, CreateSplitKeyResponse, SplitKeyMethod,
};
pub use credential::{
    AttestationCredential, Authentication, Credential, CredentialType, CredentialValidationError,
    CredentialValidationErrorKind, CredentialValue, CredentialView, DeviceCredential,
    HashedPasswordCredential, HashedPasswordTimestampSequence,
    HashedPasswordTimestampSequenceError, Nonce, OneTimePasswordCredential, OpaqueTtlv,
    SecretBytes, SecretText, TicketCredential, UsernameAndPasswordCredential,
};
pub use cryptographic_operation::{
    CryptographicAlgorithm, CryptographicOperationError, CryptographicOperationErrorKind,
    CryptographicOperationResponseContext, DigitalSignatureAlgorithm, HashingAlgorithm,
    ValidityIndicator, VerificationResponseContext,
};
pub use decrypt::{DecryptError, DecryptRequest, DecryptResponse};
pub use delegated_login::{DelegatedLoginError, DelegatedLoginRequest, DelegatedLoginResponse};
pub use delete_attribute::{DeleteAttributeError, DeleteAttributeRequest, DeleteAttributeResponse};
pub use derive_key::{
    DerivationMethod, DerivationParameters, DeriveKeyError, DeriveKeyRequest, DeriveKeyResponse,
};
pub use destroy::{DestroyError, DestroyRequest, DestroyResponse};
pub use discover_versions::{
    DiscoverVersionsError, DiscoverVersionsRequest, DiscoverVersionsResponse,
};
pub use encrypt::{EncryptError, EncryptRequest, EncryptResponse};
pub use error::{ProtocolCauseCategory, ProtocolError, ProtocolErrorKind};
pub use export::{ExportError, ExportRequest, ExportResponse};
pub use get::{GetError, GetRequest, GetResponse, KeyCompressionType, KeyFormatType, KeyWrapType};
pub use get_attribute_list::{
    GetAttributeListError, GetAttributeListRequest, GetAttributeListResponse,
};
pub use get_attributes::{GetAttributesError, GetAttributesRequest, GetAttributesResponse};
pub use get_constraints::{GetConstraintsError, GetConstraintsRequest, GetConstraintsResponse};
pub use get_usage_allocation::{
    GetUsageAllocationError, GetUsageAllocationRequest, GetUsageAllocationResponse,
};
pub use hash::{HashError, HashRequest, HashResponse};
pub use import::{ImportError, ImportRequest, ImportResponse};
pub use interop::{InteropError, InteropRequest, InteropResponse};
pub use join_split_key::{
    JoinSplitKeyError, JoinSplitKeyRequest, JoinSplitKeyResponse, SplitKeyReference,
};
pub use locate::{
    LocateError, LocateRequest, LocateResponse, ObjectGroupMember, StorageStatusMask,
};
pub use log::{LogError, LogRequest, LogResponse};
pub use login::{LoginError, LoginRequest, LoginResponse};
pub use logout::{LogoutError, LogoutRequest, LogoutResponse};
pub use mac::{MacError, MacRequest, MacResponse};
pub use mac_verify::{MacVerifyError, MacVerifyRequest, MacVerifyResponse};
pub use message::{
    MessageExtensionView, MessageValidationError, MessageValidationErrorKind, ProtocolVersion,
    RequestBatchItemView, RequestHeaderView, RequestMessage, ResponseBatchItemView,
    ResponseHeaderView, ResponseMessage,
};
pub use modify_attribute::{ModifyAttributeError, ModifyAttributeRequest, ModifyAttributeResponse};
pub use obtain_lease::{ObtainLeaseError, ObtainLeaseRequest, ObtainLeaseResponse};
pub use operation_data::OperationData;
pub use ping::{PingError, PingRequest, PingResponse};
pub use pkcs11::{Pkcs11Error, Pkcs11Request, Pkcs11Response};
pub use poll::{PollRequest, PollResponse};
pub use process::{ProcessRequest, ProcessResponse};
pub use profile::{
    ProfileEvidenceReadiness, ProfileEvidenceRecord, ProfileReadiness, ProfileReadinessBlocker,
    ProfileTransport, ProfileValidationError, ProfileValidationErrorCode, ProfileValidationResult,
    ProfileValidationSelection,
};
pub use query::{QueryError, QueryFunction, QueryRequest, QueryResponse, QueryResponseField};
pub use query_async_requests::{QueryAsyncRequestsRequest, QueryAsyncRequestsResponse};
pub use recertify::{RecertifyError, RecertifyRequest, RecertifyResponse};
pub use recover::{RecoverError, RecoverRequest, RecoverResponse};
pub use register::{OpaqueObjectContext, RegisterError, RegisterRequest, RegisterResponse};
pub use rekey::{RekeyError, RekeyRequest, RekeyResponse};
pub use rekey_key_pair::{RekeyKeyPairError, RekeyKeyPairRequest, RekeyKeyPairResponse};
pub use reprovision::{ReProvisionError, ReProvisionRequest, ReProvisionResponse};
pub use result::{
    KmipOperationResult, ResultMessage, ResultReason, ResultStatus, ResultValidationError,
};
pub use revoke::{RevocationReason, RevokeError, RevokeRequest, RevokeResponse};
pub use rng_retrieve::{RngRetrieveError, RngRetrieveRequest, RngRetrieveResponse};
pub use rng_seed::{RngSeedError, RngSeedRequest, RngSeedResponse};
pub use set_attribute::{SetAttributeError, SetAttributeRequest, SetAttributeResponse};
pub use set_constraints::{SetConstraintsError, SetConstraintsRequest, SetConstraintsResponse};
pub use set_defaults::{SetDefaultsError, SetDefaultsRequest, SetDefaultsResponse};
pub use sign::{SignError, SignRequest, SignResponse};
pub use signature_verify::{SignatureVerifyError, SignatureVerifyRequest, SignatureVerifyResponse};
pub use ticket::{Ticket, TicketType, TicketValidationError, TicketValidationErrorKind};
pub use validate::{ValidateError, ValidateRequest, ValidateResponse};

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
#[path = "../tests/unit/certify_operation_tests.rs"]
mod certify_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/check_operation_tests.rs"]
mod check_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/obtain_lease_operation_tests.rs"]
mod obtain_lease_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/recertify_operation_tests.rs"]
mod recertify_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/rekey_operation_tests.rs"]
mod rekey_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/rekey_key_pair_operation_tests.rs"]
mod rekey_key_pair_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/reprovision_operation_tests.rs"]
mod reprovision_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/register_operation_tests.rs"]
mod register_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/revoke_operation_tests.rs"]
mod revoke_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/lifecycle_error_contract_tests.rs"]
mod lifecycle_error_contract_tests;

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
// Temporarily disabled while operation-specific Red/Green slices are exercised.
#[cfg(test)]
#[path = "../tests/unit/hash_operation_tests.rs"]
mod hash_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/rng_retrieve_operation_tests.rs"]
mod rng_retrieve_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/rng_seed_operation_tests.rs"]
mod rng_seed_operation_tests;

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
#[path = "../tests/unit/join_split_key_operation_tests.rs"]
mod join_split_key_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/derive_key_operation_tests.rs"]
mod derive_key_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/export_operation_tests.rs"]
mod export_operation_tests;
#[cfg(test)]
#[path = "../tests/unit/import_operation_tests.rs"]
mod import_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/key_transfer_operation_contract_tests.rs"]
mod key_transfer_operation_contract_tests;

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
#[path = "../tests/support/managed_object_retrieval_fixtures.rs"]
mod managed_object_retrieval_fixtures;

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
#[path = "../tests/unit/get_tests.rs"]
mod get_tests;

#[cfg(test)]
#[path = "../tests/unit/locate_tests.rs"]
mod locate_tests;

#[cfg(test)]
#[path = "../tests/unit/get_attribute_list_tests.rs"]
mod get_attribute_list_tests;

#[cfg(test)]
#[path = "../tests/unit/get_constraints_tests.rs"]
mod get_constraints_tests;

#[cfg(test)]
#[path = "../tests/unit/get_usage_allocation_tests.rs"]
mod get_usage_allocation_tests;

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

#[cfg(test)]
#[path = "../tests/unit/ticket_model_tests.rs"]
mod ticket_model_tests;

#[cfg(test)]
#[path = "../tests/unit/delegated_login_operation_tests.rs"]
mod delegated_login_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/login_operation_tests.rs"]
mod login_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/logout_operation_tests.rs"]
mod logout_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/log_operation_tests.rs"]
mod log_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/validate_operation_tests.rs"]
mod validate_operation_tests;

#[cfg(test)]
#[path = "../tests/unit/pkcs11_operation_tests.rs"]
mod pkcs11_operation_tests;
