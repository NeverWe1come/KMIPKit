//! Receive KMIP 2.x TTLV and build the peer response with public tag constructors.

use crate::driver::{item, structure};
use crate::generic_wire::encode;
use crate::models::Result;
use kmipkit::protocol::{
    ProtocolVersion, QueryFunction, RequestBatchItemView, RequestMessage,
    ServerDiscoverVersionsRequest, ServerNotifyRequest, ServerPutRequest, ServerQueryRequest,
    SetEndpointRoleRequest,
};
use kmipkit::ttlv::{Item, KmipTag as Tag, Value, codec};
use kmipkit_transport::{RawTlsListener, TransportResponse};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const OPERATIONS: [u32; 5] = [0x1e, 0x1b, 0x1c, 0x18, 0x32];

pub(super) fn run(
    listener: &mut RawTlsListener,
    expected_version: ProtocolVersion,
    expected_operation: u32,
    timeout: Option<Duration>,
) -> Result<u32> {
    let mut channel = listener
        .accept(timeout)
        .map_err(|_| super::PEER_UNAVAILABLE)?;
    let limits = codec::CodecLimits::defaults();
    let packet = channel
        .recv_request(limits.max_message_bytes(), timeout)
        .map_err(|_| "failed to read incoming TTLV request")?;
    let request = codec::decode_with_limits(packet.as_bytes(), &limits)
        .map_err(|_| "incoming TTLV request could not be decoded")?;
    if request.tag().raw() != Tag::RequestMessage.as_u32() {
        return Err("unexpected incoming TTLV root");
    }
    let message = RequestMessage::try_from_ttlv(
        request
            .as_structure()
            .ok_or("incoming request is not a structure")?
            .try_clone()
            .map_err(|_| "could not retain incoming request")?,
    )
    .map_err(|_| "incoming TTLV envelope rejected")?;
    let header = message.header();
    let mut items = message.batch_items();
    if header.protocol_version() != expected_version || items.len() != 1 {
        return Err("incoming version or batch count mismatch");
    }
    let batch = items.next().ok_or("incoming batch item missing")?;
    if batch.operation() != Some(expected_operation) {
        return Err("incoming operation does not match the selected case");
    }
    let payload = payload(batch, expected_operation, expected_version)?;
    let status = u32::from(payload.is_none());
    let response = response(
        expected_operation,
        expected_version,
        batch,
        header.with_client_correlation_value(str::to_owned),
        payload,
    )?;
    let mut wire = Vec::new();
    encode(&response, &mut wire)?;
    if header
        .maximum_response_size()
        .and_then(|size| usize::try_from(size).ok())
        .is_some_and(|max| wire.len() > max)
    {
        return Err("incoming maximum response size would be exceeded");
    }
    let wire = TransportResponse::new(wire);
    channel
        .send_response(wire.as_bytes(), timeout)
        .map_err(|_| "failed to send the generic TTLV response")?;
    channel
        .close()
        .map_err(|_| "incoming channel close failed")?;
    Ok(status)
}

fn payload(
    batch: RequestBatchItemView<'_>,
    operation: u32,
    version: ProtocolVersion,
) -> Result<Option<Vec<Item>>> {
    let fields = match operation {
        0x1e => {
            let Ok(request) = ServerDiscoverVersionsRequest::try_from_request_item(batch) else {
                return Ok(None);
            };
            if request.protocol_versions().is_empty()
                || request.protocol_versions().contains(&version)
            {
                vec![version_item(version)?]
            } else {
                Vec::new()
            }
        }
        0x1b if ServerNotifyRequest::try_from_request_item(batch).is_ok() => Vec::new(),
        0x1c if ServerPutRequest::try_from_request_item(batch).is_ok() => Vec::new(),
        0x18 => {
            let Ok(request) = ServerQueryRequest::try_from_request_item(batch) else {
                return Ok(None);
            };
            let queries = request.query_functions();
            let mut fields = Vec::new();
            if queries.contains(&QueryFunction::OPERATIONS) {
                for code in OPERATIONS {
                    fields.push(scalar(Tag::Operation, Value::enumeration(code))?);
                }
            }
            if queries.contains(&QueryFunction::SERVER_INFORMATION) {
                fields.push(scalar(
                    Tag::VendorIdentification,
                    Value::text_string("KMIPKit".to_owned()),
                )?);
                fields.push(node(Tag::ServerInformation, Vec::new())?);
            }
            fields
        }
        0x32 => {
            let Ok(request) = SetEndpointRoleRequest::try_from_request_item(batch) else {
                return Ok(None);
            };
            vec![scalar(
                Tag::EndpointRole,
                Value::enumeration(request.endpoint_role().raw()),
            )?]
        }
        _ => return Ok(None),
    };
    Ok(Some(fields))
}

fn response(
    operation: u32,
    version: ProtocolVersion,
    request_batch: RequestBatchItemView<'_>,
    correlation: Option<String>,
    payload: Option<Vec<Item>>,
) -> Result<Item> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|value| i64::try_from(value.as_secs()).ok())
        .ok_or("response timestamp unavailable")?;
    let mut header = vec![
        version_item(version)?,
        scalar(Tag::TimeStamp, Value::date_time(timestamp))?,
    ];
    if let Some(correlation) = correlation {
        header.push(scalar(
            Tag::ClientCorrelationValue,
            Value::text_string(correlation),
        )?);
    }
    header.push(scalar(Tag::BatchCount, Value::integer(1))?);
    let mut batch = vec![scalar(Tag::Operation, Value::enumeration(operation))?];
    if let Some(id) = request_batch.with_unique_batch_item_id(<[u8]>::to_vec) {
        batch.push(scalar(Tag::UniqueBatchItemID, Value::byte_string(id))?);
    }
    batch.push(scalar(
        Tag::ResultStatus,
        Value::enumeration(u32::from(payload.is_none())),
    )?);
    if let Some(fields) = payload {
        batch.push(node(Tag::ResponsePayload, fields)?);
    } else {
        batch.push(scalar(Tag::ResultReason, Value::enumeration(7))?);
    }
    node(
        Tag::ResponseMessage,
        vec![
            node(Tag::ResponseHeader, header)?,
            node(Tag::BatchItem, batch)?,
        ],
    )
}

fn version_item(version: ProtocolVersion) -> Result<Item> {
    node(
        Tag::ProtocolVersion,
        vec![
            scalar(Tag::ProtocolVersionMajor, Value::integer(version.major()))?,
            scalar(Tag::ProtocolVersionMinor, Value::integer(version.minor()))?,
        ],
    )
}

fn scalar(tag: Tag, value: Value) -> Result<Item> {
    item(tag.as_u32(), value)
}

fn node(tag: Tag, fields: Vec<Item>) -> Result<Item> {
    scalar(tag, Value::structure(structure(tag.as_u32(), fields)?))
}
