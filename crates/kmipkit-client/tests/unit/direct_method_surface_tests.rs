//! KMIPKIT-0050 compile-time checks for the 56 direct Client operations.
//!
//! Operation names and direction follow OASIS KMIP v2.1 §6.1 and the reviewed
//! 56-row client operation inventory in `specs/050-parameters-auth-client`.

use kmipkit_protocol::*;
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;

use crate::{Client, ClientBatchItemResponse, ClientError, ClientOperationOutcome};

type ClientOutcome = Result<ClientOperationOutcome, ClientError>;
type AsyncControlOptionsMethod<Request> =
    fn(&mut Client, Request, Option<u32>, &CodecLimits, &RequestOptions) -> ClientOutcome;

macro_rules! assert_item_pair {
    ($method:ident, $with_options:ident, $request:ty) => {{
        let _: fn(
            &mut Client,
            $request,
            &CodecLimits,
        ) -> Result<ClientBatchItemResponse, ClientError> = Client::$method;
        let _: fn(
            &mut Client,
            $request,
            &CodecLimits,
            &RequestOptions,
        ) -> Result<ClientBatchItemResponse, ClientError> = Client::$with_options;
    }};
}

#[test]
#[allow(clippy::too_many_lines)] // One explicit inventory keeps all 56 operation symbols reviewable together.
fn all_client_initiated_operations_have_addressable_direct_methods() {
    let _ = Client::activate;
    let _ = Client::add_attribute;
    let _ = Client::adjust_attribute;
    let _ = Client::archive;
    let _ = Client::execute_cancel;
    let _ = Client::certify;
    let _ = Client::check;
    let _ = Client::create;
    let _ = Client::create_key_pair;
    let _ = Client::create_split_key;
    let _ = Client::decrypt;
    let _ = Client::delegated_login;
    let _ = Client::delete_attribute;
    let _ = Client::derive_key;
    let _ = Client::destroy;
    let _ = Client::discover_versions;
    let _ = Client::encrypt;
    let _ = Client::export;
    let _ = Client::get;
    let _ = Client::get_attribute_list;
    let _ = Client::get_attributes;
    let _ = Client::get_constraints;
    let _ = Client::get_usage_allocation;
    let _ = Client::hash;
    let _ = Client::import;
    let _ = Client::interop;
    let _ = Client::join_split_key;
    let _ = Client::locate;
    let _ = Client::log;
    let _ = Client::login;
    let _ = Client::logout;
    let _ = Client::mac;
    let _ = Client::mac_verify;
    let _ = Client::modify_attribute;
    let _ = Client::obtain_lease;
    let _ = Client::pkcs11;
    let _ = Client::ping;
    let _ = Client::execute_poll;
    let _ = Client::execute_process;
    let _ = Client::query;
    let _ = Client::execute_query_async_requests;
    let _ = Client::recertify;
    let _ = Client::rekey;
    let _ = Client::rekey_key_pair;
    let _ = Client::reprovision;
    let _ = Client::recover;
    let _ = Client::register;
    let _ = Client::revoke;
    let _ = Client::rng_retrieve;
    let _ = Client::rng_seed;
    let _ = Client::set_attribute;
    let _ = Client::set_constraints;
    let _ = Client::set_defaults;
    let _ = Client::sign;
    let _ = Client::signature_verify;
    let _ = Client::validate;

    let _: fn(
        &mut Client,
        PollRequest,
        &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_poll;
    let _: fn(
        &mut Client,
        CancelRequest,
        &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_cancel;
    let _: fn(
        &mut Client,
        ProcessRequest,
        Option<u32>,
        &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_process;
    let _: fn(
        &mut Client,
        QueryAsyncRequestsRequest,
        Option<u32>,
        &CodecLimits,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_query_async_requests;

    let _: fn(
        &mut Client,
        PollRequest,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_poll_with_options;
    let _: fn(
        &mut Client,
        CancelRequest,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<ClientOperationOutcome, ClientError> = Client::execute_cancel_with_options;
    let _: AsyncControlOptionsMethod<ProcessRequest> = Client::execute_process_with_options;
    let _: AsyncControlOptionsMethod<QueryAsyncRequestsRequest> =
        Client::execute_query_async_requests_with_options;
}

#[test]
fn missing_direct_operations_expose_default_and_options_forms() {
    assert_item_pair!(
        discover_versions,
        discover_versions_with_options,
        DiscoverVersionsRequest
    );
    assert_item_pair!(
        delegated_login,
        delegated_login_with_options,
        DelegatedLoginRequest
    );
    assert_item_pair!(login, login_with_options, LoginRequest);
    assert_item_pair!(logout, logout_with_options, LogoutRequest);
    assert_item_pair!(get, get_with_options, GetRequest);
    assert_item_pair!(locate, locate_with_options, LocateRequest);
    assert_item_pair!(
        add_attribute,
        add_attribute_with_options,
        AddAttributeRequest
    );
    assert_item_pair!(
        adjust_attribute,
        adjust_attribute_with_options,
        AdjustAttributeRequest
    );
    assert_item_pair!(
        delete_attribute,
        delete_attribute_with_options,
        DeleteAttributeRequest
    );
    assert_item_pair!(
        modify_attribute,
        modify_attribute_with_options,
        ModifyAttributeRequest
    );
    assert_item_pair!(
        set_attribute,
        set_attribute_with_options,
        SetAttributeRequest
    );
    assert_item_pair!(
        get_attributes,
        get_attributes_with_options,
        GetAttributesRequest
    );
    assert_item_pair!(encrypt, encrypt_with_options, EncryptRequest);
    assert_item_pair!(decrypt, decrypt_with_options, DecryptRequest);
}

#[test]
fn existing_direct_operations_expose_the_missing_options_forms() {
    let _: fn(&mut Client, &CodecLimits) -> Result<ClientBatchItemResponse, ClientError> =
        Client::get_constraints;
    let _: fn(
        &mut Client,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<ClientBatchItemResponse, ClientError> = Client::get_constraints_with_options;
    assert_item_pair!(
        get_attribute_list,
        get_attribute_list_with_options,
        GetAttributeListRequest
    );
    assert_item_pair!(
        set_constraints,
        set_constraints_with_options,
        SetConstraintsRequest
    );
    assert_item_pair!(set_defaults, set_defaults_with_options, SetDefaultsRequest);
    assert_item_pair!(
        get_usage_allocation,
        get_usage_allocation_with_options,
        GetUsageAllocationRequest
    );
    assert_item_pair!(validate, validate_with_options, ValidateRequest);
    assert_item_pair!(rng_retrieve, rng_retrieve_with_options, RngRetrieveRequest);
    assert_item_pair!(log, log_with_options, LogRequest);
    assert_item_pair!(interop, interop_with_options, InteropRequest);
    assert_item_pair!(rng_seed, rng_seed_with_options, RngSeedRequest);
}
