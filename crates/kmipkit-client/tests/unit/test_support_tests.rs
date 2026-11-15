//! Unit tests for internal verification hooks (`pub mod test_support`).

#[cfg(feature = "test-support")]
mod tests {
    use kmipkit_ttlv::codec::CodecLimits;
    use kmipkit_ttlv::{Item, RawTag, Value};

    use crate::test_support;
    use crate::{ClientBatchItem, ClientRequest, WireEncoding};

    fn sample_item() -> Item {
        let tag = RawTag::new(0x42_000d)
            .expect("tag fits")
            .try_checked()
            .expect("tag is valid");
        Item::new(tag, Value::integer(1)).expect("item is valid")
    }

    #[test]
    fn test_support_encoded_batch_item_operation() {
        let item = ClientBatchItem::new(ClientRequest::discover_versions());
        let op = test_support::encoded_batch_item_operation(item).expect("operation succeeds");
        assert_eq!(op, 0x1e);
    }

    #[test]
    fn test_support_json_codec_roundtrip_and_errors() {
        let limits = CodecLimits::defaults();
        let item = sample_item();
        let encoded = test_support::json::encode(&item, &limits).expect("json encoding succeeds");
        let decoded =
            test_support::json::decode(&encoded, &limits).expect("json decoding succeeds");
        assert_eq!(item.tag(), decoded.tag());
        assert_eq!(item.item_type(), decoded.item_type());

        let err = test_support::json::decode(b"not valid json", &limits);
        assert!(err.is_err());

        let small_limits = CodecLimits::new(1, 1, 1).expect("limits are valid");
        let err_enc = test_support::json::encode(&item, &small_limits);
        assert!(err_enc.is_err());
    }

    #[test]
    fn test_support_xml_codec_roundtrip_and_errors() {
        let limits = CodecLimits::defaults();
        let item = sample_item();
        let encoded = test_support::xml::encode(&item, &limits).expect("xml encoding succeeds");
        let decoded = test_support::xml::decode(&encoded, &limits).expect("xml decoding succeeds");
        assert_eq!(item.tag(), decoded.tag());
        assert_eq!(item.item_type(), decoded.item_type());

        let err = test_support::xml::decode(b"<not-closed-xml", &limits);
        assert!(err.is_err());

        let small_limits = CodecLimits::new(1, 1, 1).expect("limits are valid");
        let err_enc = test_support::xml::encode(&item, &small_limits);
        assert!(err_enc.is_err());
    }

    #[test]
    fn test_support_s2c_decode_server_batch() {
        let limits = CodecLimits::defaults();
        let json_bytes = br#"{"tag":"RequestMessage","type":"Structure","value":[{"tag":"RequestHeader","type":"Structure","value":[{"tag":"ProtocolVersion","type":"Structure","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]},{"tag":"MaximumResponseSize","type":"Integer","value":65536},{"tag":"BatchCount","type":"Integer","value":1}]},{"tag":"BatchItem","type":"Structure","value":[{"tag":"Operation","type":"Enumeration","value":"DiscoverVersions"},{"tag":"RequestPayload","type":"Structure","value":[{"tag":"ProtocolVersion","type":"Structure","value":[{"tag":"ProtocolVersionMajor","type":"Integer","value":2},{"tag":"ProtocolVersionMinor","type":"Integer","value":1}]}]}]}]}"#;
        assert!(
            test_support::s2c::decode_server_batch(json_bytes, WireEncoding::Json, &limits,)
                .is_ok()
        );

        let xml_bytes = br#"<RequestMessage><RequestHeader><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion><MaximumResponseSize type="Integer" value="256"/><BatchCount type="Integer" value="1"/></RequestHeader><BatchItem><Operation type="Enumeration" value="DiscoverVersions"/><RequestPayload><ProtocolVersion><ProtocolVersionMajor type="Integer" value="2"/><ProtocolVersionMinor type="Integer" value="1"/></ProtocolVersion></RequestPayload></BatchItem></RequestMessage>"#;
        assert!(
            test_support::s2c::decode_server_batch(xml_bytes, WireEncoding::Xml, &limits,).is_ok()
        );

        let err = test_support::s2c::decode_server_batch(b"not valid", WireEncoding::Json, &limits);
        assert!(err.is_err());
    }

    #[test]
    fn test_transport_test_support_framer() {
        use kmipkit_transport::test_support::TextDocumentFramer;

        let mut framer = TextDocumentFramer::new(WireEncoding::Json).expect("framer creates");
        for b in br#"{"tag":"ResponseMessage"}"# {
            framer.feed(*b).expect("feed succeeds");
        }
        assert!(framer.is_complete());

        let err = TextDocumentFramer::new(WireEncoding::Ttlv);
        assert!(err.is_err());
    }
}
