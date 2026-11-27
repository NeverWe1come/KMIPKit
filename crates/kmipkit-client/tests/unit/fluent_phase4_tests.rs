//! Unit tests verifying Fase 4 advanced flows: batch builder, workflows,
//! multipart streams, PendingOperation, LocatePaginator, and ServerHandlerBuilder.

use kmipkit_protocol::{
    AttributesBuilder, CipherSpec, HashAlgorithm, KeySpec, ProtocolVersion, SecretBytes,
};
use kmipkit_test_support::ExchangeScript;

use crate::asynchronous_execution_test_support::client_for;
use crate::execute_test_support::{asynchronous_response_bytes, test_structure};
use crate::fluent::{CreateSymmetricKeyBuilder, CryptographicUsageMask, PendingOperation};

#[test]
fn fluent_batch_builder_assembles_ordered_placeholder_batches() {
    let ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ok,
        request_write_chunks: Vec::new(),
    });

    let batch = client
        .batch()
        .ordered()
        .try_add(
            CreateSymmetricKeyBuilder::detached()
                .with_key_spec(KeySpec::Aes256)
                .usage_mask(CryptographicUsageMask::ENCRYPT | CryptographicUsageMask::DECRYPT)
                .name("batch-aes")
                .to_batch_item(),
        )
        .with_protocol_version(ProtocolVersion::V2_1)
        .build()
        .expect("ordered batch builds cleanly");
    assert_eq!(batch.items().len(), 1);
    assert_eq!(batch.protocol_version(), Some(ProtocolVersion::V2_1));
}

#[test]
fn streams_workflows_pending_and_paginator_construct_cleanly() {
    let ok = asynchronous_response_bytes(0x0000_003b, 0, None, None, Some(test_structure([])));
    let (mut client, _, _) = client_for(ExchangeScript::Success {
        response: ok,
        request_write_chunks: Vec::new(),
    });

    let _hash = client.streams().hash(HashAlgorithm::Sha256);
    let _enc = client.streams().encrypt("k-1", CipherSpec::AesGcm);
    let _dec = client.streams().decrypt("k-1", CipherSpec::AesGcm);
    let _wf = client.workflows();

    let attrs = AttributesBuilder::new().name("k-1").build().unwrap();
    let _pager = client.keys().locate_iter(attrs, 10);

    let pending = PendingOperation::new(SecretBytes::new(vec![1, 2, 3, 4]));
    let mut corr_len = 0usize;
    pending.correlation_value().with_bytes(|b| corr_len = b.len());
    assert_eq!(corr_len, 4);
}
