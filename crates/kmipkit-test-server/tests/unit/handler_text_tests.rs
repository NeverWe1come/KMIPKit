use super::{WireEncoding, read_raw_text};

#[tokio::test]
async fn rejects_unclosed_document_at_the_limit_without_consuming_extra_bytes() {
    let mut rest = &b"  }"[..];
    let error = read_raw_text(&mut rest, b'{', WireEncoding::Json, 3)
        .await
        .expect_err("the document would exceed the limit");
    assert_eq!(error, "raw text request exceeds the KMIP codec limit");
    assert_eq!(rest, b"}");
}

#[tokio::test]
async fn accepts_document_ending_at_the_limit_and_leaves_the_next_document() {
    let mut rest = &b"}{}"[..];
    let frame = read_raw_text(&mut rest, b'{', WireEncoding::Json, 2)
        .await
        .expect("complete frame at the byte limit");
    assert_eq!(frame, b"{}");
    assert_eq!(rest, b"{}");
}
