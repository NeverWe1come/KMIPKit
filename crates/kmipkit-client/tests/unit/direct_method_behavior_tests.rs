//! Fake-transport parity for the additive KMIPKIT-0050 direct methods.
//!
//! Operation direction and names follow OASIS KMIP v2.1 §6.1. These checks
//! compare the encoded one-item Request Message with the corresponding
//! `ClientBatch` call and verify that per-exchange options reach the transport.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;
use std::time::Duration;

use kmipkit_protocol::*;
use kmipkit_transport::{
    RequestDeliveryState, RequestOptions, TimeoutLimit, Transport, TransportCauseCategory,
    TransportError, TransportResponse,
};
use kmipkit_ttlv::codec::CodecLimits;
use kmipkit_ttlv::{Structure, Value};

use crate::execute_test_support::test_item;
use crate::{Client, ClientBatch, ClientBatchItem, ClientError, ClientRequest};

#[derive(Clone)]
struct NotSentTransport(Rc<RefCell<Vec<Vec<u8>>>>);

impl Transport for NotSentTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        _max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        self.0.borrow_mut().push(request.to_vec());
        Err(TransportError::new(
            RequestDeliveryState::NotSent,
            TransportCauseCategory::Io,
            io::Error::other("intentional direct-method parity failure"),
        ))
    }
}

fn client(
    requests: Rc<RefCell<Vec<Vec<u8>>>>,
    options: Rc<RefCell<Vec<RequestOptions>>>,
) -> Client {
    Client::for_test_with_options_observer(NotSentTransport(requests), options)
}

fn assert_direct_matches_batch<Request>(
    method: &str,
    make_request: impl Fn() -> Request,
    to_client_request: impl Fn(Request) -> ClientRequest,
    direct_call: impl FnOnce(
        &mut Client,
        Request,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<crate::ClientBatchItemResponse, ClientError>,
    default_call: impl FnOnce(
        &mut Client,
        Request,
        &CodecLimits,
    ) -> Result<crate::ClientBatchItemResponse, ClientError>,
) {
    let limits = CodecLimits::defaults();
    let options =
        RequestOptions::default().with_connect(TimeoutLimit::Bounded(Duration::from_secs(7)));
    let direct_bytes = Rc::new(RefCell::new(Vec::new()));
    let direct_options = Rc::new(RefCell::new(Vec::new()));
    let direct_error = direct_call(
        &mut client(Rc::clone(&direct_bytes), Rc::clone(&direct_options)),
        make_request(),
        &limits,
        &options,
    )
    .expect_err("the fake transport deliberately reports NotSent");

    let batch_bytes = Rc::new(RefCell::new(Vec::new()));
    let batch_options = Rc::new(RefCell::new(Vec::new()));
    let batch_error = client(Rc::clone(&batch_bytes), Rc::clone(&batch_options))
        .execute_with_options(
            ClientBatch::new(ClientBatchItem::new(to_client_request(make_request()))),
            &limits,
            &options,
        )
        .expect_err("the fake transport deliberately reports NotSent");

    let default_bytes = Rc::new(RefCell::new(Vec::new()));
    let default_options = Rc::new(RefCell::new(Vec::new()));
    let default_error = default_call(
        &mut client(Rc::clone(&default_bytes), Rc::clone(&default_options)),
        make_request(),
        &limits,
    )
    .expect_err("the fake transport deliberately reports NotSent");

    assert_eq!(
        direct_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        batch_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        default_error.delivery_state(),
        Some(RequestDeliveryState::NotSent)
    );
    assert_eq!(
        direct_bytes.borrow().len(),
        1,
        "{method} direct wrapper error: {direct_error:?}"
    );
    assert_eq!(batch_bytes.borrow().len(), 1);
    assert_eq!(default_bytes.borrow().len(), 1);
    assert_eq!(*direct_bytes.borrow(), *batch_bytes.borrow());
    assert_eq!(*default_bytes.borrow(), *batch_bytes.borrow());
    assert_eq!(*direct_options.borrow(), vec![options]);
    assert_eq!(*batch_options.borrow(), vec![options]);
    assert_eq!(*default_options.borrow(), vec![RequestOptions::default()]);
}

#[test]
#[allow(clippy::too_many_lines)] // Keep the exhaustive 14-method parity matrix in one focused test.
fn new_direct_methods_match_their_one_item_batch_wire_requests() {
    macro_rules! check {
        ($request:ty, $make:expr, $variant:path, $method:ident, $default_method:ident) => {
            assert_direct_matches_batch(
                stringify!($method),
                $make,
                $variant,
                |client, request, limits, options| client.$method(request, limits, options),
                |client, request, limits| client.$default_method(request, limits),
            );
        };
    }

    check!(
        DiscoverVersionsRequest,
        DiscoverVersionsRequest::new,
        ClientRequest::DiscoverVersions,
        discover_versions_with_options,
        discover_versions
    );
    check!(
        DelegatedLoginRequest,
        || DelegatedLoginRequest::new(Structure::new()),
        ClientRequest::DelegatedLogin,
        delegated_login_with_options,
        delegated_login
    );
    check!(
        LoginRequest,
        LoginRequest::new,
        ClientRequest::Login,
        login_with_options,
        login
    );
    check!(
        LogoutRequest,
        || LogoutRequest::new(
            Ticket::new(TicketType::LOGIN, SecretBytes::new(Vec::new()))
                .expect("empty test Ticket is representable")
        ),
        ClientRequest::Logout,
        logout_with_options,
        logout
    );
    check!(
        GetRequest,
        || GetRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString("get-id".to_owned(),)),
        ClientRequest::Get,
        get_with_options,
        get
    );
    check!(
        LocateRequest,
        || LocateRequest::new(AttributeSet::new()),
        ClientRequest::Locate,
        locate_with_options,
        locate
    );
    check!(
        AddAttributeRequest,
        || AddAttributeRequest::new(
            Some("object-id".to_owned()),
            NewAttribute::new(test_item(
                0x0054_0001,
                Value::text_string("test".to_owned()),
            )),
        ),
        ClientRequest::AddAttribute,
        add_attribute_with_options,
        add_attribute
    );
    check!(
        AdjustAttributeRequest,
        || AdjustAttributeRequest::new(
            Some("object-id".to_owned()),
            AttributeReference::tag(0x0054_0001),
            AdjustmentType::INCREMENT,
            None,
        ),
        ClientRequest::AdjustAttribute,
        adjust_attribute_with_options,
        adjust_attribute
    );
    check!(
        DeleteAttributeRequest,
        || DeleteAttributeRequest::new(
            Some("object-id".to_owned()),
            None,
            Some(AttributeReference::tag(0x0054_0001)),
        ),
        ClientRequest::DeleteAttribute,
        delete_attribute_with_options,
        delete_attribute
    );
    check!(
        ModifyAttributeRequest,
        || ModifyAttributeRequest::new(
            Some("object-id".to_owned()),
            None,
            NewAttribute::new(test_item(
                0x0054_0001,
                Value::text_string("test".to_owned()),
            )),
        ),
        ClientRequest::ModifyAttribute,
        modify_attribute_with_options,
        modify_attribute
    );
    check!(
        SetAttributeRequest,
        || SetAttributeRequest::new(
            Some("object-id".to_owned()),
            NewAttribute::new(test_item(
                0x0054_0001,
                Value::text_string("test".to_owned()),
            )),
        ),
        ClientRequest::SetAttribute,
        set_attribute_with_options,
        set_attribute
    );
    check!(
        GetAttributesRequest,
        || GetAttributesRequest::try_new(Some("object-id".to_owned()), [])
            .expect("empty Get Attributes references are valid"),
        ClientRequest::GetAttributes,
        get_attributes_with_options,
        get_attributes
    );
    check!(
        EncryptRequest,
        || {
            EncryptRequest::new(
                Some(UniqueIdentifier::TextString("object-id".to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(Vec::new()))),
            )
        },
        ClientRequest::Encrypt,
        encrypt_with_options,
        encrypt
    );
    check!(
        DecryptRequest,
        || {
            DecryptRequest::new(
                Some(UniqueIdentifier::TextString("object-id".to_owned())),
                Some(OperationData::ByteString(SecretBytes::new(Vec::new()))),
            )
        },
        ClientRequest::Decrypt,
        decrypt_with_options,
        decrypt
    );
}

#[test]
fn existing_option_methods_forward_the_options_to_the_exchange() {
    macro_rules! check {
        ($request:ty, $make:expr, $variant:path, $method:ident, $default_method:ident) => {
            assert_direct_matches_batch(
                stringify!($method),
                $make,
                $variant,
                |client, request, limits, options| client.$method(request, limits, options),
                |client, request, limits| client.$default_method(request, limits),
            );
        };
    }

    assert_direct_matches_batch(
        "get_constraints_with_options",
        || (),
        |()| ClientRequest::get_constraints(),
        |client, (), limits, options| client.get_constraints_with_options(limits, options),
        |client, (), limits| client.get_constraints(limits),
    );
    check!(
        GetAttributeListRequest,
        || GetAttributeListRequest::new(Some("object-id".to_owned())),
        ClientRequest::GetAttributeList,
        get_attribute_list_with_options,
        get_attribute_list
    );
    check!(
        SetConstraintsRequest,
        || SetConstraintsRequest::new(Structure::new()),
        ClientRequest::SetConstraints,
        set_constraints_with_options,
        set_constraints
    );
    check!(
        SetDefaultsRequest,
        SetDefaultsRequest::new,
        ClientRequest::SetDefaults,
        set_defaults_with_options,
        set_defaults
    );
    check!(
        GetUsageAllocationRequest,
        || GetUsageAllocationRequest::new(1)
            .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned()),),
        ClientRequest::GetUsageAllocation,
        get_usage_allocation_with_options,
        get_usage_allocation
    );
    check!(
        ValidateRequest,
        || ValidateRequest::new()
            .with_unique_identifier(UniqueIdentifier::TextString("object-id".to_owned(),)),
        ClientRequest::Validate,
        validate_with_options,
        validate
    );
    check!(
        RngRetrieveRequest,
        || RngRetrieveRequest::new(8),
        ClientRequest::RngRetrieve,
        rng_retrieve_with_options,
        rng_retrieve
    );
    check!(
        LogRequest,
        || LogRequest::new(SecretText::new("test".to_owned())),
        ClientRequest::Log,
        log_with_options,
        log
    );
    check!(
        InteropRequest,
        || InteropRequest::new(1, SecretText::new("test".to_owned())),
        ClientRequest::Interop,
        interop_with_options,
        interop
    );
    check!(
        RngSeedRequest,
        || RngSeedRequest::new(SecretBytes::new(Vec::new())),
        ClientRequest::RngSeed,
        rng_seed_with_options,
        rng_seed
    );
}
