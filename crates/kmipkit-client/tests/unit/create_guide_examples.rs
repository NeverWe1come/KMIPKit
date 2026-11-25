//! Compile-check the fluent Create example in both user guides.
//! It traces to KMIP v2.1 §6.1.8, Tables 186–188; these examples are informative, not conformance tests.

use kmipkit_protocol::{AttributeSet, ObjectType};
use kmipkit_ttlv::codec::CodecLimits;

use crate::{Client, ClientBatchItemResponse, ClientError, ClientOperation};

const ENGLISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/en/managed-objects-and-key-material.md");
const SPANISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/es/objetos-gestionados-y-material-de-clave.md");

type CreateExample =
    fn(&mut Client, &CodecLimits, ObjectType, AttributeSet) -> Result<(), ClientError>;

#[allow(dead_code)]
fn create_object(
    client: &mut Client,
    limits: &CodecLimits,
    object_type: ObjectType,
    attributes: AttributeSet,
) -> Result<(), ClientError> {
    let item: ClientBatchItemResponse = client
        .request()
        .create()
        .object_type(object_type)
        .attributes(attributes)
        .execute(limits)?;
    let outcome = item.outcome();
    assert_eq!(outcome.operation(), ClientOperation::Create);
    let response_view = outcome.response();
    let response = response_view.create().expect("typed Create response");
    let _status = response.result().status().raw();
    let _object_type = response.object_type();
    let _unique_identifier = response.unique_identifier();
    Ok(())
}

#[test]
fn both_guides_cite_the_create_tables_and_preserve_caller_policy_choices() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("§6.1.8"));
        assert!(guide.contains("Tables 186–188") || guide.contains("Tablas 186–188"));
        assert!(guide.contains(".create()"));
        assert!(guide.contains(".object_type(object_type)"));
        assert!(guide.contains(".attributes(attributes)"));
        assert!(guide.contains("Protection Storage Masks"));
    }
    assert!(ENGLISH_GUIDE.contains("does not choose"));
    assert!(SPANISH_GUIDE.contains("KMIPKit no elige"));
}

#[test]
fn documented_fluent_create_example_compiles_against_the_client_api() {
    let _: CreateExample = create_object;
}
