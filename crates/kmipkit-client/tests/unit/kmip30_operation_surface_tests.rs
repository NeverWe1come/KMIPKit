//! Compile-time client surface for the seven KMIP 3.0 operations in Table 584.

use kmipkit_protocol::{
    CreateCredentialRequest, CreateGroupRequest, CreateUserRequest, DeactivateRequest,
    DecapsulateRequest, EncapsulateRequest, ObliterateRequest,
};
use kmipkit_transport::RequestOptions;
use kmipkit_ttlv::codec::CodecLimits;

use super::{Client, ClientBatchItemResponse, ClientError, ClientOperation};

type DirectMethod<Request> =
    fn(&mut Client, Request, &CodecLimits) -> Result<ClientBatchItemResponse, ClientError>;
type DirectMethodWithOptions<Request> = fn(
    &mut Client,
    Request,
    &CodecLimits,
    &RequestOptions,
) -> Result<ClientBatchItemResponse, ClientError>;

macro_rules! assert_direct_methods {
    ($request:ty, $name:ident, $options_name:ident) => {
        let _: DirectMethod<$request> = Client::$name;
        let _: DirectMethodWithOptions<$request> = Client::$options_name;
    };
}

#[test]
fn all_seven_kmip30_operations_have_exact_table_584_codes() {
    assert_eq!(ClientOperation::CreateGroup.raw(), 0x3C);
    assert_eq!(ClientOperation::Obliterate.raw(), 0x3D);
    assert_eq!(ClientOperation::CreateUser.raw(), 0x3E);
    assert_eq!(ClientOperation::CreateCredential.raw(), 0x3F);
    assert_eq!(ClientOperation::Deactivate.raw(), 0x40);
    assert_eq!(ClientOperation::Encapsulate.raw(), 0x41);
    assert_eq!(ClientOperation::Decapsulate.raw(), 0x42);
}

#[test]
fn all_seven_kmip30_operations_have_direct_and_options_methods() {
    assert_direct_methods!(
        CreateCredentialRequest,
        create_credential,
        create_credential_with_options
    );
    assert_direct_methods!(CreateGroupRequest, create_group, create_group_with_options);
    assert_direct_methods!(CreateUserRequest, create_user, create_user_with_options);
    assert_direct_methods!(DeactivateRequest, deactivate, deactivate_with_options);
    assert_direct_methods!(DecapsulateRequest, decapsulate, decapsulate_with_options);
    assert_direct_methods!(EncapsulateRequest, encapsulate, encapsulate_with_options);
    assert_direct_methods!(ObliterateRequest, obliterate, obliterate_with_options);
}
