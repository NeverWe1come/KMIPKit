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
import org.kmipkit.fluent.CreateKeyPairBuilder;
import org.kmipkit.fluent.SymmetricKeyBuilder;
import org.kmipkit.fluent.SignBuilder;
import org.kmipkit.fluent.SignatureVerifyBuilder;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.TypedAttributeValueTypeMismatchException;

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
    private static final byte[] SIGNATURE_PLAINTEXT = "KMIPKit ECDSA signature journey".getBytes(java.nio.charset.StandardCharsets.UTF_8);
    private static final byte[] SIGNATURE_DATA = {
        0x30, 0x45, 0x02, 0x21, 0x00, (byte) 0xaa, (byte) 0xbb, (byte) 0xcc, (byte) 0xdd,
        (byte) 0xee, (byte) 0xff, 0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, (byte) 0x88,
        (byte) 0x99, 0x02, 0x21, 0x00, 0x33, 0x44, 0x55, 0x66, 0x77, (byte) 0x88, (byte) 0x99,
        (byte) 0xaa, (byte) 0xbb, (byte) 0xcc, (byte) 0xdd, (byte) 0xee, (byte) 0xff, 0x00, 0x11};

    private Consumer() {}

    public static void main(String[] args) throws Exception {
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
            executeKeyPairSignatureJourney(kmipClient, limits);
            executeExpertAttributeJourney(kmipClient, limits);
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

    private static CryptographicParameters ecdsaSha256Parameters() {
        try (CryptographicParameters.Builder builder = CryptographicParameters.builder()) {
            return builder.hashingAlgorithm(KmipValues.HashingAlgorithm.SHA_256)
                    .digitalSignatureAlgorithm(KmipValues.DigitalSignatureAlgorithm.ECDSA_WITH_SHA256)
                    .build();
        }
    }

    private static void executeKeyPairSignatureJourney(KmipClient client, CodecLimits limits) throws Exception {
        UniqueIdentifier privateIdentifier;
        UniqueIdentifier publicIdentifier;
        CreateKeyPairBuilder builder = client.request().createKeyPair()
                    .algorithm(KmipValues.CryptographicAlgorithm.ECDSA).length(256)
                    .privateUsageMask((int) KmipValues.CryptographicUsageMask.SIGN)
                    .publicUsageMask((int) KmipValues.CryptographicUsageMask.VERIFY);
        try (RawTag raw = RawTag.fromRaw(0x420075);
                Tag tag = raw.checked();
                TtlvValue curve = TtlvValue.Enumeration(KmipValues.RecommendedCurve.P_256);
                TtlvItem curveItem = TtlvItem.create(tag, curve, limits);
                TtlvStructure empty = TtlvStructure.create();
                TtlvStructure domain = TtlvStructure.withItem(empty, curveItem, limits);
                TtlvValue domainValue = TtlvValue.structure(domain, limits);
                TypedAttribute domainAttribute = TypedAttribute.create(
                        StandardAttribute.CRYPTOGRAPHIC_DOMAIN_PARAMETERS, domainValue);
                ClientBatchResponse created = builder.commonAttribute(domainAttribute).execute(limits);
                ClientBatchItemResult item = created.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                throw new IllegalStateException("Create Key Pair did not complete successfully");
            }
            ClientBatchItemResult.CreateKeyPairResponse response = item.createKeyPairResponse().orElseThrow();
            privateIdentifier = response.privateKeyUniqueIdentifier();
            publicIdentifier = response.publicKeyUniqueIdentifier();
        }

        byte[] signature;
        try (CryptographicParameters parameters = ecdsaSha256Parameters();
                SignBuilder signBuilder = client.request().sign();
                ClientBatchResponse signed = signBuilder
                        .uniqueIdentifier(privateIdentifier).data(SIGNATURE_PLAINTEXT)
                        .cryptographicParameters(parameters).execute(limits);
                ClientBatchItemResult item = signed.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                throw new IllegalStateException("Sign did not complete successfully");
            }
            ClientBatchItemResult.SignResponse response = item.signResponse().orElseThrow();
            if (!response.uniqueIdentifier().text().equals(privateIdentifier.text())) {
                throw new IllegalStateException("Sign response returned the wrong key identifier");
            }
            try (TtlvStructure ignored = response.responseTtlv()) {
                signature = response.signatureData().orElseThrow();
            }
            if (!Arrays.equals(signature, SIGNATURE_DATA)) {
                throw new IllegalStateException("Sign response did not match the signature fixture");
            }
        }

        try (CryptographicParameters parameters = ecdsaSha256Parameters();
                SignatureVerifyBuilder verifyBuilder = client.request().signatureVerify();
                ClientBatchResponse verified = verifyBuilder
                        .uniqueIdentifier(publicIdentifier).data(SIGNATURE_PLAINTEXT)
                        .signatureData(signature).cryptographicParameters(parameters).execute(limits);
                ClientBatchItemResult item = verified.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                throw new IllegalStateException("Signature Verify did not complete successfully");
            }
            ClientBatchItemResult.SignatureVerifyResponse response = item.signatureVerifyResponse().orElseThrow();
            try (TtlvStructure ignored = response.responseTtlv()) {
                if (!response.uniqueIdentifier().text().equals(publicIdentifier.text())
                        || response.validityIndicatorRaw().orElse(-1) != 1) {
                    throw new IllegalStateException("Signature Verify did not confirm the fixture signature");
                }
            }
        } finally {
            Arrays.fill(signature, (byte) 0);
        }
        System.out.println("java: fluent Create Key Pair/Sign/Signature Verify journey verified");
    }

    private static void executeExpertAttributeJourney(KmipClient client, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(StandardAttribute.OBJECT_GROUP.tag());
                Tag objectGroupTag = raw.checked();
                TtlvValue wrongValue = TtlvValue.Integer(1);
                SymmetricKeyBuilder invalid = client.request().createSymmetricKey()) {
            boolean rejected = false;
            try {
                invalid.aes256().usageMask((int) (
                                KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                        .name("fluent-invalid-expert-attributes")
                        .cataloguedAttribute(objectGroupTag, wrongValue);
            } catch (TypedAttributeValueTypeMismatchException expected) {
                if (expected.getCause() != null
                        || expected.toString().contains("fluent-invalid-expert-attributes")) {
                    throw new IllegalStateException("wrong-type Object Group error is not safely redacted");
                }
                rejected = true;
            }
            if (!rejected) {
                throw new IllegalStateException("wrong-type Object Group was accepted by the fluent builder");
            }
        }

        UniqueIdentifier identifier;
        try (RawTag raw = RawTag.fromRaw(StandardAttribute.OBJECT_GROUP.tag());
                Tag objectGroupTag = raw.checked();
                TtlvValue objectGroup = TtlvValue.TextString(
                        "fluent-expert-group".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                SymmetricKeyBuilder builder = client.request().createSymmetricKey();
                ClientBatchResponse created = builder.aes256().usageMask((int) (
                                KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                        .name("fluent-expert-attributes")
                        .cataloguedAttribute(objectGroupTag, objectGroup)
                        .execute(limits);
                ClientBatchItemResult item = created.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0) {
                throw new IllegalStateException("Expert attribute Create did not complete successfully");
            }
            identifier = item.createResponse().orElseThrow().uniqueIdentifier();
            if (!identifier.text().equals("fluent-expert-attributes")) {
                throw new IllegalStateException("Expert attribute Create returned an unexpected identifier");
            }
        }

        try (ClientBatchResponse destroyed = client.request().destroy(identifier)
                        .withOptions(RequestOptions.defaults()).execute(limits);
                ClientBatchItemResult item = destroyed.itemAt(0)) {
            if (!item.outcomeKind().isCompleted() || item.resultStatus() != 0
                    || !item.destroyResponse().orElseThrow().uniqueIdentifier()
                            .text().equals(identifier.text())) {
                throw new IllegalStateException("Expert attribute Destroy did not preserve its identifier");
            }
        }
        System.out.println("java: fluent catalogued expert-attribute journey verified");
    }
}
