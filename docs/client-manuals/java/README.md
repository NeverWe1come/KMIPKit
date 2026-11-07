# KMIPKit Java Client Manual

Java 17+ client library for **OASIS KMIP 2.1** with high-level builders, full typed protocol access, and raw TTLV encoding over TLS 1.3 mTLS.

---

## 1. Installation

### Gradle (Kotlin DSL)
```kotlin
dependencies {
    implementation("org.kmipkit:kmipkit-client:1.0.0")
}
```

### Maven
```xml
<dependency>
    <groupId>org.kmipkit</groupId>
    <artifactId>kmipkit-client</artifactId>
    <version>1.0.0</version>
</dependency>
```

The JAR automatically extracts and loads the verified platform-specific native binary (`libkmipkit.so` on Linux, `libkmipkit.dylib` on macOS, `kmipkit.dll` on Windows) at runtime.

---

## 2. Quickstart

```java
import org.kmipkit.client.KmipClient;
import org.kmipkit.config.ClientConfig;
import org.kmipkit.models.SymmetricAlgorithm;
import org.kmipkit.models.CryptographicUsage;

public class Quickstart {
    public static void main(String[] args) {
        var config = ClientConfig.builder()
            .endpoint("kms.corp.internal:5696")
            .caCertificatePath("/etc/kmip/ca.pem")
            .clientCertificatePath("/etc/kmip/client.crt")
            .clientKeyPath("/etc/kmip/client.key")
            .build();

        try (var client = KmipClient.connect(config)) {
            // Discover Versions
            var versions = client.discoverVersions().execute();
            System.out.println("Supported versions: " + versions);

            // Create Key
            String keyId = client.createSymmetricKey()
                .name("app-data-key")
                .algorithm(SymmetricAlgorithm.AES)
                .lengthBits(256)
                .usage(CryptographicUsage.ENCRYPT, CryptographicUsage.DECRYPT)
                .execute();

            // Activate & Encrypt
            client.activate(keyId).execute();
            byte[] ciphertext = client.encrypt(keyId)
                .plaintext("Sensitive banking data".getBytes())
                .execute();

            // Decrypt
            byte[] plaintext = client.decrypt(keyId)
                .ciphertext(ciphertext)
                .execute();

            // Destroy
            client.destroy(keyId).execute();
        }
    }
}
```

---

## 3. Connection and Transport Options

- **Raw TLS (Port 5696)**: Default for enterprise HSMs.
- **HTTPS (Port 5697)**: TTLV over HTTPS/1.1 for corporate proxies.
- **Mutual TLS**: Required. Server certificate validation and hostname checking are mandatory.

---

## 4. Exception Hierarchy

All exceptions extend unchecked `KmipKitException`:
- `KmipResponseException`: Server rejected the operation (includes `ResultStatus` and `ResultReason`).
- `TransportException`: Network failure (includes delivery state: `NOT_SENT`, `POSSIBLY_SENT`, `RESPONSE_STARTED`).
- `ValidationException`: Pre-send invariant violation.

---

## 5. Local Testing

```java
@Test
void testWithLocalServer() {
    try (var server = KmipTestServer.startEphemeral();
         var client = KmipClient.connect(server.getClientConfig())) {
        String keyId = client.createSymmetricKey()
            .lengthBits(256)
            .execute();
        assertNotNull(keyId);
    }
}
```
