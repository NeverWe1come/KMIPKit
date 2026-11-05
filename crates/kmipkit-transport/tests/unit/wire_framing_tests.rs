use crate::config::WireEncoding;

use super::TextDocumentFramer;

fn scan(encoding: WireEncoding, chunks: &[&[u8]]) -> Result<TextDocumentFramer, std::io::Error> {
    let mut framer = TextDocumentFramer::new(encoding)?;
    for chunk in chunks {
        for byte in *chunk {
            framer.feed(*byte)?;
        }
    }
    Ok(framer)
}

#[test]
fn json_document_completion_is_incremental_and_ignores_brackets_in_strings() {
    let document = br#" {"tag":"ResponseMessage","value":["brace } and [ slash \""]} 	"#;
    for split in 0..=document.len() {
        let chunks = [&document[..split], &document[split..]];
        let framer = scan(WireEncoding::Json, &chunks).expect("the complete JSON document scans");
        assert!(framer.is_complete(), "split at {split}");
    }
}

#[test]
fn json_document_framer_rejects_non_object_roots_and_trailing_values() {
    let error = scan(WireEncoding::Json, &[b"[1]"]).expect_err("array root is not KMIP JSON");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    let error = scan(
        WireEncoding::Json,
        &[br#"{"tag":"ResponseMessage","value":[]}"#, b" false"],
    )
    .expect_err("trailing non-whitespace is not part of one JSON document");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn text_framers_distinguish_other_supported_encodings() {
    for (encoding, bytes) in [
        (WireEncoding::Json, &b"<ResponseMessage/>"[..]),
        (WireEncoding::Json, &b"\x42\x00\x7b\x01"[..]),
        (WireEncoding::Xml, &b"{\"tag\":\"ResponseMessage\"}"[..]),
        (WireEncoding::Xml, &b"\x42\x00\x7b\x01"[..]),
    ] {
        let error = scan(encoding, &[bytes]).expect_err("another KMIP encoding is distinguished");
        assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    }
}

#[test]
fn json_document_framer_keeps_incomplete_values_open_and_accepts_trailing_space() {
    let incomplete = scan(WireEncoding::Json, &[br#"{"tag":"ResponseMessage""#])
        .expect("an incomplete document remains buffered");
    assert!(!incomplete.is_complete());

    let complete = scan(WireEncoding::Json, &[b"{} \r\n\t"])
        .expect("JSON whitespace may follow the root object");
    assert!(complete.is_complete());
}

#[test]
fn xml_document_completion_handles_fragments_comments_cdata_and_quoted_delimiters() {
    let document = b"<?xml version=\"1.0\"?><!-- prolog > --><ResponseMessage note=\"a > b\"><!-- child --><![CDATA[<not-an-element>]]><BatchItem/></ResponseMessage> <!-- epilog --><?after-root?> \n";
    let mut framer = TextDocumentFramer::new(WireEncoding::Xml).expect("XML framer is available");
    for byte in document {
        framer
            .feed(*byte)
            .expect("XML fragments scan incrementally");
    }
    assert!(framer.is_complete());
}

#[test]
fn xml_document_framer_rejects_doctypes_and_second_roots() {
    let mut doctype = TextDocumentFramer::new(WireEncoding::Xml).expect("XML framer is available");
    for byte in b"<!DOCTYPE ResponseMessage>" {
        if let Err(error) = doctype.feed(*byte) {
            assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
            break;
        }
    }
    assert!(!doctype.is_complete());

    let mut second_root =
        TextDocumentFramer::new(WireEncoding::Xml).expect("XML framer is available");
    let error = b"<ResponseMessage/><Unexpected/>"
        .iter()
        .find_map(|byte| second_root.feed(*byte).err())
        .expect("a second root is rejected");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
}

#[test]
fn xml_document_framer_leaves_incomplete_roots_open() {
    let incomplete = scan(WireEncoding::Xml, &[b"<ResponseMessage><BatchItem/>"])
        .expect("an incomplete root remains buffered");
    assert!(!incomplete.is_complete());
}

#[test]
fn xml_document_framer_accepts_a_utf8_bom_before_the_document() {
    let complete = scan(WireEncoding::Xml, &[b"\xef\xbb\xbf<ResponseMessage/>"])
        .expect("UTF-8 BOM and XML root are framed");
    assert!(complete.is_complete());
}

#[test]
fn text_framers_reject_invalid_starts_malformed_markup_and_foreign_data() {
    let error = TextDocumentFramer::new(WireEncoding::Ttlv)
        .expect_err("TTLV does not use a text document framer");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);

    for bytes in [
        &b"x"[..],
        b"\xef\xbbx",
        b" x",
        b"</ResponseMessage>",
        b"<!-x",
        b"<ResponseMessage><![CX",
        b"<ResponseMessage/>x",
        b" \t{\"tag\":\"ResponseMessage\"}",
    ] {
        let error = scan(WireEncoding::Xml, &[bytes])
            .expect_err("malformed XML framing or a foreign response is rejected");
        assert!(matches!(
            error.kind(),
            std::io::ErrorKind::InvalidData | std::io::ErrorKind::Unsupported
        ));
    }
}
