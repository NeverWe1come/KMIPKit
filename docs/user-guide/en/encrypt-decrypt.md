# Encrypt and Decrypt with a typed client

The typed Encrypt and Decrypt requests send caller-selected values to the KMIP
server; KMIPKit does not encrypt locally or choose cryptographic parameters.
These examples target OASIS KMIP Specification v2.1 §6.1.17 Tables 214–216
(Encrypt) and §6.1.11 Tables 196–198 (Decrypt). Multipart field placement
follows §6.1 and §§7.3, 7.4, 7.8, 7.14, and 7.17.

The code shows caller-owned request construction, single-part and multipart
calls, handling for completed and Pending results, and an explicit Recover
before Encrypt when the caller already knows the object is archived. Each
`execute` call makes one exchange. A Pending result ends the shown flow so the
application can continue through its explicit asynchronous workflow; the
client does not poll or retry automatically.

```rust,kmipkit-test
use kmipkit_client::{
    Client, ClientBatch, ClientBatchItem, ClientBatchOutcome, ClientBatchResponse, ClientError,
    ClientRequest,
};
use kmipkit_protocol::{
    DecryptRequest, EncryptRequest, OperationData, RecoverRequest, SecretBytes, UniqueIdentifier,
};
use kmipkit_ttlv::{Structure, codec::CodecLimits};

fn observe_encrypt(response: &ClientBatchResponse) {
    match response.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
        }
        Some(ClientBatchOutcome::Encrypt(encrypted))
            if encrypted.result().status().known_name() == Some("Success") =>
        {
            let _ciphertext_length = encrypted
                .data()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
            let _multipart_value_length = encrypted
                .correlation_value()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
        }
        Some(ClientBatchOutcome::Encrypt(_)) => {
            // Preserve and handle the typed server result in application code.
        }
        _ => {}
    }
}

fn observe_decrypt(response: &kmipkit_client::ClientBatchResponse) {
    match response.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
        }
        Some(ClientBatchOutcome::Decrypt(decrypted))
            if decrypted.result().status().known_name() == Some("Success") =>
        {
            let _plaintext_length = decrypted
                .data()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
            let _multipart_value_length = decrypted
                .correlation_value()
                .map(|value| value.with_bytes(|bytes| bytes.len()));
        }
        Some(ClientBatchOutcome::Decrypt(_)) => {
            // Preserve and handle the typed server result in application code.
        }
        _ => {}
    }
}

pub fn encrypt_one_part(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    iv_selected_by_caller: Option<SecretBytes>,
    plaintext: SecretBytes,
) -> Result<ClientBatchResponse, ClientError> {
    let mut request = EncryptRequest::new(Some(key_id), Some(OperationData::ByteString(plaintext)));
    if let Some(parameters) = parameters_selected_by_caller {
        request = request.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = iv_selected_by_caller {
        request = request.with_iv_counter_nonce(iv);
    }

    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(request)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_encrypt(&response);
    Ok(response)
}

pub fn decrypt_one_part(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    iv_selected_by_caller: Option<SecretBytes>,
    aad_selected_by_caller: Option<SecretBytes>,
    tag_selected_by_caller: Option<SecretBytes>,
    ciphertext: SecretBytes,
) -> Result<ClientBatchResponse, ClientError> {
    let mut request =
        DecryptRequest::new(Some(key_id), Some(OperationData::ByteString(ciphertext)));
    if let Some(parameters) = parameters_selected_by_caller {
        request = request.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = iv_selected_by_caller {
        request = request.with_iv_counter_nonce(iv);
    }
    if let Some(aad) = aad_selected_by_caller {
        request = request.with_authenticated_encryption_additional_data(aad);
    }
    if let Some(tag) = tag_selected_by_caller {
        request = request.with_authenticated_encryption_tag(tag);
    }

    let response = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(request)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_decrypt(&response);
    Ok(response)
}

pub fn encrypt_in_two_parts(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    initial_parameters: Option<Structure>,
    final_parameters: Option<Structure>,
    initial_plaintext: SecretBytes,
    final_plaintext: SecretBytes,
    initial_aad: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let mut initial = EncryptRequest::new(
        Some(key_id.clone()),
        Some(OperationData::ByteString(initial_plaintext)),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(initial_aad);
    if let Some(parameters) = initial_parameters {
        initial = initial.with_cryptographic_parameters(parameters);
    }

    let first = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(initial)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    let correlation = match first.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![first]);
        }
        Some(ClientBatchOutcome::Encrypt(encrypted))
            if encrypted.result().status().known_name() == Some("Success") =>
        {
            let Some(value) = encrypted.correlation_value() else {
                return Ok(vec![first]);
            };
            value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()))
        }
        _ => return Ok(vec![first]),
    };

    let mut final_part = EncryptRequest::new(
        Some(key_id),
        Some(OperationData::ByteString(final_plaintext)),
    )
    .with_correlation_value(correlation)
    .with_final_indicator(true);
    if let Some(parameters) = final_parameters {
        final_part = final_part.with_cryptographic_parameters(parameters);
    }
    let last = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Encrypt(final_part)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_encrypt(&last);
    Ok(vec![first, last])
}

pub fn decrypt_in_two_parts(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    initial_parameters: Option<Structure>,
    final_parameters: Option<Structure>,
    initial_iv: Option<SecretBytes>,
    initial_ciphertext: SecretBytes,
    final_ciphertext: SecretBytes,
    initial_aad: SecretBytes,
    initial_tag: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let mut initial = DecryptRequest::new(
        Some(key_id.clone()),
        Some(OperationData::ByteString(initial_ciphertext)),
    )
    .with_init_indicator(true)
    .with_authenticated_encryption_additional_data(initial_aad)
    .with_authenticated_encryption_tag(initial_tag);
    if let Some(parameters) = initial_parameters {
        initial = initial.with_cryptographic_parameters(parameters);
    }
    if let Some(iv) = initial_iv {
        initial = initial.with_iv_counter_nonce(iv);
    }

    let first = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(initial)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    let correlation = match first.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![first]);
        }
        Some(ClientBatchOutcome::Decrypt(decrypted))
            if decrypted.result().status().known_name() == Some("Success") =>
        {
            let Some(value) = decrypted.correlation_value() else {
                return Ok(vec![first]);
            };
            value.with_bytes(|bytes| SecretBytes::new(bytes.to_vec()))
        }
        _ => return Ok(vec![first]),
    };

    let mut final_part = DecryptRequest::new(
        Some(key_id),
        Some(OperationData::ByteString(final_ciphertext)),
    )
    .with_correlation_value(correlation)
    .with_final_indicator(true);
    if let Some(parameters) = final_parameters {
        final_part = final_part.with_cryptographic_parameters(parameters);
    }
    let last = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Decrypt(final_part)))
            .with_asynchronous_indicator(1),
        limits,
    )?;
    observe_decrypt(&last);
    Ok(vec![first, last])
}

pub fn recover_then_encrypt(
    client: &mut Client,
    limits: &CodecLimits,
    key_id: UniqueIdentifier,
    parameters_selected_by_caller: Option<Structure>,
    plaintext: SecretBytes,
) -> Result<Vec<ClientBatchResponse>, ClientError> {
    let recovered = client.execute(
        ClientBatch::new(ClientBatchItem::new(ClientRequest::Recover(
            RecoverRequest::new(Some(key_id.clone())),
        )))
        .with_asynchronous_indicator(1),
        limits,
    )?;
    match recovered.get(0).map(|item| item.outcome()) {
        Some(ClientBatchOutcome::Pending(pending)) => {
            let _opaque_value_length = pending.asynchronous_correlation_value().len();
            return Ok(vec![recovered]);
        }
        Some(ClientBatchOutcome::Recover(response))
            if response.result().status().known_name() == Some("Success") =>
        {
            let _recovered_identifier = response.unique_identifier();
        }
        _ => return Ok(vec![recovered]),
    }

    let encrypted = encrypt_one_part(
        client,
        limits,
        key_id,
        parameters_selected_by_caller,
        None,
        plaintext,
    )?;
    Ok(vec![recovered, encrypted])
}

fn main() {}
```

The single-part functions return the full typed response so the caller can
inspect success, failure, or Pending. The multipart functions return the
responses received so far; if an initial response is Pending or otherwise
non-successful, they stop before issuing the next part. On completed success,
they carry only the exact server Correlation Value into the next caller-built
request; the copy is owned by `SecretBytes`. AAD and the Decrypt tag appear
only on the initial Decrypt request in this example. No field is moved
automatically. Applications can inspect and retain each typed KMIP result
according to their own policy; a successful Rust call alone does not mean the
KMIP operation succeeded.

The Recover sequence is likewise caller-controlled: it sends an explicit
Recover, proceeds to Encrypt only after a completed Success result, and stops
on Pending or another result while returning the response for caller handling.
This is the client precondition from KMIP v2.1
§6.1 for an object the caller knows is archived; KMIPKit does not infer object
state or recover automatically.
