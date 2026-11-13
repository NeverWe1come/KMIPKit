"""External Python consumer for the real TLS 1.3/mTLS lifecycle journey."""

import os
from pathlib import Path

from kmipkit import client, cryptographic_parameters, errors, extensions, protocol_values, ttlv
from kmipkit.typed_attributes import StandardAttribute, TypedAttribute


AES_GCM_PLAINTEXT = b"KMIPKit AES-GCM journey"
AES_GCM_CIPHERTEXT = bytes.fromhex("0c4f9f4b8e8cb63bcc04c4a6f6aa354de9b9f2469e1e26")
AES_GCM_IV = bytes(range(12))
AES_GCM_AAD = b"kmipkit-gcm-aad"
AES_GCM_TAG = bytes.fromhex("b516ad2a79bf55d147482f13feed6ea6")
SIGNATURE_PLAINTEXT = b"KMIPKit ECDSA signature journey"
SIGNATURE_DATA = bytes.fromhex(
    "3045022100aabbccddeeff0011223344556677889902210033445566778899aabbccddeeff0011"
)


def _aes_gcm_parameters() -> cryptographic_parameters.CryptographicParameters:
    builder = cryptographic_parameters.CryptographicParameters.builder()
    try:
        return (
            builder.cryptographic_algorithm(protocol_values.CryptographicAlgorithm.AES)
            .block_cipher_mode(protocol_values.BlockCipherMode.GCM)
            .iv_length(96)
            .tag_length(128)
            .build()
        )
    finally:
        builder.close()


def execute_aes_gcm_journey(kmip_client: client.Client, limits: ttlv.CodecLimits) -> None:
    with kmip_client.request().create_symmetric_key() as builder:
        created = (
            builder.aes_256()
            .usage_mask(
                protocol_values.CryptographicUsageMask.ENCRYPT
                | protocol_values.CryptographicUsageMask.DECRYPT
            )
            .name("fluent-aes-gcm-key")
            .execute(limits)
        )
    try:
        create_item = created.items[0]
        assert create_item.operation == protocol_values.Operation.CREATE
        assert create_item.outcome is client.BatchOutcome.COMPLETED
        assert create_item.result_status == 0
        identifier = create_item.unique_identifier
        assert identifier == extensions.UniqueIdentifier.text_string("fluent-aes-gcm-key")
    finally:
        created.close()

    parameters = _aes_gcm_parameters()
    try:
        with kmip_client.request().encrypt() as builder:
            encrypted = (
                builder.unique_identifier(identifier)
                .data(AES_GCM_PLAINTEXT)
                .cryptographic_parameters(parameters)
                .iv_counter_nonce(AES_GCM_IV)
                .authenticated_encryption_additional_data(AES_GCM_AAD)
                .execute(limits)
            )
        try:
            encrypt_item = encrypted.items[0]
            encrypt_response = encrypt_item.encrypt_response
            assert encrypt_item.operation == protocol_values.Operation.ENCRYPT
            assert encrypt_item.outcome is client.BatchOutcome.COMPLETED
            assert encrypt_item.result_status == 0
            assert encrypt_response is not None
            assert encrypt_response.unique_identifier == identifier
            assert encrypt_response.data == AES_GCM_CIPHERTEXT
            assert encrypt_response.iv_counter_nonce == AES_GCM_IV
            assert encrypt_response.authenticated_encryption_tag == AES_GCM_TAG
            ciphertext = encrypt_response.data
            authentication_tag = encrypt_response.authenticated_encryption_tag
            assert ciphertext is not None and authentication_tag is not None
        finally:
            encrypted.close()
    finally:
        parameters.close()

    parameters = _aes_gcm_parameters()
    try:
        with kmip_client.request().decrypt() as builder:
            decrypted = (
                builder.unique_identifier(identifier)
                .data(ciphertext)
                .cryptographic_parameters(parameters)
                .iv_counter_nonce(AES_GCM_IV)
                .authenticated_encryption_additional_data(AES_GCM_AAD)
                .authenticated_encryption_tag(authentication_tag)
                .execute(limits)
            )
        try:
            decrypt_item = decrypted.items[0]
            decrypt_response = decrypt_item.decrypt_response
            assert decrypt_item.operation == protocol_values.Operation.DECRYPT
            assert decrypt_item.outcome is client.BatchOutcome.COMPLETED
            assert decrypt_item.result_status == 0
            assert decrypt_response is not None
            assert decrypt_response.unique_identifier == identifier
            assert decrypt_response.data == AES_GCM_PLAINTEXT
        finally:
            decrypted.close()
    finally:
        parameters.close()

    destroyed = kmip_client.request().destroy(identifier).execute(limits)
    try:
        destroy_item = destroyed.items[0]
        assert destroy_item.operation == protocol_values.Operation.DESTROY
        assert destroy_item.outcome is client.BatchOutcome.COMPLETED
        assert destroy_item.result_status == 0
        assert destroy_item.destroy_response is not None
        assert destroy_item.destroy_response.unique_identifier == identifier
    finally:
        destroyed.close()
    print("python: fluent AES-GCM journey verified")


def _ecdsa_sha256_parameters() -> cryptographic_parameters.CryptographicParameters:
    builder = cryptographic_parameters.CryptographicParameters.builder()
    try:
        return (
            builder.hashing_algorithm(protocol_values.HashingAlgorithm.SHA_256)
            .digital_signature_algorithm(
                protocol_values.DigitalSignatureAlgorithm.ECDSA_WITH_SHA256
            )
            .build()
        )
    finally:
        builder.close()


def execute_key_pair_signature_journey(
    kmip_client: client.Client, limits: ttlv.CodecLimits
) -> None:
    with ttlv.raw_tag_from_raw(0x420075) as raw, ttlv.raw_tag_try_checked(raw) as tag:
        curve = ttlv.ttlv_value_enumeration(protocol_values.RecommendedCurve.P_256)
        curve_item = ttlv.ttlv_item_create(tag, curve, limits)
        domain = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), curve_item, limits)
    with ttlv.ttlv_value_structure(domain, limits) as value, TypedAttribute.create(
        StandardAttribute.CRYPTOGRAPHIC_DOMAIN_PARAMETERS, value
    ) as domain_attribute:
        created = (
            kmip_client.request()
            .create_key_pair()
            .algorithm(protocol_values.CryptographicAlgorithm.ECDSA)
            .length(256)
            .common_attribute(domain_attribute)
            .private_usage_mask(protocol_values.CryptographicUsageMask.SIGN)
            .public_usage_mask(protocol_values.CryptographicUsageMask.VERIFY)
            .execute(limits)
        )
    try:
        create_item = created.items[0]
        assert create_item.operation == protocol_values.Operation.CREATE_KEY_PAIR
        assert create_item.outcome is client.BatchOutcome.COMPLETED
        assert create_item.result_status == 0
        private_identifier = create_item.private_key_unique_identifier
        public_identifier = create_item.public_key_unique_identifier
        assert private_identifier is not None and public_identifier is not None
    finally:
        created.close()

    parameters = _ecdsa_sha256_parameters()
    try:
        with kmip_client.request().sign() as builder:
            signed = (
                builder.unique_identifier(private_identifier)
                .data(SIGNATURE_PLAINTEXT)
                .cryptographic_parameters(parameters)
                .execute(limits)
            )
        sign_response = None
        try:
            sign_item = signed.items[0]
            sign_response = sign_item.sign_response
            assert sign_item.operation == protocol_values.Operation.SIGN
            assert sign_item.outcome is client.BatchOutcome.COMPLETED
            assert sign_item.result_status == 0
            assert sign_response is not None
            assert sign_response.unique_identifier == private_identifier
            assert sign_response.signature_data == SIGNATURE_DATA
            signature = bytearray(sign_response.signature_data)
        finally:
            if sign_response is not None:
                sign_response.response_ttlv.close()
            signed.close()
    finally:
        parameters.close()

    parameters = _ecdsa_sha256_parameters()
    try:
        with kmip_client.request().signature_verify() as builder:
            verified = (
                builder.unique_identifier(public_identifier)
                .data(SIGNATURE_PLAINTEXT)
                .signature_data(signature)
                .cryptographic_parameters(parameters)
                .execute(limits)
            )
        verify_response = None
        try:
            verify_item = verified.items[0]
            verify_response = verify_item.signature_verify_response
            assert verify_item.operation == protocol_values.Operation.SIGNATURE_VERIFY
            assert verify_item.outcome is client.BatchOutcome.COMPLETED
            assert verify_item.result_status == 0
            assert verify_response is not None
            assert verify_response.unique_identifier == public_identifier
            assert verify_response.validity_indicator_raw == 1
        finally:
            if verify_response is not None:
                verify_response.response_ttlv.close()
            verified.close()
    finally:
        parameters.close()
        signature[:] = bytes(len(signature))
    print("python: fluent Create Key Pair/Sign/Signature Verify journey verified")


def execute_expert_attribute_journey(
    kmip_client: client.Client, limits: ttlv.CodecLimits
) -> None:
    with ttlv.ttlv_value_integer(1) as wrong_value:
        with kmip_client.request().create_symmetric_key() as invalid_builder:
            try:
                invalid_builder.add_attribute(StandardAttribute.OBJECT_GROUP, wrong_value)
            except errors.ClientExecutionError as failure:
                assert failure.status == 13
                assert failure.client_category is errors.ClientErrorCategory.VALIDATION
                assert failure.cause is errors.ClientErrorCause.INVALID_INPUT
                assert failure.delivery is errors.DeliveryState.NOT_SENT
                assert failure.response_ttlv is None
            else:
                raise AssertionError("wrong-type Object Group was accepted by the fluent builder")

    with ttlv.ttlv_value_text_string(b"fluent-expert-group", limits) as object_group:
        with kmip_client.request().create_symmetric_key() as builder:
            created = (
                builder.aes_256()
                .usage_mask(
                    protocol_values.CryptographicUsageMask.ENCRYPT
                    | protocol_values.CryptographicUsageMask.DECRYPT
                )
                .name("fluent-expert-attributes")
                .add_attribute(StandardAttribute.OBJECT_GROUP, object_group)
                .execute(limits)
            )
    try:
        create_item = created.items[0]
        assert create_item.operation == protocol_values.Operation.CREATE
        assert create_item.outcome is client.BatchOutcome.COMPLETED
        assert create_item.result_status == 0
        identifier = create_item.unique_identifier
        assert identifier == extensions.UniqueIdentifier.text_string(
            "fluent-expert-attributes"
        )
    finally:
        created.close()

    destroyed = kmip_client.request().destroy(identifier).execute(limits)
    try:
        destroy_item = destroyed.items[0]
        assert destroy_item.operation == protocol_values.Operation.DESTROY
        assert destroy_item.outcome is client.BatchOutcome.COMPLETED
        assert destroy_item.result_status == 0
        assert destroy_item.destroy_response is not None
        assert destroy_item.destroy_response.unique_identifier == identifier
    finally:
        destroyed.close()
    print("python: fluent catalogued expert-attribute journey verified")


def main() -> None:
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
        with ttlv.codec_limits_defaults() as limits:
            results: list[client.BatchResult] = []
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
                    .execute(limits)
                )
                results.append(created)
                create_item = created.items[0]
                assert create_item.operation == 1
                assert create_item.outcome is client.BatchOutcome.COMPLETED
                assert create_item.result_status == 0
                identifier = create_item.unique_identifier
                assert identifier == extensions.UniqueIdentifier.text_string("fluent-key")

                retrieved = kmip_client.request().get(identifier).execute(limits)
                results.append(retrieved)
                get_item = retrieved.items[0]
                assert get_item.operation == 10
                assert get_item.outcome is client.BatchOutcome.COMPLETED
                assert get_item.result_status == 0
                get_response = get_item.get_response()
                assert get_response is not None
                assert get_response.unique_identifier == identifier
                assert get_response.object is not None
                retrieved_identifier = get_response.unique_identifier

                destroyed = kmip_client.request().destroy(retrieved_identifier).execute(limits)
                results.append(destroyed)
                destroy_item = destroyed.items[0]
                assert destroy_item.operation == 20
                assert destroy_item.outcome is client.BatchOutcome.COMPLETED
                assert destroy_item.result_status == 0
                assert destroy_item.destroy_response is not None
                assert destroy_item.destroy_response.unique_identifier == retrieved_identifier
                print("python: fluent Create/Get/Destroy journey verified")
            finally:
                for result in results:
                    result.close()
            execute_aes_gcm_journey(kmip_client, limits)
            execute_key_pair_signature_journey(kmip_client, limits)
            execute_expert_attribute_journey(kmip_client, limits)


if __name__ == "__main__":
    main()
