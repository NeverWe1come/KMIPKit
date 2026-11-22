//! Compile-check the Rust RNG Retrieve examples in the bilingual guides.
//! They trace to OASIS KMIP v2.1 §6.1.49, Tables 316–318, and
//! KMIPKIT-0093-FR-004, FR-007, and FR-008; these are project examples, not
//! official conformance tests.

use kmipkit_ttlv::codec::CodecLimits;

use crate::{Client, ClientBatchItemResponse, ClientError};

const ENGLISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/en/cryptographic-operations.md");
const SPANISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/es/operaciones-criptograficas.md");

type RngRetrieveExample =
    fn(&mut Client, i32, &CodecLimits) -> Result<ClientBatchItemResponse, ClientError>;

#[allow(dead_code)]
fn request_rng_output(
    client: &mut Client,
    data_length: i32,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client
        .request()
        .rng_retrieve()
        .data_length(data_length)
        .execute(limits)
}

#[allow(dead_code)]
fn returned_rng_output_length(response: &ClientBatchItemResponse) -> Option<usize> {
    response
        .outcome()
        .response()
        .rng_retrieve()
        .and_then(|value| value.data())
        .map(|bytes| bytes.with_bytes(<[u8]>::len))
}

#[test]
fn both_guides_cite_the_operation_and_describe_secret_output() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("§6.1.49"));
        assert!(guide.contains(".rng_retrieve()"));
        assert!(guide.contains(".data_length(data_length)"));
        assert!(guide.contains("SecretBytes"));
        assert!(guide.contains("never polls or retries") || guide.contains("nunca ejecuta Poll"));
    }
}

#[test]
fn documented_rng_retrieve_example_compiles_against_the_client_api() {
    let _: RngRetrieveExample = request_rng_output;
    let _: fn(&ClientBatchItemResponse) -> Option<usize> = returned_rng_output_length;
}
