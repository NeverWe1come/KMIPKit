//! Fake-transport parity for the additive KMIPKIT-0050 direct methods.
//!
//! Operation direction and names follow OASIS KMIP v2.1 §6.1. These checks
//! compare the encoded one-item Request Message with the corresponding
//! `ClientBatch` call and verify that per-exchange options reach the transport.

use std::cell::RefCell;
use std::io;
use std::rc::Rc;

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

fn client(requests: Rc<RefCell<Vec<Vec<u8>>>>, options: Rc<RefCell<Vec<RequestOptions>>>) -> Client {
    Client::for_test_with_options_observer(NotSentTransport(requests), options)
}

fn assert_direct_matches_batch<Request>(
    make_request: impl Fn() -> Request,
    to_client_request: impl Fn(Request) -> ClientRequest,
    direct_call: impl FnOnce(
        &mut Client,
        Request,
        &CodecLimits,
        &RequestOptions,
    ) -> Result<crate::ClientBatchItemResponse, ClientError>,
) {
    let limits = CodecLimits::defaults();
    let options = RequestOptions::default().with_connect(TimeoutLimit::Unbounded);
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

    assert_eq!(direct_error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(batch_error.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(direct_bytes.borrow().len(), 1);
    assert_eq!(batch_bytes.borrow().len(), 1);
    assert_eq!(*direct_bytes.borrow(), *batch_bytes.borrow());
    assert_eq!(*direct_options.borrow(), vec![options]);
    assert_eq!(*batch_options.borrow(), vec![options]);
}

#[test]
fn new_direct_methods_match_their_one_item_batch_wire_requests() {
    macro_rules! check {
        ($request:ty, $make:expr, $variant:path, $method:ident) => {
            assert_direct_matches_batch(
                $make,
                $variant,
                |client, request, limits, options| {
                    client.$method(request, limits, options)
                },
            );
        };
    }

    check!(
        DiscoverVersionsRequest,
        || DiscoverVersionsRequest::new(),
        ClientRequest::DiscoverVersions,
        discover_versions_with_options
    );
    check!(
        DelegatedLoginRequest,
        || DelegatedLoginRequest::new(Structure::new()),
        ClientRequest::DelegatedLogin,
        delegated_login_with_options
    );
    check!(LoginRequest, || LoginRequest::new(), ClientRequest::Login, login_with_options);
    check!(
        LogoutRequest,
        || LogoutRequest::new(
            Ticket::new(TicketType::LOGIN, SecretBytes::new(Vec::new()))
                .expect("empty test Ticket is representable")
        ),
        ClientRequest::Logout,
        logout_with_options
    );
    check!(GetRequest, || GetRequest::new(), ClientRequest::Get, get_with_options);
    check!(
        LocateRequest,
        || LocateRequest::new(AttributeSet::new()),
        ClientRequest::Locate,
        locate_with_options
    );
    check!(
        AddAttributeRequest,
        || AddAttributeRequest::new(
            None,
            NewAttribute::new(test_item(0x0042_0001, Value::date_time(0))),
        ),
        ClientRequest::AddAttribute,
        add_attribute_with_options
    );
    check!(
        AdjustAttributeRequest,
        || AdjustAttributeRequest::new(
            None,
            AttributeReference::tag(0x0042_0001),
            AdjustmentType::INCREMENT,
            None,
        ),
        ClientRequest::AdjustAttribute,
        adjust_attribute_with_options
    );
    check!(
        DeleteAttributeRequest,
        || DeleteAttributeRequest::new(
            None,
            None,
            Some(AttributeReference::tag(0x0042_0001)),
        ),
        ClientRequest::DeleteAttribute,
        delete_attribute_with_options
    );
    check!(
        ModifyAttributeRequest,
        || ModifyAttributeRequest::new(
            None,
            None,
            NewAttribute::new(test_item(0x0042_0001, Value::date_time(0))),
        ),
        ClientRequest::ModifyAttribute,
        modify_attribute_with_options
    );
    check!(
        SetAttributeRequest,
        || SetAttributeRequest::new(
            None,
            NewAttribute::new(test_item(0x0042_0001, Value::date_time(0))),
        ),
        ClientRequest::SetAttribute,
        set_attribute_with_options
    );
    check!(
        GetAttributesRequest,
        || GetAttributesRequest::try_new(None, [])
            .expect("empty Get Attributes references are valid"),
        ClientRequest::GetAttributes,
        get_attributes_with_options
    );
    check!(
        EncryptRequest,
        || EncryptRequest::new(None, None),
        ClientRequest::Encrypt,
        encrypt_with_options
    );
    check!(
        DecryptRequest,
        || DecryptRequest::new(None, None),
        ClientRequest::Decrypt,
        decrypt_with_options
    );
}
