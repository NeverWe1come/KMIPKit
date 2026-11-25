# KMIPKit C Bindings (Stable C11 ABI)

The C binding exposes the full KMIPKit 2.0 / 2.1 / 3.0 client surface through opaque handles, fixed-width types, versioned structs (`struct_size`, `abi_version`), and thread-local error snapshots.

* **[Installation & CMake Integration](docs/installation.md)**
* **[Native Library & Memory Ownership](docs/native-library.md)**
* **[Testing & ABI Consumer Verification](docs/testing.md)**

## Quick Example

```c
#include <stdio.h>
#include "kmipkit.h"
#include "kmipkit_fluent.h"

int main(void) {
    kmipkit_transport_config_t *transport = NULL;
    if (kmipkit_transport_config_new_raw_tls(
            "kms.internal.example.com", 5696,
            "/etc/kmip/client.crt", "/etc/kmip/client.key", "/etc/kmip/ca.crt",
            &transport) != KMIPKIT_OK) {
        return 1;
    }

    kmipkit_client_t *client = NULL;
    kmipkit_status_t st = kmipkit_client_new_default(transport, &client);
    kmipkit_transport_config_free(transport);
    if (st != KMIPKIT_OK) return 1;

    kmipkit_discover_versions_response_t *resp = NULL;
    if (kmipkit_client_discover_versions(client, &resp) == KMIPKIT_OK) {
        printf("Discover Versions OK\n");
        kmipkit_discover_versions_response_free(resp);
    }

    kmipkit_client_free(client);
    return 0;
}
```
