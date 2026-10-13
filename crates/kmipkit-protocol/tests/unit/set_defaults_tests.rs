//! Project-derived tests: OASIS KMIP v2.1 §6.1.53, Tables 328–330.
use crate::query_ping_fixtures::{item,structure,response_item,response_message};
use crate::{SetDefaultsRequest,SetDefaultsResponse};
use kmipkit_ttlv::Value;

#[test]
fn request_encodes_table_fields() {
    let request=SetDefaultsRequest::new(None);
    let payload=request.to_ttlv_payload().expect("valid payload");
    let tags=payload.view().children().iter().map(|f| f.tag().raw()).collect::<Vec<_>>();
    assert_eq!(tags, vec![]);
}
#[test]
fn complete_success_exposes_result() {
    let message=response_message(54,0,None,None,Some(structure([])));
    let response=SetDefaultsResponse::try_from_response_item(response_item(&message)).expect("table success");
    assert_eq!(response.result().status().raw(),0);
    assert!(!format!("{response:?}").contains("allocated-object"));
}
#[test]
fn failures_preserve_all_table_and_vendor_reasons() {
    // Tables 328–330: shared reasons plus operation-specific alternatives.
    for reason in [1,2,3,4,5,6,7,8,10,12,13,17,18,23,0x80000001] {
        let message=response_message(54,1,Some(reason),Some("server refused"),None);
        let response=SetDefaultsResponse::try_from_response_item(response_item(&message)).expect("KMIP failure");
        assert_eq!(response.result().reason().map(crate::ResultReason::raw),Some(reason));
        assert_eq!(response.result().message().map(crate::ResultMessage::as_str),Some("server refused"));
    }
}
#[test]
fn missing_success_payload_is_rejected() {
    let version=structure([item(0x42006a,Value::integer(2)),item(0x42006b,Value::integer(1))]);
    let header=structure([item(0x420069,Value::structure(version)),item(0x420092,Value::date_time(1)),item(0x42000d,Value::integer(1))]);
    let batch=structure([item(0x42005c,Value::enumeration(54)),item(0x42007f,Value::enumeration(0))]);
    assert!(crate::ResponseMessage::try_from_ttlv(structure([item(0x42007a,Value::structure(header)),item(0x42000f,Value::structure(batch))])).is_err());
}
#[test]
fn malformed_required_fields_are_rejected() {
    let message=response_message(54,0,None,None,Some(structure([item(0x420012,Value::integer(1))])));
    assert!(SetDefaultsResponse::try_from_response_item(response_item(&message)).is_err());
}
#[test]
fn other_operation_is_rejected() {
    let message=response_message(0x3b,0,None,None,Some(structure([])));
    assert!(SetDefaultsResponse::try_from_response_item(response_item(&message)).is_err());
}

#[test]
fn absent_defaults_remain_distinct_from_empty_structure() {
 assert!(SetDefaultsRequest::new(None).to_ttlv_payload().expect("omitted").view().children().is_empty());
 assert_eq!(SetDefaultsRequest::new(Some(structure([]))).to_ttlv_payload().expect("present").view().children().len(),1);
}
