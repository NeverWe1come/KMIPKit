import java.nio.file.Path;
import java.util.Arrays;
import java.util.List;
import org.kmipkit.client.KmipClient;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.fluent.SymmetricKeyBuilder;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;

public final class Consumer {
    private static final byte[] AES_GCM_PLAINTEXT = "KMIPKit AES-GCM journey".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    private static final byte[] AES_GCM_CIPHERTEXT = {
        0x0c, 0x4f, (byte) 0x9f, 0x4b, (byte) 0x8e, (byte) 0x8c, (byte) 0xb6, 0x3b,
        (byte) 0xcc, 0x04, (byte) 0xc4, (byte) 0xa6, (byte) 0xf6, (byte) 0xaa, 0x35, 0x4d,
        (byte) 0xe9, (byte) 0xb9, (byte) 0xf2, 0x46, (byte) 0x9e, 0x1e, 0x26};
    private static final byte[] AES_GCM_IV = {0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11};
    private static final byte[] AES_GCM_AAD = "kmipkit-gcm-aad".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    private static final byte[] AES_GCM_TAG = {
        (byte) 0xb5, 0x16, (byte) 0xad, 0x2a, 0x79, (byte) 0xbf, 0x55, (byte) 0xd1,
        0x47, 0x48, 0x2f, 0x13, (byte) 0xfe, (byte) 0xed, 0x6e, (byte) 0xa6};

    private Consumer() {}

    public static void main(String[] args) {
        int port = Integer.parseInt(System.getenv("KMIPKIT_CONSUMER_TEST_PORT"));
        Path fixtures = Path.of(System.getenv("KMIPKIT_CONSUMER_TEST_FIXTURES"));
        CodecLimits limits = CodecLimits.defaults();
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", port)
                        .clientIdentityFiles(List.of(fixtures.resolve("client-cert.pem")),
                                fixtures.resolve("client-key.pem"),
                                TransportConfiguration.Encoding.PEM)
                        .trustCertificateFiles(List.of(fixtures.resolve("server-cert.pem")),
                                TransportConfiguration.Encoding.PEM)
                        .tlsServerName("localhost")
                        .build();
                KmipClient kmipClient = KmipClient.create(configuration, transport)) {
            UniqueIdentifier identifier;
            try (SymmetricKeyBuilder builder = kmipClient.request().createSymmetricKey();
                ClientBatchResponse created = builder.aes256().usageMask((int) (
                                KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                        .name("fluent-key")
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = created.itemAt(0)) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                    throw new IllegalStateException("Create did not complete successfully");
                }
                identifier = item.createResponse().orElseThrow().uniqueIdentifier();
            }

            UniqueIdentifier retrievedIdentifier;
            try (ClientBatchResponse retrieved = kmipClient.request().get(identifier)
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = retrieved.itemAt(0);
                    var response = item.getResponse().orElseThrow()) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                        || !response.uniqueIdentifier().orElseThrow().text().equals(identifier.text())
                        || response.object().isEmpty()) {
                    throw new IllegalStateException("Get did not return the created object");
                }
                retrievedIdentifier = response.uniqueIdentifier().orElseThrow();
            }

            try (ClientBatchResponse destroyed = kmipClient.request().destroy(retrievedIdentifier)
                            .withOptions(RequestOptions.defaults()).execute(limits);
                    ClientBatchItemResult item = destroyed.itemAt(0)) {
                if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                        || !item.destroyResponse().orElseThrow().uniqueIdentifier()
                                .text().equals(retrievedIdentifier.text())) {
                    throw new IllegalStateException("Destroy did not return the retrieved identifier");
                }
            }
            System.out.println("java: fluent Create/Get/Destroy journey verified");
            executeAesGcmJourney(kmipClient, limits);
        }
    }

    private static void executeAesGcmJourney(KmipClient client, CodecLimits limits) {
        byte[] ciphertext = null;
        byte[] authenticationTag = null;
        UniqueIdentifier identifier;
        try (SymmetricKeyBuilder builder = client.request().createSymmetricKey();
                ClientBatchResponse created = builder.aes256().usageMask((int) (
                                KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                        .name("fluent-aes-gcm-key").execute(limits);
                ClientBatchItemResult item = created.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                throw new IllegalStateException("AES-GCM Create did not complete successfully");
            }
            identifier = item.createResponse().orElseThrow().uniqueIdentifier();
        }

        try (CryptographicParameters.Builder parameterBuilder = CryptographicParameters.builder();
                CryptographicParameters parameters = parameterBuilder
                        .cryptographicAlgorithm(KmipValues.CryptographicAlgorithm.AES)
                        .blockCipherMode(KmipValues.BlockCipherMode.GCM)
                        .ivLength(96).tagLength(128).build();
                ClientBatchResponse encrypted = client.request().encrypt()
                        .uniqueIdentifier(identifier).data(AES_GCM_PLAINTEXT)
                        .cryptographicParameters(parameters).ivCounterNonce(AES_GCM_IV)
                        .authenticatedEncryptionAdditionalData(AES_GCM_AAD).execute(limits);
                ClientBatchItemResult item = encrypted.itemAt(0);
                ClientBatchItemResult.EncryptResponse response = item.encryptResponse().orElseThrow()) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                    || !response.uniqueIdentifier().text().equals(identifier.text())
                    || !Arrays.equals(response.data().orElseThrow(), AES_GCM_CIPHERTEXT)
                    || !Arrays.equals(response.ivCounterNonce().orElseThrow(), AES_GCM_IV)
                    || !Arrays.equals(response.authenticatedEncryptionTag().orElseThrow(), AES_GCM_TAG)) {
                throw new IllegalStateException("Encrypt response did not match the AES-GCM fixture vector");
            }
            ciphertext = response.data().orElseThrow();
            authenticationTag = response.authenticatedEncryptionTag().orElseThrow();
        }

        try (CryptographicParameters.Builder parameterBuilder = CryptographicParameters.builder();
                CryptographicParameters parameters = parameterBuilder
                        .cryptographicAlgorithm(KmipValues.CryptographicAlgorithm.AES)
                        .blockCipherMode(KmipValues.BlockCipherMode.GCM)
                        .ivLength(96).tagLength(128).build();
                ClientBatchResponse decrypted = client.request().decrypt()
                        .uniqueIdentifier(identifier).data(ciphertext)
                        .cryptographicParameters(parameters).ivCounterNonce(AES_GCM_IV)
                        .authenticatedEncryptionAdditionalData(AES_GCM_AAD)
                        .authenticatedEncryptionTag(authenticationTag).execute(limits);
                ClientBatchItemResult item = decrypted.itemAt(0);
                ClientBatchItemResult.DecryptResponse response = item.decryptResponse().orElseThrow()) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                    || !response.uniqueIdentifier().text().equals(identifier.text())
                    || !Arrays.equals(response.data().orElseThrow(), AES_GCM_PLAINTEXT)) {
                throw new IllegalStateException("Decrypt did not return the original plaintext");
            }
        } finally {
            if (ciphertext != null) Arrays.fill(ciphertext, (byte) 0);
            if (authenticationTag != null) Arrays.fill(authenticationTag, (byte) 0);
        }

        try (ClientBatchResponse destroyed = client.request().destroy(identifier).execute(limits);
                ClientBatchItemResult item = destroyed.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                    || !item.destroyResponse().orElseThrow().uniqueIdentifier().text().equals(identifier.text())) {
                throw new IllegalStateException("AES-GCM Destroy did not complete successfully");
            }
        }
        System.out.println("java: fluent AES-GCM journey verified");
    }
}
