//! Compile-check the Rust RNG Seed examples in the bilingual user guides.
//! They trace to OASIS KMIP v2.1 §6.1.50, Tables 319–321; they are project examples, not
//! official conformance tests.

use kmipkit_protocol::SecretBytes;
use kmipkit_ttlv::codec::CodecLimits;

use crate::{Client, ClientBatchItemResponse, ClientError};

const ENGLISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/en/cryptographic-operations.md");
const SPANISH_GUIDE: &str =
    include_str!("../../../../docs/user-guide/es/operaciones-criptograficas.md");

type RngSeedExample =
    fn(&mut Client, Vec<u8>, &CodecLimits) -> Result<ClientBatchItemResponse, ClientError>;

type ReportedSeedLength = fn(&ClientBatchItemResponse) -> Option<i32>;

#[allow(dead_code)]
fn seed_server_rng(
    client: &mut Client,
    seed: Vec<u8>,
    limits: &CodecLimits,
) -> Result<ClientBatchItemResponse, ClientError> {
    client
        .request()
        .rng_seed()
        .data(SecretBytes::new(seed))
        .execute_outcome_with_limits(limits)
}

#[allow(dead_code)]
fn server_reported_seed_length(response: &ClientBatchItemResponse) -> Option<i32> {
    response
        .outcome()
        .response()
        .rng_seed()
        .and_then(kmipkit_protocol::RngSeedResponse::data_length)
}

#[test]
fn both_guides_cite_rng_seed_and_explain_secret_and_zero_length_semantics() {
    for guide in [ENGLISH_GUIDE, SPANISH_GUIDE] {
        assert!(guide.contains("§6.1.50"));
        assert!(guide.contains(".rng_seed()"));
        assert!(guide.contains(".data(SecretBytes::new(seed))"));
        assert!(guide.contains("zero") || guide.contains("cero"));
        assert!(guide.contains("never polls or retries") || guide.contains("nunca ejecuta Poll"));
    }
}

#[test]
fn documented_rng_seed_example_compiles_against_the_client_api() {
    let _: RngSeedExample = seed_server_rng;
    let _: ReportedSeedLength = server_reported_seed_length;
}
