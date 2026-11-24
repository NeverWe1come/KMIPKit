# KMIPKit Python Bindings (CFFI)

Typed Python 3.12 API backed by `libkmipkit_ffi` via CFFI. Supports context managers (`with ... as ...`) for deterministic native handle release and secret zeroization.

* **[Installation](docs/installation.md)**
* **[Native Library Handling](docs/native-library.md)**
* **[Testing](docs/testing.md)**

## Quick Example

```python
from kmipkit import Client, ClientConfiguration, TransportConfiguration, WireEncoding
from kmipkit.protocol import CryptographicAlgorithm, KeyUsage

transport = (
    TransportConfiguration.raw_tls("kms.internal.example.com", 5696)
    .with_client_identity_pem_files("/etc/kmip/client.crt", "/etc/kmip/client.key")
    .with_trust_ca_pem_file("/etc/kmip/ca.crt")
    .build()
)

with Client(ClientConfiguration.defaults().with_wire_encoding(WireEncoding.TTLV), transport) as client:
    created = (
        client.create_symmetric_key()
        .algorithm(CryptographicAlgorithm.AES)
        .length(256)
        .usage_mask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
        .name("python-service-dek")
        .execute()
    )
    client.activate(created.unique_identifier).execute()
```
