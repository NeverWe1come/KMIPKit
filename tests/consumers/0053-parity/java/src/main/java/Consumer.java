import java.nio.file.Path;
import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.List;
import org.kmipkit.client.KmipClient;
import org.kmipkit.fluent.SymmetricKeyBuilder;
import org.kmipkit.fluent.CreateKeyPairBuilder;
import org.kmipkit.fluent.DecryptBuilder;
import org.kmipkit.fluent.EncryptBuilder;
import org.kmipkit.fluent.SignBuilder;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.credentials.Authentication;
import org.kmipkit.credentials.Credential;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.protocol.KmipValues;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.KeyMaterial;
import org.kmipkit.ttlv.ManagedObject;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TypedAttribute;

public final class Consumer {
    private Consumer() {}

    private static CryptographicParameters aesGcmParameters() {
        try (CryptographicParameters.Builder builder = CryptographicParameters.builder()) {
            return builder.cryptographicAlgorithm(KmipValues.CryptographicAlgorithm.AES)
                    .blockCipherMode(KmipValues.BlockCipherMode.GCM)
                    .ivLength(96)
                    .tagLength(16)
                    .build();
        }
    }

    private static TtlvItem item(int rawTag, TtlvValue value, CodecLimits limits) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, limits);
        }
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        return TtlvStructure.withItem(structure, item(rawTag, value, limits), limits);
    }

    private static TtlvStructure syntheticPingResponse(CodecLimits limits) {
        TtlvStructure version = TtlvStructure.create();
        version = append(version, 0x42006a, TtlvValue.Integer(2), limits);
        version = append(version, 0x42006b, TtlvValue.Integer(1), limits);
        TtlvStructure header = TtlvStructure.create();
        header = append(header, 0x420069, TtlvValue.structure(version, limits), limits);
        header = append(header, 0x420092, TtlvValue.DateTime(1_751_100_123L), limits);
        header = append(header, 0x42000d, TtlvValue.Integer(1), limits);
        TtlvStructure responseItem = TtlvStructure.create();
        responseItem = append(responseItem, 0x42005c, TtlvValue.Enumeration(0x3b), limits);
        responseItem = append(responseItem, 0x42007f, TtlvValue.Enumeration(0), limits);
        responseItem = append(
                responseItem, 0x42007c,
                TtlvValue.structure(TtlvStructure.create(), limits), limits);
        TtlvStructure root = TtlvStructure.create();
        root = append(root, 0x42007a, TtlvValue.structure(header, limits), limits);
        return append(root, 0x42000f, TtlvValue.structure(responseItem, limits), limits);
    }

    private static Authentication demoAuthentication(CodecLimits limits) {
        TtlvStructure credentialValue = TtlvStructure.create();
        credentialValue = append(
                credentialValue, 0x4200bb,
                TtlvValue.TextString("demo-user".getBytes(), limits), limits);
        TtlvStructure credential = TtlvStructure.create();
        credential = append(credential, 0x420024, TtlvValue.Enumeration(1), limits);
        credential = append(
                credential, 0x420025,
                TtlvValue.structure(credentialValue, limits), limits);
        return Authentication.create(List.of(Credential.fromTtlv(credential)));
    }

    public static void main(String[] args) {
        CodecLimits limits = CodecLimits.defaults();
        try (TtlvValue commentValue = TtlvValue.TextString("local example".getBytes(), limits);
                TypedAttribute attribute = TypedAttribute.create(
                        StandardAttribute.CONTACT_INFORMATION, commentValue);
                CryptographicParameters.Builder parameterBuilder = CryptographicParameters.builder();
                CryptographicParameters parameters = parameterBuilder
                        .cryptographicAlgorithm(0x8000_0001L).build()) {
            if (attribute.attribute() != StandardAttribute.CONTACT_INFORMATION
                    || parameters.cryptographicAlgorithm().orElseThrow() != 0x8000_0001L) {
                throw new IllegalStateException("typed package example failed");
            }

            try (TtlvItem dataType = item(0x420059, TtlvValue.Enumeration(1), limits);
                    TtlvItem dataValue = item(0x42005a, TtlvValue.ByteString("demo bytes".getBytes(), limits), limits);
                    ManagedObject managedObject = ManagedObject.opaqueObject(List.of(dataType, dataValue))) {
                if (managedObject.objectType() != 8) throw new IllegalStateException("object type mismatch");
            }

            try (TtlvItem material = item(0x420043, TtlvValue.ByteString("demo key bytes".getBytes(), limits), limits);
                    KeyMaterial keyMaterial = KeyMaterial.fromItem(1, material)) {
                if (keyMaterial.formatRaw() != 1) throw new IllegalStateException("key format mismatch");
            }

            ClientBatchItem ping = ClientBatchItem.ping();
            try (var report = ping.validationReport()) {
                if (!report.isValid()) throw new IllegalStateException("Ping did not validate");
            }
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ping).setAuthentication(demoAuthentication(limits));
            }

            try (ResponseMessage response = ResponseMessage.fromTtlv(syntheticPingResponse(limits));
                    var item = response.batchItemAt(0)) {
                if (response.batchItemCount() != 1 || item.resultStatus().orElseThrow() != 0) {
                    throw new IllegalStateException("synthetic response did not parse");
                }
                try (TtlvStructure responseTtlv = item.ttlv(); var view = responseTtlv.view()) {
                    if (view.itemCount() != 3) throw new IllegalStateException("response TTLV mismatch");
                }
            }

            int port = Integer.parseInt(System.getenv("KMIPKIT_CONSUMER_TEST_PORT"));
            Path fixtures = Path.of(System.getenv("KMIPKIT_CONSUMER_TEST_FIXTURES"));
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
                try (SymmetricKeyBuilder builder = kmipClient.request().createSymmetricKey()
                                .aes256()
                                .usageMask((int) (KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                                .name("fluent-key");
                        ClientBatchResponse created = builder.execute(limits);
                        ClientBatchItemResult itemResult = created.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0) {
                        throw new IllegalStateException("Create did not complete successfully");
                    }
                    identifier = itemResult.createResponse().orElseThrow().uniqueIdentifier();
                }
                try (ClientBatchResponse retrieved = kmipClient.request().get(identifier)
                                .asynchronousIndicator(1).withOptions(RequestOptions.defaults()).execute(limits);
                        ClientBatchItemResult itemResult = retrieved.itemAt(0);
                        var response = itemResult.getResponse().orElseThrow()) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !response.uniqueIdentifier().orElseThrow().text().equals(identifier.text())
                            || response.object().isEmpty()) {
                        throw new IllegalStateException("Get did not return the created object");
                    }
                }

                byte[] plaintext = "fluent plaintext".getBytes(StandardCharsets.UTF_8);
                byte[] nonce = "nonce-12byte".getBytes(StandardCharsets.UTF_8);
                byte[] additionalData = "fluent context".getBytes(StandardCharsets.UTF_8);
                byte[] ciphertext = null;
                byte[] responseNonce = null;
                byte[] authenticationTag = null;
                try {
                    try (CryptographicParameters aesParameters = aesGcmParameters();
                            EncryptBuilder builder = kmipClient.request().encrypt();
                            ClientBatchResponse encrypted = builder.uniqueIdentifier(identifier)
                                    .cryptographicParameters(aesParameters)
                                    .data(plaintext)
                                    .ivCounterNonce(nonce)
                                    .authenticatedEncryptionAdditionalData(additionalData)
                                    .execute(limits);
                            ClientBatchItemResult itemResult = encrypted.itemAt(0);
                            ClientBatchItemResult.EncryptResponse response =
                                    itemResult.encryptResponse().orElseThrow()) {
                        if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                                || !response.uniqueIdentifier().text().equals(identifier.text())) {
                            throw new IllegalStateException("Encrypt did not return a successful typed response");
                        }
                        ciphertext = response.data().orElseThrow();
                        responseNonce = response.ivCounterNonce().orElseThrow();
                        authenticationTag = response.authenticatedEncryptionTag().orElseThrow();
                        if (!Arrays.equals(ciphertext, "fixture ciphertext".getBytes(StandardCharsets.UTF_8))
                                || !Arrays.equals(responseNonce, nonce)
                                || !Arrays.equals(authenticationTag, "fixture-auth-tag".getBytes(StandardCharsets.UTF_8))) {
                            throw new IllegalStateException("Encrypt response did not preserve the AES-GCM fields");
                        }
                    }
                    try (CryptographicParameters decryptParameters = aesGcmParameters();
                            DecryptBuilder builder = kmipClient.request().decrypt();
                            ClientBatchResponse decrypted = builder.uniqueIdentifier(identifier)
                                    .cryptographicParameters(decryptParameters)
                                    .data(ciphertext)
                                    .ivCounterNonce(responseNonce)
                                    .authenticatedEncryptionAdditionalData(additionalData)
                                    .authenticatedEncryptionTag(authenticationTag)
                                    .execute(limits);
                            ClientBatchItemResult itemResult = decrypted.itemAt(0);
                            ClientBatchItemResult.DecryptResponse response =
                                    itemResult.decryptResponse().orElseThrow()) {
                        byte[] recovered = response.data().orElseThrow();
                        try {
                            if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                                    || !response.uniqueIdentifier().text().equals(identifier.text())
                                    || !Arrays.equals(recovered, plaintext)) {
                                throw new IllegalStateException("Decrypt did not recover the fixture plaintext");
                            }
                        } finally {
                            Arrays.fill(recovered, (byte) 0);
                        }
                    }
                } finally {
                    Arrays.fill(plaintext, (byte) 0);
                    Arrays.fill(nonce, (byte) 0);
                    Arrays.fill(additionalData, (byte) 0);
                    if (ciphertext != null) Arrays.fill(ciphertext, (byte) 0);
                    if (responseNonce != null) Arrays.fill(responseNonce, (byte) 0);
                    if (authenticationTag != null) Arrays.fill(authenticationTag, (byte) 0);
                }

                try (ClientBatchResponse destroyed = kmipClient.destroyById(
                                identifier, limits, RequestOptions.defaults());
                        ClientBatchItemResult itemResult = destroyed.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !itemResult.destroyResponse().orElseThrow().uniqueIdentifier()
                                    .text().equals(identifier.text())) {
                        throw new IllegalStateException("Destroy did not return the created identifier");
                    }
                }
                try (ClientBatchResponse pending = kmipClient.request().destroy(identifier)
                                .asynchronousIndicator(1).execute(limits);
                        ClientBatchItemResult itemResult = pending.itemAt(0)) {
                    byte[] correlation = itemResult.pendingCorrelationValue().orElseThrow();
                    try {
                        if (!itemResult.outcomeKind().isPending() || itemResult.resultStatus() != 2
                                || !Arrays.equals(correlation, "destroy-pending-id".getBytes())) {
                            throw new IllegalStateException("Pending outcome or correlation was not preserved");
                        }
                    } finally {
                        Arrays.fill(correlation, (byte) 0);
                    }
                }
                try (ClientBatchResponse failed = kmipClient.request().destroy(identifier)
                                .asynchronousIndicator(1).execute(limits);
                        ClientBatchItemResult itemResult = failed.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 1
                            || itemResult.destroyResponse().isPresent()
                            || !itemResult.resultMessage().orElseThrow().equals("fixture rejected destroy")) {
                        throw new IllegalStateException("Failure outcome was not preserved");
                    }
                }
                System.out.println("java: fluent AES-GCM lifecycle Success/Pending/Failure outcomes verified");

                UniqueIdentifier privateIdentifier;
                UniqueIdentifier publicIdentifier;
                try (CreateKeyPairBuilder builder = kmipClient.request().createKeyPair()
                                .algorithm(KmipValues.CryptographicAlgorithm.ECDSA)
                                .length(256)
                                .privateUsageMask((int) KmipValues.CryptographicUsageMask.SIGN)
                                .publicUsageMask((int) KmipValues.CryptographicUsageMask.VERIFY)
                                .privateName("fluent-private-key")
                                .publicName("fluent-public-key");
                        ClientBatchResponse paired = builder.execute(limits);
                        ClientBatchItemResult itemResult = paired.itemAt(0)) {
                    var response = itemResult.createKeyPairResponse().orElseThrow();
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0) {
                        throw new IllegalStateException("Create Key Pair did not complete successfully");
                    }
                    privateIdentifier = response.privateKeyUniqueIdentifier();
                    publicIdentifier = response.publicKeyUniqueIdentifier();
                    if (!privateIdentifier.text().equals("fluent-private-key")
                            || !publicIdentifier.text().equals("fluent-public-key")) {
                        throw new IllegalStateException("Create Key Pair identifiers did not match the fixture");
                    }
                }

                byte[] signInput = "fluent sign input".getBytes(StandardCharsets.UTF_8);
                try {
                    try (CryptographicParameters signParameters = CryptographicParameters.builder()
                                    .hashingAlgorithm(KmipValues.HashingAlgorithm.SHA_256)
                                    .digitalSignatureAlgorithm(
                                            KmipValues.DigitalSignatureAlgorithm.ECDSA_WITH_SHA256)
                                    .build();
                            SignBuilder builder = kmipClient.request().sign();
                            ClientBatchResponse signed = builder.uniqueIdentifier(privateIdentifier)
                                    .cryptographicParameters(signParameters)
                                    .data(signInput)
                                    .execute(limits);
                            ClientBatchItemResult itemResult = signed.itemAt(0)) {
                        var response = itemResult.signResponse().orElseThrow();
                        try (TtlvStructure responseTtlv = response.responseTtlv()) {
                            byte[] signature = response.signatureData().orElseThrow();
                            try {
                                if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                                        || !response.uniqueIdentifier().text().equals(privateIdentifier.text())
                                        || !Arrays.equals(signature,
                                                "fixture signature".getBytes(StandardCharsets.UTF_8))) {
                                    throw new IllegalStateException("Sign did not preserve typed Signature Data");
                                }
                            } finally {
                                Arrays.fill(signature, (byte) 0);
                            }
                        }
                    }
                } finally {
                    Arrays.fill(signInput, (byte) 0);
                }

                for (UniqueIdentifier keyIdentifier : List.of(privateIdentifier, publicIdentifier)) {
                    try (ClientBatchResponse destroyedKey = kmipClient.request().destroy(keyIdentifier)
                                    .execute(limits);
                            ClientBatchItemResult itemResult = destroyedKey.itemAt(0)) {
                        if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                                || !itemResult.destroyResponse().orElseThrow().uniqueIdentifier()
                                        .text().equals(keyIdentifier.text())) {
                            throw new IllegalStateException("Destroy did not return a key-pair identifier");
                        }
                    }
                }
                System.out.println(
                        "java: fluent ECDSA key pair, Sign, and Destroy verified over TLS 1.3/mTLS");

                UniqueIdentifier expertIdentifier;
                try (RawTag raw = RawTag.fromRaw(0x540001);
                        Tag tag = raw.checked();
                        TtlvValue vendorValue = TtlvValue.TextString(
                                "fluent expert journey".getBytes(StandardCharsets.UTF_8), limits);
                        TtlvItem vendorValueItem = TtlvItem.create(tag, vendorValue, limits);
                        SymmetricKeyBuilder builder = kmipClient.request().createSymmetricKey()
                                .aes256()
                                .usageMask((int) (KmipValues.CryptographicUsageMask.ENCRYPT
                                        | KmipValues.CryptographicUsageMask.DECRYPT))
                                .name("fluent-expert-key");
                        ClientBatchResponse created = builder
                                .vendorAttribute("KMIPKit.Example", "purpose", vendorValueItem)
                                .execute(limits);
                        ClientBatchItemResult itemResult = created.itemAt(0)) {
                    var response = itemResult.createResponse().orElseThrow();
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || response.objectType() != 2
                            || !response.uniqueIdentifier().text().equals("fluent-expert-key")) {
                        throw new IllegalStateException(
                                "expert-attribute Create did not return its typed response");
                    }
                    expertIdentifier = response.uniqueIdentifier();
                }
                try (ClientBatchResponse retrieved = kmipClient.request().get(expertIdentifier).execute(limits);
                        ClientBatchItemResult itemResult = retrieved.itemAt(0);
                        var response = itemResult.getResponse().orElseThrow()) {
                    response.object().orElseThrow();
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !response.uniqueIdentifier().orElseThrow().text()
                                    .equals(expertIdentifier.text())) {
                        throw new IllegalStateException(
                                "expert-attribute Get did not return the typed identifier");
                    }
                }
                try (ClientBatchResponse destroyed = kmipClient.request().destroy(expertIdentifier)
                                .execute(limits);
                        ClientBatchItemResult itemResult = destroyed.itemAt(0)) {
                    if (!itemResult.outcomeKind().isCompleted() || itemResult.resultStatus() != 0
                            || !itemResult.destroyResponse().orElseThrow().uniqueIdentifier()
                                    .text().equals(expertIdentifier.text())) {
                        throw new IllegalStateException(
                                "expert-attribute Destroy did not return its typed identifier");
                    }
                }
                System.out.println(
                        "java: fluent Vendor Attribute Create/Get/Destroy verified over TLS 1.3/mTLS");
            }
        }
    }
}
