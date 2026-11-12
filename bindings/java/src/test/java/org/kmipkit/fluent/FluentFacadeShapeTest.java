package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNull;

import java.io.IOException;
import java.net.ServerSocket;
import java.net.SocketTimeoutException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.ClientOperationException;
import org.kmipkit.client.KmipClient;
import org.kmipkit.client.SymmetricKeyRequest;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.StandardAttribute;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.protocol.KmipValues.CryptographicUsageMask;
import org.junit.jupiter.api.Test;

/** Checks the Java package shape while preserving the established extensions API. */
final class FluentFacadeShapeTest {
    @Test
    void keyPairBuilderRequiresExplicitAlgorithmAndLength() {
        // KMIPKIT-0093-FR-011/013; KMIP 2.1 section 6.1.2 Create Key Pair.
        CreateKeyPairBuilder builder = new CreateKeyPairBuilder(null);
        assertThrows(InvalidInputException.class, builder::build);
        builder.algorithm(6);
        assertThrows(InvalidInputException.class, builder::build);
        builder.length(256).privateUsageMask(1).publicUsageMask(2);
        try (ClientBatchItem item = builder.toBatchItem()) {
            assertEquals(2, item.operation());
        }
        assertThrows(InvalidInputException.class, () -> builder.algorithm(-1));
        assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
        assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
    }

    @Test
    void exposesTaskEntryPointAndRetainsLegacyClientType() throws Exception {
        assertNotNull(Class.forName("org.kmipkit.client.KmipClient"));
        assertNotNull(Class.forName("org.kmipkit.client.SymmetricKeyRequest"));
        assertNotNull(Class.forName("org.kmipkit.fluent.SymmetricKeyBuilder"));
        assertNotNull(Class.forName("org.kmipkit.extensions.Client"));
    }

    @Test
    void buildersExposeBuildBatchAndExecutionMethods() throws Exception {
        assertNotNull(SymmetricKeyBuilder.class.getMethod("build"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("toBatchItem"));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("withOptions",
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(SymmetricKeyBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("asynchronousIndicator", long.class));
        assertNotNull(OperationBuilder.class.getMethod("execute",
                org.kmipkit.ttlv.CodecLimits.class));
    }

    @Test
    void kmipClientExposesNamedOperationConveniences() throws Exception {
        Class<?> client = Class.forName("org.kmipkit.client.KmipClient");
        Class<?> request = Class.forName("org.kmipkit.client.SymmetricKeyRequest");
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("createSymmetricKey", request,
                org.kmipkit.ttlv.CodecLimits.class, org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("getById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class));
        assertNotNull(client.getMethod("destroyById",
                org.kmipkit.extensions.UniqueIdentifier.class, org.kmipkit.ttlv.CodecLimits.class,
                org.kmipkit.extensions.RequestOptions.class));
    }

    @Test
    void createBuilderUsesRustValidationAndProducesTypedCreateItem() {
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
            assertUnsentValidation(assertThrows(ClientOperationException.class,
                    () -> builder.aes256().build()), 4);
            assertThrows(InvalidInputException.class, () -> builder.algorithm(-1));
            assertThrows(InvalidInputException.class, () -> builder.name(null));
            assertThrows(InvalidInputException.class, () -> builder.withOptions(null));
            assertThrows(InvalidInputException.class, () -> new OperationBuilder(null, null, false));
        }
        try (SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) (CryptographicUsageMask.ENCRYPT | CryptographicUsageMask.DECRYPT))
                    .name("payments-key");
            try (ClientBatchItem item = builder.toBatchItem()) {
                assertNotNull(item);
                assertEquals(1, item.operation());
            }
            builder.close();
            assertThrows(InvalidInputException.class, builder::build);
        }
    }

    @Test
    void expertPositionValidationIsTypedAndNotSentBeforeAnyConnection() throws Exception {
        // KMIPKIT-0096-FR-003; KMIP v2.1 sections 4-5 and ADR-0012.
        try (ServerSocket listener = new ServerSocket(0);
                ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {fixture("client-cert.pem")}, fixture("client-key.pem"))
                        .trustCertificatesPem(new byte[][] {fixture("server-cert.pem")}).build();
                KmipClient client = KmipClient.create(configuration, transport);
                RawTag raw = RawTag.fromRaw(0x540001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(
                        "position-private-value".getBytes(java.nio.charset.StandardCharsets.UTF_8), CodecLimits.defaults());
                TtlvItem item = TtlvItem.create(tag, value, CodecLimits.defaults())) {
            listener.setSoTimeout(100);
            for (int[] positions : new int[][] {{99}, {0, 0}}) {
                try (SymmetricKeyBuilder builder = client.request().createSymmetricKey()) {
                    builder.aes256().usageMask((int) CryptographicUsageMask.ENCRYPT);
                    for (int position : positions) builder.attributeItemAt(position, item);
                    assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
                    assertUnsentValidation(assertThrows(ClientOperationException.class,
                            () -> builder.execute(CodecLimits.defaults())), 4);
                    assertThrows(SocketTimeoutException.class, listener::accept);
                }
            }
        }
    }

    @Test
    void expertInsertionReportsNativeTypedValidationAndPreservesOtherArgumentErrors() {
        // Catalogued Cryptographic Algorithm requires Enumeration, not TextString.
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag raw = RawTag.fromRaw(0x420028);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.TextString(
                        "position-private-value".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            assertUnsentValidation(assertThrows(ClientOperationException.class,
                    () -> builder.attributeItemAt(0, item)), 13);
            assertThrows(InvalidInputException.class,
                    () -> NativeFluentRegistry.symmetricKeyBuilderInsertAttributeItem(0, -1, 0));
            assertThrows(InvalidInputException.class,
                    () -> org.kmipkit.NativeExtensionRegistry.symmetricKeyBuilderSetAlgorithm(0, 3));
            assertThrows(InvalidInputException.class, () -> builder.attributeItemAt(-1, item));
            assertThrows(InvalidInputException.class, () -> builder.attributeItemAt(0, null));
        }
    }

    private static void assertUnsentValidation(ClientOperationException failure, int status) {
        assertEquals(status, failure.statusCode());
        assertEquals(1, failure.categoryCode());
        assertEquals(1, failure.causeCode());
        assertEquals(1, failure.deliveryStateCode());
        assertNull(failure.getCause());
        assertFalse(failure.toString().contains("position-private-value"));
    }

    @Test
    void builderAcceptsCataloguedUnknownAndVendorAttributes() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag descriptionRaw = RawTag.fromRaw(StandardAttribute.DESCRIPTION.tag());
                Tag descriptionTag = descriptionRaw.checked();
                TtlvValue description = TtlvValue.TextString(
                        "managed without TTLV tags".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                RawTag expertRaw = RawTag.fromRaw(0x540001);
                Tag expertTag = expertRaw.checked();
                TtlvValue expertValue = TtlvValue.TextString(
                        "expert value".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                TtlvItem expertItem = TtlvItem.create(expertTag, expertValue, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) (CryptographicUsageMask.ENCRYPT | CryptographicUsageMask.DECRYPT))
                    .cataloguedAttribute(descriptionTag, description)
                    .attributeItem(expertItem)
                    .vendorAttribute("Vendor_Name.1", "region", expertItem)
                    .withOptions(RequestOptions.defaults())
                    .asynchronousIndicator(1);
            assertThrows(InvalidInputException.class,
                    () -> builder.vendorAttribute(null, "region", expertItem));
            try (ClientBatchItem item = builder.build()) {
                assertEquals(1, item.operation());
            }
            try (TtlvValue wrongValue = TtlvValue.Integer(1)) {
                assertThrows(InvalidInputException.class,
                        () -> builder.cataloguedAttribute(descriptionTag, wrongValue));
            }
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class, () -> builder.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    @Test
    void builderCanInsertAnExpertItemAtAnExplicitFinalAttributePosition() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag raw = RawTag.fromRaw(0x54F001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(17);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) CryptographicUsageMask.ENCRYPT)
                    .attributeItemAt(0, item);
            try (ClientBatchItem batchItem = builder.build()) {
                assertEquals(1, batchItem.operation());
            }
            assertThrows(InvalidInputException.class,
                    () -> builder.attributeItemAt(-1, item));
        }

        try (RawTag raw = RawTag.fromRaw(0x54F001);
                Tag tag = raw.checked();
                TtlvValue value = TtlvValue.Integer(17);
                TtlvItem item = TtlvItem.create(tag, value, limits);
                SymmetricKeyBuilder builder = new SymmetricKeyBuilder(null)) {
            builder.aes256()
                    .usageMask((int) CryptographicUsageMask.ENCRYPT)
                    .attributeItemAt(99, item);
            assertUnsentValidation(assertThrows(ClientOperationException.class, builder::build), 4);
        }
    }

    @Test
    void facadeConveniencesBuildAndExecuteTypedCreateGetAndDestroy() throws Exception {
        int unusedPort;
        try (ServerSocket reservation = new ServerSocket(0)) {
            unusedPort = reservation.getLocalPort();
        }
        byte[] clientCertificate = fixture("client-cert.pem");
        byte[] clientPrivateKey = fixture("client-key.pem");
        byte[] serverCertificate = fixture("server-cert.pem");

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", unusedPort)
                        .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                        .trustCertificatesPem(new byte[][] {serverCertificate})
                        .build();
                KmipClient client = KmipClient.create(configuration, transport)) {
            CodecLimits limits = CodecLimits.defaults();
            UniqueIdentifier id = UniqueIdentifier.textString("fluent-id");
            try (ClientBatchItem create = client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT).build();
                    ClientBatchItem get = client.request().get(id).build();
                    ClientBatchItem destroy = client.request().destroy(id).build();
                    ClientBatchItem getCopy = client.request().get(id).toBatchItem()) {
                assertEquals(1, create.operation());
                assertEquals(10, get.operation());
                assertEquals(20, destroy.operation());
                assertEquals(10, getCopy.operation());
            }
            assertThrows(ClientOperationException.class, () -> client.createSymmetricKey(
                    SymmetricKeyRequest.named(3, 256, (int) CryptographicUsageMask.ENCRYPT, "named"), limits));
            assertThrows(ClientOperationException.class, () -> client.createSymmetricKey(
                    SymmetricKeyRequest.unnamed(3, 256, (int) CryptographicUsageMask.ENCRYPT), limits,
                    RequestOptions.defaults()));
            assertThrows(ClientOperationException.class, () -> client.getById(id, limits));
            assertThrows(ClientOperationException.class,
                    () -> client.getById(id, limits, RequestOptions.defaults()));
            assertThrows(ClientOperationException.class, () -> client.destroyById(id, limits));
            assertThrows(ClientOperationException.class,
                    () -> client.destroyById(id, limits, RequestOptions.defaults()));
            assertThrows(InvalidInputException.class, () -> client.createSymmetricKey(null, limits));
            assertThrows(InvalidInputException.class,
                    () -> client.createSymmetricKey(null, limits, RequestOptions.defaults()));
            assertThrows(InvalidInputException.class, () -> client.getById(null, limits));
            assertThrows(InvalidInputException.class, () -> client.destroyById(id, null));
            assertThrows(InvalidInputException.class, () -> client.destroyById(null, limits));

            assertThrows(ClientOperationException.class,
                    () -> client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().createSymmetricKey().aes256()
                            .usageMask((int) CryptographicUsageMask.ENCRYPT)
                            .withOptions(RequestOptions.defaults()).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().get(id).asynchronousIndicator(1).execute(limits));
            assertThrows(ClientOperationException.class,
                    () -> client.request().destroy(id).withOptions(RequestOptions.defaults())
                            .asynchronousIndicator(1).execute(limits));
            assertThrows(InvalidInputException.class,
                    () -> client.request().get(id).asynchronousIndicator(-1));
            assertThrows(InvalidInputException.class,
                    () -> client.request().destroy(id).asynchronousIndicator(0x1_0000_0000L));

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                assertThrows(ClientOperationException.class, () -> client.execute(batch, limits));
            }
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions());
                assertThrows(ClientOperationException.class,
                        () -> client.executeWithOptions(batch, limits, RequestOptions.defaults()));
            }
        }
    }

    private static byte[] fixture(String filename) throws IOException {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IOException("shared mTLS fixture not found: " + filename);
    }
}
