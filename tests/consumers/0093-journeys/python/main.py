"""External Python consumer for the real TLS 1.3/mTLS lifecycle journey."""

import os
from pathlib import Path

from kmipkit import client, extensions, protocol_values, ttlv


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


if __name__ == "__main__":
    main()
