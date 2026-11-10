package org.kmipkit.fluent;

import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertEquals;

import java.io.IOException;
import java.net.ServerSocket;
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
            assertThrows(InvalidInputException.class, builder::build);
            assertThrows(InvalidInputException.class,
                    () -> builder.aes256().build());
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
    void builderAcceptsCataloguedUnknownAndVendorAttributes() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag descriptionRaw = RawTag.fromRaw(StandardAttribute.DESCRIPTION.tag());
                Tag descriptionTag = descriptionRaw.checked();
                TtlvValue description = TtlvValue.TextString(
                        "managed without TTLV tags".getBytes(java.nio.charset.StandardCharsets.UTF_8), limits);
                RawTag expertRaw = RawTag.fromRaw(0x4200fe);
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
