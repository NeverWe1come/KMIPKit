# KMIPKit Java Bindings (JNI)

Idiomatic Java 17+ API backed by `libkmipkit_ffi` via a thin C++ JNI bridge (`kmipkit_jni`). All native resources implement `AutoCloseable` for deterministic zeroization via `try-with-resources`.

* **[Installation (Gradle / Maven)](docs/installation.md)**
* **[Native Library Loading & Ownership](docs/native-library.md)**
* **[Testing & Parity Verification](docs/testing.md)**

## Quick Example

```java
import org.kmipkit.extensions.*;
import org.kmipkit.protocol.*;
import java.nio.file.Path;
import java.time.Duration;

try (TransportConfiguration transport = TransportConfiguration.rawTls("kms.internal.example.com", 5696)
        .clientIdentityPemFiles(Path.of("/etc/kmip/client.crt"), Path.of("/etc/kmip/client.key"))
        .trustCaPemFile(Path.of("/etc/kmip/ca.crt"))
        .totalTimeout(Duration.ofSeconds(15))
        .build();
     ClientConfiguration config = ClientConfiguration.defaults().withWireEncoding(WireEncoding.TTLV);
     Client client = Client.create(config, transport)) {

    var created = client.createSymmetricKey()
        .algorithm(KmipValues.CryptographicAlgorithm.AES)
        .length(256)
        .usageMask(KeyUsage.ENCRYPT | KeyUsage.DECRYPT)
        .name("java-app-dek")
        .execute();
    client.activate(created.uniqueIdentifier().orElseThrow()).execute();
}
```
