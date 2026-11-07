# KMIPKit Python Client Manual

`kmipkit` is a high-performance, strictly typed Python 3.12+ client library for **OASIS KMIP 2.1**. It binds directly to the verified native Rust engine using CFFI with zero runtime compilation required.

---

## 1. Installation

Install via `pip` or `uv`:

```bash
pip install kmipkit
```

Pre-built binary wheels are distributed for:
- **Linux**: `x86_64` and `aarch64` (`manylinux_2_28`)
- **macOS**: `x86_64` and `arm64` (Apple Silicon, macOS 12+)
- **Windows**: `x86_64` (Windows 10 / Server 2019+)

---

## 2. Quickstart

```python
from kmipkit import Client, ClientConfig, TlsConfig
from kmipkit.enums import SymmetricAlgorithm, CryptographicUsage

# 1. Configure mutual TLS
tls = TlsConfig(
    ca_file="/etc/kmip/ca.pem",
    cert_file="/etc/kmip/client.crt",
    key_file="/etc/kmip/client.key",
)
config = ClientConfig(endpoint="kms.internal.corp:5696", tls=tls)

# 2. Execute operations with context manager
with Client.connect(config) as client:
    # Discover versions
    versions = client.discover_versions().execute()
    print(f"Server KMIP versions: {versions}")

    # Create AES-256 Symmetric Key
    key_id = (
        client.create_symmetric_key()
        .name("database-key")
        .algorithm(SymmetricAlgorithm.AES)
        .length_bits(256)
        .usage(CryptographicUsage.ENCRYPT | CryptographicUsage.DECRYPT)
        .execute()
    )
    print(f"Created key: {key_id}")

    # Activate Key
    client.activate(key_id).execute()

    # Encrypt data
    plaintext = b"Confidential customer record"
    ciphertext = client.encrypt(key_id).plaintext(plaintext).execute()

    # Decrypt data
    decrypted = client.decrypt(key_id).ciphertext(ciphertext).execute()
    assert decrypted == plaintext

    # Destroy Key
    client.destroy(key_id).execute()
    print("Key revoked and destroyed.")
```

---

## 3. Connection and Transport Options

```python
from kmipkit import ClientConfig, TransportMode

# Raw TLS (Port 5696)
raw_config = ClientConfig(
    endpoint="hsm.local:5696",
    transport_mode=TransportMode.RAW_TLS,
    tls=tls_config,
)

# HTTPS (Port 5697)
https_config = ClientConfig(
    endpoint="https://vault.local:5697/kmip",
    transport_mode=TransportMode.HTTPS,
    tls=tls_config,
)
```

---

## 4. Error Handling and Exception Hierarchy

All library exceptions inherit from `KmipKitError`:

```python
from kmipkit.errors import KmipKitError, KmipResponseError, TransportError, DeliveryState

try:
    client.encrypt(key_id).plaintext(b"data").execute()
except KmipResponseError as err:
    print(f"Server rejected request: {err.status} - {err.reason}: {err.message}")
except TransportError as err:
    if err.delivery_state == DeliveryState.NOT_SENT:
        print("Network drop before transmission: safe to retry.")
    else:
        print("Network drop in-flight: do not retry blindly.")
except KmipKitError as err:
    print(f"Client exception: {err}")
```

---

## 5. Local Testing

```python
import pytest
from kmipkit_test_server import TestServer
from kmipkit import Client

def test_key_creation():
    with TestServer.start_ephemeral() as server:
        with Client.connect(server.client_config) as client:
            key_id = client.create_symmetric_key().length_bits(256).execute()
            assert key_id is not None
```
