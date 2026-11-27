"""External Python package consumer; all data stays local to this process."""

import os
from pathlib import Path

from kmipkit import (
    client,
    credentials,
    cryptographic_parameters,
    extensions,
    managed_objects,
    messages,
    protocol_values,
    ttlv,
    typed_attributes,
    validation,
)


def _item(raw_tag: int, value: ttlv.Value, limits: ttlv.CodecLimits) -> ttlv.TtlvItem:
    with ttlv.raw_tag_from_raw(raw_tag) as raw:
        tag = ttlv.raw_tag_try_checked(raw)
        try:
            return ttlv.ttlv_item_create(tag, value, limits)
        finally:
            tag.close()


def _append(
    structure: ttlv.Structure,
    raw_tag: int,
    value: ttlv.Value,
    limits: ttlv.CodecLimits,
) -> ttlv.Structure:
    return ttlv.ttlv_structure_with_item(structure, _item(raw_tag, value, limits), limits)


def _synthetic_ping_response(limits: ttlv.CodecLimits) -> ttlv.Structure:
    version = ttlv.ttlv_structure_create()
    version = _append(version, 0x0042_006A, ttlv.ttlv_value_integer(2), limits)
    version = _append(version, 0x0042_006B, ttlv.ttlv_value_integer(1), limits)
    header = ttlv.ttlv_structure_create()
    header = _append(header, 0x0042_0069, ttlv.ttlv_value_structure(version, limits), limits)
    header = _append(header, 0x0042_0092, ttlv.ttlv_value_date_time(1_751_100_123), limits)
    header = _append(header, 0x0042_000D, ttlv.ttlv_value_integer(1), limits)
    batch_item = ttlv.ttlv_structure_create()
    batch_item = _append(batch_item, 0x0042_005C, ttlv.ttlv_value_enumeration(0x3B), limits)
    batch_item = _append(batch_item, 0x0042_007F, ttlv.ttlv_value_enumeration(0), limits)
    batch_item = _append(
        batch_item, 0x0042_007C, ttlv.ttlv_value_structure(ttlv.ttlv_structure_create(), limits), limits
    )
    root = ttlv.ttlv_structure_create()
    root = _append(root, 0x0042_007A, ttlv.ttlv_value_structure(header, limits), limits)
    return _append(root, 0x0042_000F, ttlv.ttlv_value_structure(batch_item, limits), limits)


def _demo_authentication(limits: ttlv.CodecLimits) -> credentials.Authentication:
    value = ttlv.ttlv_structure_create()
    value = _append(value, 0x0042_00BB, ttlv.ttlv_value_text_string(b"demo-user", limits), limits)
    envelope = ttlv.ttlv_structure_create()
    envelope = _append(envelope, 0x0042_0024, ttlv.ttlv_value_enumeration(1), limits)
    envelope = _append(envelope, 0x0042_0025, ttlv.ttlv_value_structure(value, limits), limits)
    return credentials.Authentication.create([credentials.Credential.from_ttlv(envelope)])


def _aes_gcm_parameters() -> cryptographic_parameters.CryptographicParameters:
    builder = cryptographic_parameters.CryptographicParameters.builder()
    try:
        return (
            builder.cryptographic_algorithm(protocol_values.CryptographicAlgorithm.AES)
            .block_cipher_mode(protocol_values.BlockCipherMode.GCM)
            .iv_length(96)
            .tag_length(16)
            .build()
        )
    finally:
        builder.close()


def main() -> None:
    limits = ttlv.codec_limits_defaults()
    with limits:
        with ttlv.ttlv_value_text_string(b"local example", limits) as value:
            with typed_attributes.TypedAttribute.create(
                typed_attributes.StandardAttribute.CONTACT_INFORMATION, value
            ) as attribute:
                assert attribute.attribute is typed_attributes.StandardAttribute.CONTACT_INFORMATION

        opaque_fields = [
            _item(0x0042_0059, ttlv.ttlv_value_enumeration(1), limits),
            _item(0x0042_005A, ttlv.ttlv_value_byte_string(b"demo bytes", limits), limits),
        ]
        with managed_objects.ManagedObject.opaque_object(opaque_fields) as managed_object:
            assert managed_object.object_type == 8
        for field in opaque_fields:
            field.close()

        raw_material = _item(
            0x0042_0043, ttlv.ttlv_value_byte_string(b"demo key bytes", limits), limits
        )
        with managed_objects.KeyMaterial.from_item(1, raw_material) as key_material:
            assert key_material.format_raw == 1
        raw_material.close()

        with cryptographic_parameters.CryptographicParameters.builder() as builder:
            with builder.cryptographic_algorithm(0x8000_0001).build() as parameters:
                assert parameters.cryptographic_algorithm() == 0x8000_0001

        ping = extensions.client_batch_item_ping()
        with validation.validate_request(ping) as validation_report:
            assert validation_report.is_valid
        batch = client.Batch.create()
        batch.add_item(ping)
        batch.set_authentication(_demo_authentication(limits))
        assert batch.item_count == 1
        batch.close()

        with messages.ResponseMessage.from_ttlv(_synthetic_ping_response(limits)) as response:
            assert response.batch_item_count == 1
            with response.batch_item_at(0) as item:
                assert item.result_status == 0
                with item.to_ttlv() as response_ttlv:
                    with ttlv.ttlv_structure_view(response_ttlv) as response_view:
                        assert ttlv.ttlv_structure_view_item_count(response_view) == 3

        port = int(os.environ["KMIPKIT_CONSUMER_TEST_PORT"])
        fixtures = Path(os.environ["KMIPKIT_CONSUMER_TEST_FIXTURES"])
        registry = extensions.create_client_extension_registry(
            (), extensions.default_extension_registry_limits()
        )
        configuration = extensions.create_client_configuration(registry)
        transport = client.TransportConfig(
            endpoint=client.Endpoint.raw_tls("127.0.0.1", port, tls_server_name="localhost"),
            client_certificates=(
                client.TransportInput.file(
                    fixtures / "client-cert.pem", client.CredentialEncoding.PEM
                ),
            ),
            client_private_key=client.TransportInput.file(
                fixtures / "client-key.pem", client.CredentialEncoding.PEM
            ),
            trust=client.TrustSource.certificate_authorities(
                (
                    client.TransportInput.file(
                        fixtures / "server-cert.pem", client.CredentialEncoding.PEM
                    ),
                )
            ),
        )
        with configuration, client.Client.create(configuration, transport) as kmip_client:
            batch = client.Batch.create()
            batch.add_item(extensions.client_batch_item_ping())
            result = kmip_client.execute(batch, limits)
            raw = None
            try:
                item_result = result.items[0]
                assert item_result.ping_response is True
                assert item_result.result_status == 0
                raw = item_result.raw()
                assert raw is not None
                with ttlv.ttlv_structure_view(raw) as raw_view:
                    assert ttlv.ttlv_structure_view_item_count(raw_view) == 0
                print("python: Ping result and raw TTLV verified")
            finally:
                try:
                    if raw is not None:
                        raw.close()
                finally:
                    result.close()

            lifecycle_results: list[client.BatchResult | client.BatchItemResult] = []
            try:
                created = (
                    kmip_client.request()
                    .create_symmetric_key()
                    .aes_256()
                    .usage_mask(
                        protocol_values.CryptographicUsageMask.ENCRYPT
                        | protocol_values.CryptographicUsageMask.DECRYPT
                    )
                    .name("fluent-key")
                    .execute()
                )
                lifecycle_results.append(created)
                assert created.operation == protocol_values.Operation.CREATE
                assert created.outcome is client.BatchOutcome.COMPLETED
                assert created.result_status == 0
                assert created.create_object_type == 2
                identifier = created.unique_identifier
                assert identifier == extensions.UniqueIdentifier.text_string("fluent-key")

                get_item = kmip_client.request().get(identifier).execute()
                lifecycle_results.append(get_item)
                assert get_item.operation == protocol_values.Operation.GET
                assert get_item.outcome is client.BatchOutcome.COMPLETED
                assert get_item.result_status == 0
                get_response = get_item.get_response()
                assert get_response is not None
                assert get_response.unique_identifier == identifier
                assert get_response.object is not None

                plaintext = b"fluent plaintext"
                nonce = b"nonce-12byte"
                additional_data = b"fluent context"
                with _aes_gcm_parameters() as parameters:
                    encrypt_item = (
                        kmip_client.request()
                        .encrypt()
                        .unique_identifier(identifier)
                        .cryptographic_parameters(parameters)
                        .data(plaintext)
                        .iv_counter_nonce(nonce)
                        .authenticated_encryption_additional_data(additional_data)
                        .execute()
                    )
                lifecycle_results.append(encrypt_item)
                assert encrypt_item.operation == protocol_values.Operation.ENCRYPT
                assert encrypt_item.outcome is client.BatchOutcome.COMPLETED
                assert encrypt_item.result_status == 0
                encrypt_response = encrypt_item.encrypt_response
                assert encrypt_response is not None
                assert encrypt_response.unique_identifier == identifier
                ciphertext = encrypt_response.data
                response_nonce = encrypt_response.iv_counter_nonce
                authentication_tag = encrypt_response.authenticated_encryption_tag
                assert ciphertext == b"fixture ciphertext"
                assert response_nonce == nonce
                assert authentication_tag == b"fixture-auth-tag"

                with _aes_gcm_parameters() as parameters:
                    decrypt_item = (
                        kmip_client.request()
                        .decrypt()
                        .unique_identifier(identifier)
                        .cryptographic_parameters(parameters)
                        .data(ciphertext)
                        .iv_counter_nonce(response_nonce)
                        .authenticated_encryption_additional_data(additional_data)
                        .authenticated_encryption_tag(authentication_tag)
                        .execute()
                    )
                lifecycle_results.append(decrypt_item)
                assert decrypt_item.operation == protocol_values.Operation.DECRYPT
                assert decrypt_item.outcome is client.BatchOutcome.COMPLETED
                assert decrypt_item.result_status == 0
                decrypt_response = decrypt_item.decrypt_response
                assert decrypt_response is not None
                assert decrypt_response.unique_identifier == identifier
                assert decrypt_response.data == plaintext

                destroy_item = kmip_client.request().destroy(identifier).execute()
                lifecycle_results.append(destroy_item)
                assert destroy_item.operation == protocol_values.Operation.DESTROY
                assert destroy_item.outcome is client.BatchOutcome.COMPLETED
                assert destroy_item.result_status == 0
                destroy_response = destroy_item.destroy_response
                assert destroy_response is not None
                assert destroy_response.unique_identifier == identifier
                print("python: fluent AES-GCM Create/Get/Encrypt/Decrypt/Destroy verified over TLS 1.3/mTLS")

                pair_item = (
                    kmip_client.request()
                    .create_key_pair()
                    .algorithm(protocol_values.CryptographicAlgorithm.ECDSA)
                    .length(256)
                    .private_usage_mask(protocol_values.CryptographicUsageMask.SIGN)
                    .public_usage_mask(protocol_values.CryptographicUsageMask.VERIFY)
                    .private_name("fluent-private-key")
                    .public_name("fluent-public-key")
                    .execute()
                )
                lifecycle_results.append(pair_item)
                assert pair_item.operation == protocol_values.Operation.CREATE_KEY_PAIR
                assert pair_item.outcome is client.BatchOutcome.COMPLETED
                assert pair_item.result_status == 0
                private_identifier = pair_item.private_key_unique_identifier
                public_identifier = pair_item.public_key_unique_identifier
                assert private_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-private-key"
                )
                assert public_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-public-key"
                )

                with cryptographic_parameters.CryptographicParameters.builder() as builder:
                    with builder.hashing_algorithm(
                        protocol_values.HashingAlgorithm.SHA_256
                    ).digital_signature_algorithm(
                        protocol_values.DigitalSignatureAlgorithm.ECDSA_WITH_SHA256
                    ).build() as parameters:
                        sign_item = (
                            kmip_client.request()
                            .sign()
                            .unique_identifier(private_identifier)
                            .cryptographic_parameters(parameters)
                            .data(b"fluent sign input")
                            .execute()
                        )
                lifecycle_results.append(sign_item)
                assert sign_item.operation == protocol_values.Operation.SIGN
                assert sign_item.outcome is client.BatchOutcome.COMPLETED
                assert sign_item.result_status == 0
                sign_response = sign_item.sign_response
                assert sign_response is not None
                signature = bytearray()
                try:
                    signature = bytearray(sign_response.signature_data or b"")
                    assert sign_response.unique_identifier == private_identifier
                    assert signature == b"fixture signature"
                finally:
                    for offset in range(len(signature)):
                        signature[offset] = 0
                    sign_response.response_ttlv.close()

                for key_identifier in (private_identifier, public_identifier):
                    destroyed_item = kmip_client.request().destroy(key_identifier).execute()
                    lifecycle_results.append(destroyed_item)
                    assert destroyed_item.operation == protocol_values.Operation.DESTROY
                    assert destroyed_item.outcome is client.BatchOutcome.COMPLETED
                    assert destroyed_item.result_status == 0
                    destroy_response = destroyed_item.destroy_response
                    assert destroy_response is not None
                    assert destroy_response.unique_identifier == key_identifier
                print(
                    "python: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS"
                )

                with ttlv.ttlv_value_text_string(b"fluent expert journey", limits) as vendor_value:
                    vendor_value_item = _item(0x0054_0001, vendor_value, limits)
                    try:
                        with kmip_client.request().create_symmetric_key() as builder:
                            expert_create_item = (
                                builder.aes_256()
                                .usage_mask(
                                    protocol_values.CryptographicUsageMask.ENCRYPT
                                    | protocol_values.CryptographicUsageMask.DECRYPT
                                )
                                .name("fluent-expert-key")
                                .add_vendor_attribute(
                                    "KMIPKit.Example", "purpose", vendor_value_item
                                )
                                .execute()
                            )
                        lifecycle_results.append(expert_create_item)
                    finally:
                        vendor_value_item.close()

                assert expert_create_item.operation == protocol_values.Operation.CREATE
                assert expert_create_item.outcome is client.BatchOutcome.COMPLETED
                assert expert_create_item.result_status == 0
                assert expert_create_item.create_object_type == 2
                expert_identifier = expert_create_item.unique_identifier
                assert expert_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-expert-key"
                )

                expert_get_item = kmip_client.request().get(expert_identifier).execute()
                lifecycle_results.append(expert_get_item)
                assert expert_get_item.operation == protocol_values.Operation.GET
                assert expert_get_item.outcome is client.BatchOutcome.COMPLETED
                assert expert_get_item.result_status == 0
                expert_get_response = expert_get_item.get_response()
                assert expert_get_response is not None
                try:
                    assert expert_get_response.unique_identifier == expert_identifier
                    assert expert_get_response.object is not None
                finally:
                    expert_get_response.close()

                expert_destroy_item = kmip_client.request().destroy(expert_identifier).execute()
                lifecycle_results.append(expert_destroy_item)
                assert expert_destroy_item.operation == protocol_values.Operation.DESTROY
                assert expert_destroy_item.outcome is client.BatchOutcome.COMPLETED
                assert expert_destroy_item.result_status == 0
                expert_destroy_response = expert_destroy_item.destroy_response
                assert expert_destroy_response is not None
                assert expert_destroy_response.unique_identifier == expert_identifier
                print(
                    "python: fluent Vendor Attribute Create/Get/Destroy verified over TLS 1.3/mTLS"
                )

                with client.Batch.create() as batch:
                    batch.set_ordered(True)
                    with kmip_client.request().create_symmetric_key() as builder:
                        batch.add_item(
                            extensions.with_unique_batch_item_id(
                                builder.aes_256()
                                .usage_mask(
                                    protocol_values.CryptographicUsageMask.ENCRYPT
                                    | protocol_values.CryptographicUsageMask.DECRYPT
                                )
                                .name("fluent-batch-key")
                                .build(),
                                b"fluent-batch-create",
                            )
                        )
                    batch.add_item(
                        extensions.with_unique_batch_item_id(
                            extensions.client_batch_item_get(), b"fluent-batch-get"
                        )
                    )
                    batched = kmip_client.execute(batch, limits)
                lifecycle_results.append(batched)

                assert len(batched.items) == 2
                batch_create_item, batch_get_item = batched.items
                assert batch_create_item.operation == protocol_values.Operation.CREATE
                assert batch_create_item.outcome is client.BatchOutcome.COMPLETED
                assert batch_create_item.result_status == 0
                assert batch_create_item.create_object_type == 2
                batch_identifier = batch_create_item.unique_identifier
                assert batch_identifier == extensions.UniqueIdentifier.text_string(
                    "fluent-batch-key"
                )

                assert batch_get_item.operation == protocol_values.Operation.GET
                assert batch_get_item.outcome is client.BatchOutcome.COMPLETED
                assert batch_get_item.result_status == 0
                batch_get_response = batch_get_item.get_response()
                assert batch_get_response is not None
                try:
                    assert batch_get_response.unique_identifier == batch_identifier
                    assert batch_get_response.object is not None
                finally:
                    batch_get_response.close()
                print(
                    "python: fluent ordered batch Create/Get with ID Placeholder verified over TLS 1.3/mTLS"
                )
            finally:
                for lifecycle_result in lifecycle_results:
                    lifecycle_result.close()

if __name__ == "__main__":
    main()
