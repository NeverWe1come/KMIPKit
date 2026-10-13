//! Project-derived tests: OASIS KMIP v2.1 §6.1.52, Tables 325–327.
use crate::query_ping_fixtures::{item,structure,response_item,response_message};
use crate::{SetConstraintsRequest,SetConstraintsResponse};
use kmipkit_ttlv::Value;

#[test]
fn request_encodes_table_fields() {
    let request=SetConstraintsRequest::new(structure([]));
    let payload=request.to_ttlv_payload().expect("valid payload");
    let tags=payload.view().children().iter().map(|f| f.tag().raw()).collect::<Vec<_>>();
    assert_eq!(tags, vec![0x420168]);
}
#[test]
fn complete_success_exposes_result() {
    let message=response_message(55,0,None,None,Some(structure([])));
    let response=SetConstraintsResponse::try_from_response_item(response_item(&message)).expect("table success");
    assert_eq!(response.result().status().raw(),0);
    assert!(!format!("{response:?}").contains("allocated-object"));
}
#[test]
fn failures_preserve_all_table_and_vendor_reasons() {
    // Tables 325–327: shared reasons plus operation-specific alternatives.
    for reason in [1,2,3,4,5,6,7,8,10,12,13,17,18,23,0x80000001] {
        let message=response_message(55,1,Some(reason),Some("server refused"),None);
        let response=SetConstraintsResponse::try_from_response_item(response_item(&message)).expect("KMIP failure");
        assert_eq!(response.result().reason().map(crate::ResultReason::raw),Some(reason));
        assert_eq!(response.result().message().map(crate::ResultMessage::as_str),Some("server refused"));
    }
}
#[test]
fn missing_success_payload_is_rejected() {
    let version=structure([item(0x42006a,Value::integer(2)),item(0x42006b,Value::integer(1))]);
    let header=structure([item(0x420069,Value::structure(version)),item(0x420092,Value::date_time(1)),item(0x42000d,Value::integer(1))]);
    let batch=structure([item(0x42005c,Value::enumeration(55)),item(0x42007f,Value::enumeration(0))]);
    assert!(crate::ResponseMessage::try_from_ttlv(structure([item(0x42007a,Value::structure(header)),item(0x42000f,Value::structure(batch))])).is_err());
}
#[test]
fn malformed_required_fields_are_rejected() {
    let message=response_message(55,0,None,None,Some(structure([item(0x420012,Value::integer(1))])));
    assert!(SetConstraintsResponse::try_from_response_item(response_item(&message)).is_err());
}
#[test]
fn other_operation_is_rejected() {
    let message=response_message(0x3b,0,None,None,Some(structure([])));
    assert!(SetConstraintsResponse::try_from_response_item(response_item(&message)).is_err());
}
