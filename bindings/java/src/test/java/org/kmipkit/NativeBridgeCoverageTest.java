package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;
import java.net.InetAddress;
import java.net.ServerSocket;
import java.net.Socket;
import java.net.SocketTimeoutException;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.security.cert.X509Certificate;
import java.time.Duration;
import java.util.List;
import java.util.Optional;
import java.util.function.Function;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.FutureTask;
import java.util.concurrent.TimeUnit;
import javax.net.ssl.SSLServerSocket;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.function.Executable;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.ClientBatchItemResult;
import org.kmipkit.extensions.ClientBatchResponse;
import org.kmipkit.extensions.AttributeReference;
import org.kmipkit.extensions.ClientConfiguration;
import org.kmipkit.extensions.ClientExtensionRegistry;
import org.kmipkit.extensions.ClientRequestMessageExtension;
import org.kmipkit.extensions.ProfileObjectState;
import org.kmipkit.extensions.ProfileValidationContext;
import org.kmipkit.extensions.ProfileValidationSelection;
import org.kmipkit.extensions.Compatibility;
import org.kmipkit.extensions.Discriminator;
import org.kmipkit.extensions.DuplicateExtensionKeyException;
import org.kmipkit.extensions.ExtensionChildRule;
import org.kmipkit.extensions.ExtensionCompatibilityException;
import org.kmipkit.extensions.ExtensionDefinition;
import org.kmipkit.extensions.ExtensionIdentity;
import org.kmipkit.extensions.ExtensionRegistryLimits;
import org.kmipkit.extensions.ExtensionSchema;
import org.kmipkit.extensions.ProtocolVersion;
import org.kmipkit.extensions.SplitKeyReference;
import org.kmipkit.extensions.TransportConfiguration;
import org.kmipkit.extensions.UniqueIdentifier;
import org.kmipkit.extensions.InvalidExtensionIdentityException;
import org.kmipkit.extensions.RegisteredExtensionValue;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.extensions.TtlvPath;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;

final class NativeBridgeCoverageTest {
    private static final long[] CODEC_LIMITS = {16_777_216, 64, 100_000};
    private static final long[] REGISTRY_LIMITS = {
            256, 16_384, 256, 4_096, 1_048_576, 4_096,
            1_048_576, 256, 16_384, 200_000, 1_048_576, 64
    };
    private static final int CRITICALITY_EXTENSION_TAG = 0x420001;
    private static final CodecLimits DEFAULT_CODEC_LIMITS = CodecLimits.defaults();

    private record ResponseGetter(String name, Function<ClientBatchItemResult, Optional<?>> invoke) {}

    @Test
    void nativeAbiVersionMatchesTheExpectedVersion() {
        NativeExtensionRegistry.ensureLoaded();

        assertEquals(1, NativeExtensionRegistry.nativeAbiVersion());
    }

    @Test
    void nativeStringBridgeRoundTripsUnicodeAndRejectsInvalidJavaText() {
        String text = "two-byte é, three-byte €, four-byte 🔐";
        long identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", text, "1");
        try {
            assertEquals(text, NativeExtensionRegistry.extensionIdentityName(identity));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.IDENTITY, identity);
        }

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", null, "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\ud800", "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\udc00", "1"));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "x".repeat(4_097), "1"));
    }

    @Test
    void nativeArrayReadersRejectNullMalformedAndOverLimitInputs() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(null, CODEC_LIMITS));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(new byte[0], null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {1, 2}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {-1, 2, 3}));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.ttlvValueByteString(new byte[2], new long[] {1, 1, 1}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(null, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(new long[] {0}, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.codecLimitsCreate(-1, 1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.release(-1, 1));
    }

    @Test
    void nativeStatusFailuresBecomeTypedErrorsForNullHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionCount(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionIdentityAt(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemExtensionCriticalityIndicatorAt(0, 0));

        List<Executable> responseGetters = List.of(
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadata(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadataV2(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseUniqueBatchItemId(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseExtensionCount(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseExtensionTtlvAt(0, 0),
                () -> NativeExtensionRegistry.clientBatchItemResponsePendingCorrelationValue(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDiscoverVersionsSupportedVersions(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseCreate(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseCertify(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRecertify(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRekey(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRekeyKeyPair(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseReprovision(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseReprovisionTtlv(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRegister(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRevoke(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRngRetrieve(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRngSeed(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseActivate(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseArchive(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDestroy(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseRecover(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseCheck(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDelegatedLogin(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseLogin(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseLogout(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseObtainLease(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseImport(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseJoinSplitKey(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseGet(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseLocate(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseGetConstraints(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 1, 16),
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributes(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseAddAttribute(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseAdjustAttribute(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDeleteAttribute(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseModifyAttribute(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseSetAttribute(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseSetConstraints(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseSetDefaults(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseValidate(0),
                () -> NativeExtensionRegistry.clientBatchItemResponsePing(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseQueryTtlv(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseInterop(0),
                () -> NativeExtensionRegistry.clientBatchItemResponsePkcs11(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseLog(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseExport(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDeriveKey(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateKeyPair(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseHash(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseMac(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseMacVerify(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseSign(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseSignatureVerify(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseDecrypt(0),
                () -> NativeExtensionRegistry.clientBatchItemResponseEncrypt(0));
        for (Executable responseGetter : responseGetters) {
            assertThrows(InvalidInputException.class, responseGetter);
        }
    }

    @Test
    void nativeBatchItemCriticalityReaderReturnsBothCallerSelectedValues() {
        try (RawTag rawTag = RawTag.fromRaw(CRITICALITY_EXTENSION_TAG);
                Tag tag = rawTag.checked();
                ExtensionIdentity identity = ExtensionIdentity.create("coverage.vendor", "criticality", "1");
                Compatibility compatibility = Compatibility.create(2, 1, 2, 1, "0.0.0", "99.0.0");
                TtlvPath discriminatorPath = TtlvPath.create(tag);
                TtlvValue discriminatorValue = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), DEFAULT_CODEC_LIMITS);
                ExtensionSchema markerSchema = ExtensionSchema.scalar(TtlvItemType.TextString);
                ExtensionChildRule childRule = ExtensionChildRule.required(tag, markerSchema);
                ExtensionSchema schema = ExtensionSchema.structure(List.of(childRule), List.of(), false);
                Discriminator discriminator = Discriminator.create(discriminatorPath, discriminatorValue);
                TtlvStructure payload = markerPayload()) {
            ExtensionDefinition definition = ExtensionDefinition.create(
                    identity, compatibility, discriminator, schema);
            try (definition;
                    ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(definition), ExtensionRegistryLimits.defaults())) {
                for (boolean criticalityIndicator : new boolean[] {false, true}) {
                    try (RegisteredExtensionValue currentValue = registry.validateExtensionValue(
                                    identity, payload, DEFAULT_CODEC_LIMITS);
                            ClientRequestMessageExtension extension = ClientRequestMessageExtension.create(
                                    currentValue, criticalityIndicator);
                            ClientBatchItem baseItem = ClientBatchItem.discoverVersions()) {
                        ClientBatchItem itemWithExtension = baseItem.withExtension(extension);
                        try (itemWithExtension) {
                            assertEquals(criticalityIndicator,
                                    itemWithExtension.extensionCriticalityIndicatorAt(0));
                        }
                    }
                }
            }
        }
    }

    @Test
    void nativeOperationSpecificResponseBridgesMapLiveTypedResponses() throws Exception {
        byte[] serverCertificate = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "server-cert.pem");
        byte[] serverPrivateKey = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "server-key.pem");
        byte[] clientCertificate = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "client-cert.pem");
        byte[] clientPrivateKey = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "client-key.pem");
        X509Certificate parsedClientCertificate = (X509Certificate) invokeTlsFixture(
                "testCertificate", new Class<?>[] {byte[].class}, (Object) clientCertificate);
        SSLServerSocket listener = (SSLServerSocket) invokeTlsFixture(
                "deterministicMtlsListener",
                new Class<?>[] {byte[].class, byte[].class, X509Certificate.class},
                serverCertificate, serverPrivateKey, parsedClientCertificate);
        byte[] discoverVersionsResponse = (byte[]) invokeTlsFixture(
                "discoverVersionsResponseMessage", new Class<?>[] {int.class}, 1);
        byte[] getAttributeListResponse = (byte[]) invokeTlsFixture(
                "getAttributeListResponseMessage", new Class<?>[0]);
        byte[] getAttributeListMinimalResponse = (byte[]) invokeTlsFixture(
                "getAttributeListMinimalResponseMessage", new Class<?>[0]);
        byte[] exportResponse = (byte[]) invokeTlsFixture("exportResponseMessage", new Class<?>[0]);
        byte[] getResponse = (byte[]) invokeTlsFixture("getResponseMessage", new Class<?>[0]);
        byte[] joinSplitKeyResponse = (byte[]) invokeTlsFixture(
                "joinSplitKeyResponseMessage", new Class<?>[0]);
        byte[] getAttributesResponse = (byte[]) invokeTlsFixture("getAttributesResponseMessage", new Class<?>[0]);
        byte[] locateResponse = (byte[]) invokeTlsFixture("locateResponseMessage",
                new Class<?>[] {int.class}, 2);
        byte[] recertifyResponse = (byte[]) invokeTlsFixture("recertifyResponseMessage", new Class<?>[0]);
        byte[] obtainLeaseResponse = (byte[]) invokeTlsFixture("obtainLeaseResponseMessage", new Class<?>[0]);
        byte[] rekeyResponse = (byte[]) invokeTlsFixture("rekeyResponseMessage", new Class<?>[0]);
        FutureTask<?> peer = new FutureTask<>(() -> invokeTlsFixture(
                "serveResponses", new Class<?>[] {SSLServerSocket.class, byte[][].class},
                listener, (Object) new byte[][] {discoverVersionsResponse, getAttributeListResponse,
                        getAttributeListMinimalResponse, exportResponse, getResponse,
                        joinSplitKeyResponse, getAttributesResponse, locateResponse,
                        recertifyResponse, obtainLeaseResponse, rekeyResponse}));
        Thread peerThread = new Thread(peer, "kmipkit-native-response-mapper-fixture");
        peerThread.setDaemon(true);
        peerThread.start();

        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                        List.of(), ExtensionRegistryLimits.defaults());
                ClientConfiguration configuration = ClientConfiguration.create(registry);
                TransportConfiguration transport = TransportConfiguration.builder()
                        .rawTls("127.0.0.1", listener.getLocalPort())
                        .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                        .trustCertificatesPem(new byte[][] {serverCertificate})
                        .timeouts(TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(5)),
                                TransportConfiguration.Timeout.bounded(Duration.ofSeconds(8)))
                        .build();
                Client client = Client.create(configuration, transport)) {
            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.discoverVersions()
                        .withUniqueBatchItemId((byte[]) invokeTlsFixture(
                                "uniqueBatchItemId", new Class<?>[] {int.class}, 0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertEquals(List.of(new ProtocolVersion(2, 1)), result.supportedVersions().orElseThrow());
                    assertResponseMappers(result, "supportedVersions");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.getAttributeList(UniqueIdentifier.textString("attribute-list-object"))
                        .withUniqueBatchItemId((byte[]) invokeTlsFixture(
                                "uniqueBatchItemId", new Class<?>[] {int.class}, 0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0);
                        ClientBatchItemResult.GetAttributeListResponse typed =
                                result.getAttributeListResponse().orElseThrow()) {
                    assertEquals("attribute-list-object", typed.uniqueIdentifier().text());
                    assertEquals(2, typed.attributeReferences().size());
                    assertResponseMappers(result, "getAttributeListResponse");
                    long nativeItem = nativeHandle(result);
                    assertThrows(ResourceLimitException.class,
                            () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(
                                    nativeItem, 1, 16));
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.getAttributeList(
                        UniqueIdentifier.textString("minimal-attribute-list-object"))
                        .withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0);
                        ClientBatchItemResult.GetAttributeListResponse typed =
                                result.getAttributeListResponse().orElseThrow()) {
                    assertEquals("minimal-attribute-list-object", typed.uniqueIdentifier().text());
                    assertEquals(List.of(AttributeReference.tag(0x42002f)),
                            typed.attributeReferences());
                    assertResponseMappers(result, "getAttributeListResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.export(UniqueIdentifier.textString("export-fixture"),
                        null, null, null, null)
                        .withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0);
                        ClientBatchItemResult.ExportResponse typed = result.exportResponse().orElseThrow()) {
                    assertEquals(7, typed.objectTypeRaw().orElseThrow());
                    assertEquals("export-fixture", typed.uniqueIdentifier().orElseThrow().text());
                    assertTrue(typed.attributes().isPresent());
                    assertTrue(typed.anyObject().isPresent());
                    assertResponseMappers(result, "exportResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.get(UniqueIdentifier.integer(-17),
                        null, null, null, null)
                        .withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0);
                        ClientBatchItemResult.GetResponse typed = result.getResponse().orElseThrow();
                        TtlvStructureView object = typed.object().orElseThrow().view()) {
                    assertEquals(2, typed.objectTypeRaw().orElseThrow());
                    assertEquals(UniqueIdentifier.Kind.INTEGER,
                            typed.uniqueIdentifier().orElseThrow().kind());
                    assertEquals(-17, typed.uniqueIdentifier().orElseThrow().number());
                    assertEquals(1, object.itemCount());
                    assertResponseMappers(result, "getResponse");
                }
            }

            try (SplitKeyReference reference = new SplitKeyReference(
                            UniqueIdentifier.textString("source-fragment"), 1);
                    ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.joinSplitKey(4, List.of(reference))
                        .withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertResponseMappers(result, "joinSplitKeyResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.getAttributes(UniqueIdentifier.textString("object-7"),
                        List.of(AttributeReference.tag(0x42002f))));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertResponseMappers(result, "getAttributesResponse");
                }
            }

            try (TtlvStructure attributes = TtlvStructure.create();
                    ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.locate(null, null, null, null, attributes));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertResponseMappers(result, "locateResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.recertify(UniqueIdentifier.textString("certificate-to-recertify"),
                        null, null, null, null, null, null).withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertResponseMappers(result, "recertifyResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.add(ClientBatchItem.obtainLease(UniqueIdentifier.textString("lease-request-id"))
                        .withUniqueBatchItemId(batchItemId(0)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult result = response.itemAt(0)) {
                    assertResponseMappers(result, "obtainLeaseResponse");
                }
            }

            try (ClientBatch batch = ClientBatch.create()) {
                batch.asynchronousIndicator(1)
                        .add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-success"),
                                null, null, null).withUniqueBatchItemId(batchItemId(0)))
                        .add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-failure"),
                                null, null, null).withUniqueBatchItemId(batchItemId(1)))
                        .add(ClientBatchItem.rekey(UniqueIdentifier.textString("existing-pending"),
                                null, null, null).withUniqueBatchItemId(batchItemId(2)));
                try (ClientBatchResponse response = client.execute(batch, DEFAULT_CODEC_LIMITS);
                        ClientBatchItemResult success = response.itemAt(0);
                        ClientBatchItemResult failure = response.itemAt(1);
                        ClientBatchItemResult pending = response.itemAt(2)) {
                    assertResponseMappers(success, "rekeyResponse");
                    assertResponseMappers(failure, null);
                    assertResponseMappers(pending, null);
                }
            }
            assertEquals(11, ((List<?>) peer.get(12, TimeUnit.SECONDS)).size());
        } finally {
            listener.close();
        }
    }

    @Test
    void clientExecuteWithProfilesRejectsMalformedTextBeforeOpeningTransport() throws Exception {
        byte[] clientCertificate = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "client-cert.pem");
        byte[] clientPrivateKey = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "client-key.pem");
        byte[] serverCertificate = (byte[]) invokeTlsFixture(
                "testFixture", new Class<?>[] {String.class}, "server-cert.pem");
        ProfileValidationSelection surrogateSelection =
                new ProfileValidationSelection(List.of("\ud800"));
        ProfileValidationSelection validSelection =
                new ProfileValidationSelection(List.of("KMIPKIT-PROFILE-BASELINE-CLIENT"));
        ProfileValidationContext surrogateContext = new ProfileValidationContext().withObjectState(
                0, new ProfileObjectState("\ud800"));
        ProfileValidationContext oversizedContext = new ProfileValidationContext().withObjectState(
                0, new ProfileObjectState("x".repeat(4_097)));

        try (ServerSocket listener = new ServerSocket(0, 1, InetAddress.getByName("127.0.0.1"))) {
            listener.setSoTimeout(1_000);
            FutureTask<Boolean> accepted = new FutureTask<>(() -> {
                try (Socket ignored = listener.accept()) {
                    return true;
                } catch (SocketTimeoutException timeout) {
                    return false;
                }
            });
            Thread observer = new Thread(accepted, "kmipkit-profile-jni-no-connect-observer");
            observer.setDaemon(true);
            observer.start();

            try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                            List.of(), ExtensionRegistryLimits.defaults());
                    ClientConfiguration configuration = ClientConfiguration.create(registry);
                    TransportConfiguration transport = TransportConfiguration.builder()
                            .rawTls("127.0.0.1", listener.getLocalPort())
                            .clientIdentityPem(new byte[][] {clientCertificate}, clientPrivateKey)
                            .trustCertificatesPem(new byte[][] {serverCertificate})
                            .build();
                    Client client = Client.create(configuration, transport);
                    ClientBatch selectionBatch = ClientBatch.create();
                    ClientBatch surrogateContextBatch = ClientBatch.create();
                    ClientBatch oversizedContextBatch = ClientBatch.create()) {
                selectionBatch.add(ClientBatchItem.discoverVersions());
                surrogateContextBatch.add(ClientBatchItem.discoverVersions());
                oversizedContextBatch.add(ClientBatchItem.discoverVersions());

                assertThrows(InvalidInputException.class,
                        () -> client.executeWithProfiles(selectionBatch, DEFAULT_CODEC_LIMITS,
                                surrogateSelection));
                assertThrows(InvalidInputException.class,
                        () -> client.executeWithProfilesAndContext(surrogateContextBatch,
                                DEFAULT_CODEC_LIMITS, validSelection, surrogateContext));
                assertThrows(InvalidInputException.class,
                        () -> client.executeWithProfilesAndContext(oversizedContextBatch,
                                DEFAULT_CODEC_LIMITS, validSelection, oversizedContext));
                assertFalse(accepted.get(2, TimeUnit.SECONDS),
                        "invalid profile text must be rejected before opening the transport");
            }
        }
    }

    private static Object invokeTlsFixture(String name, Class<?>[] parameterTypes, Object... arguments)
            throws Exception {
        Method method = ClientFacadeTlsIntegrationTest.class.getDeclaredMethod(name, parameterTypes);
        if (!method.trySetAccessible()) {
            throw new IllegalStateException("TLS test fixture is inaccessible: " + name);
        }
        try {
            return method.invoke(null, arguments);
        } catch (InvocationTargetException exception) {
            Throwable cause = exception.getCause();
            if (cause instanceof Exception checked) throw checked;
            if (cause instanceof Error error) throw error;
            throw new IllegalStateException("TLS test fixture failed: " + name, cause);
        }
    }

    private static long nativeHandle(ClientBatchItemResult result) throws Exception {
        Field field = ClientBatchItemResult.class.getDeclaredField("handle");
        if (!field.trySetAccessible()) {
            throw new IllegalStateException("client batch item handle is inaccessible");
        }
        NativeHandle handle = (NativeHandle) field.get(result);
        return handle.withValue(value -> value);
    }

    private static byte[] batchItemId(int index) throws Exception {
        return (byte[]) invokeTlsFixture("uniqueBatchItemId", new Class<?>[] {int.class}, index);
    }

    private static void assertResponseMappers(ClientBatchItemResult result, String expectedGetter)
            throws Exception {
        for (ResponseGetter getter : responseGetters()) {
            Optional<?> mapped = getter.invoke().apply(result);
            if (mapped.isPresent()) {
                assertEquals(expectedGetter, getter.name(), "unexpected response mapping through " + getter.name());
            }
            if (getter.name().equals(expectedGetter)) {
                assertTrue(mapped.isPresent(), "response must map through " + getter.name());
            }
            closeMappedResponse(mapped);
        }
    }

    private static List<ResponseGetter> responseGetters() {
        return List.of(
                new ResponseGetter("supportedVersions", result -> result.supportedVersions()),
                new ResponseGetter("createResponse", result -> result.createResponse()),
                new ResponseGetter("certifyResponse", result -> result.certifyResponse()),
                new ResponseGetter("recertifyResponse", result -> result.recertifyResponse()),
                new ResponseGetter("rekeyResponse", result -> result.rekeyResponse()),
                new ResponseGetter("rekeyKeyPairResponse", result -> result.rekeyKeyPairResponse()),
                new ResponseGetter("reprovisionResponse", result -> result.reprovisionResponse()),
                new ResponseGetter("registerResponse", result -> result.registerResponse()),
                new ResponseGetter("activateResponse", result -> result.activateResponse()),
                new ResponseGetter("archiveResponse", result -> result.archiveResponse()),
                new ResponseGetter("destroyResponse", result -> result.destroyResponse()),
                new ResponseGetter("recoverResponse", result -> result.recoverResponse()),
                new ResponseGetter("revokeResponse", result -> result.revokeResponse()),
                new ResponseGetter("rngRetrieveResponse", result -> result.rngRetrieveResponse()),
                new ResponseGetter("rngSeedResponse", result -> result.rngSeedResponse()),
                new ResponseGetter("checkResponse", result -> result.checkResponse()),
                new ResponseGetter("delegatedLoginResponse", result -> result.delegatedLoginResponse()),
                new ResponseGetter("loginResponse", result -> result.loginResponse()),
                new ResponseGetter("pingResponse", result -> result.pingResponse()),
                new ResponseGetter("logoutResponse", result -> result.logoutResponse()),
                new ResponseGetter("queryResponse", result -> result.queryResponse()),
                new ResponseGetter("getUsageAllocationResponse", result -> result.getUsageAllocationResponse()),
                new ResponseGetter("getAttributeListResponse", result -> result.getAttributeListResponse()),
                new ResponseGetter("getAttributesResponse", result -> result.getAttributesResponse()),
                new ResponseGetter("addAttributeResponse", result -> result.addAttributeResponse()),
                new ResponseGetter("adjustAttributeResponse", result -> result.adjustAttributeResponse()),
                new ResponseGetter("deleteAttributeResponse", result -> result.deleteAttributeResponse()),
                new ResponseGetter("modifyAttributeResponse", result -> result.modifyAttributeResponse()),
                new ResponseGetter("setAttributeResponse", result -> result.setAttributeResponse()),
                new ResponseGetter("obtainLeaseResponse", result -> result.obtainLeaseResponse()),
                new ResponseGetter("importResponse", result -> result.importResponse()),
                new ResponseGetter("joinSplitKeyResponse", result -> result.joinSplitKeyResponse()),
                new ResponseGetter("getResponse", result -> result.getResponse()),
                new ResponseGetter("locateResponse", result -> result.locateResponse()),
                new ResponseGetter("getConstraintsResponse", result -> result.getConstraintsResponse()),
                new ResponseGetter("setConstraintsResponse", result -> result.setConstraintsResponse()),
                new ResponseGetter("setDefaultsResponse", result -> result.setDefaultsResponse()),
                new ResponseGetter("validateResponse", result -> result.validateResponse()),
                new ResponseGetter("interopResponse", result -> result.interopResponse()),
                new ResponseGetter("pkcs11Response", result -> result.pkcs11Response()),
                new ResponseGetter("logResponse", result -> result.logResponse()),
                new ResponseGetter("exportResponse", result -> result.exportResponse()),
                new ResponseGetter("deriveKeyResponse", result -> result.deriveKeyResponse()),
                new ResponseGetter("createKeyPairResponse", result -> result.createKeyPairResponse()),
                new ResponseGetter("createSplitKeyUniqueIdentifiers", result -> result.createSplitKeyUniqueIdentifiers()),
                new ResponseGetter("encryptResponse", result -> result.encryptResponse()),
                new ResponseGetter("decryptResponse", result -> result.decryptResponse()),
                new ResponseGetter("hashResponse", result -> result.hashResponse()),
                new ResponseGetter("macResponse", result -> result.macResponse()),
                new ResponseGetter("macVerifyResponse", result -> result.macVerifyResponse()),
                new ResponseGetter("signResponse", result -> result.signResponse()),
                new ResponseGetter("signatureVerifyResponse", result -> result.signatureVerifyResponse()));
    }

    private static void closeMappedResponse(Optional<?> mapped) throws Exception {
        if (mapped.isPresent() && mapped.get() instanceof AutoCloseable closeable) {
            closeable.close();
        }
    }

    private static TtlvStructure markerPayload() {
        try (RawTag rawTag = RawTag.fromRaw(CRITICALITY_EXTENSION_TAG);
                Tag tag = rawTag.checked();
                TtlvStructure empty = TtlvStructure.create();
                TtlvValue value = TtlvValue.TextString(
                        "marker".getBytes(StandardCharsets.UTF_8), DEFAULT_CODEC_LIMITS);
                TtlvItem item = TtlvItem.create(tag, value, DEFAULT_CODEC_LIMITS)) {
            return TtlvStructure.withItem(empty, item, DEFAULT_CODEC_LIMITS);
        }
    }

    @Test
    void nativeTypedResponseBridgesRejectNullItemHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMetadataV2(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreate(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCertify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCheck(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDelegatedLogin(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetUsageAllocation(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetConstraints(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseExport(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, -1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 1, -1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseGetAttributeList(0, 1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDeriveKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateKeyPair(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseHash(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMac(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseMacVerify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSign(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseSignatureVerify(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseDiscoverVersionsSupportedVersions(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseJoinSplitKey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvStructureView(0));
    }

    @Test
    void nativeResponseBridgesRejectNullHandlesForRemainingOperations() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseRekey(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponsePing(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseQueryTtlv(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponsePkcs11(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseCreateSplitKey(0));
    }

    @Test
    void usageAllocationContextBridgesRejectNullHandles() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextUsageLimitsUnit(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextAllocatedUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextAvailableUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextReservedUnits(0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextUniqueIdentifier(0));
    }

    @Test
    void nativeQueryBridgesRejectMalformedObjectGroupText() {
        byte[][] malformedObjectGroups = {new byte[] {(byte) 0xff}};
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(new int[] {1}, malformedObjectGroups));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, malformedObjectGroups, CODEC_LIMITS));
    }

    @Test
    void nativeQueryWithLimitsEnforcesElementBudgetBeforeAllocation() {
        long[] limitsBelowMinimumQuerySize = {16_777_216, 64, 1};
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, null, limitsBelowMinimumQuerySize));
    }

    @Test
    void nativeValidationBridgesRejectMalformedBoundaryInputs() {
        byte[][] nullObjectGroups = new byte[1][];
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQuery(new int[] {1}, nullObjectGroups));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemQueryWithLimits(
                        new int[] {1}, nullObjectGroups, CODEC_LIMITS));

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.splitKeyReferenceCreate(0, null, 0, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        -1, new long[] {1}, false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        1, new long[0], false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemJoinSplitKey(
                        1, new long[] {0}, false, 0, 0, 0));

        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvValueEnumeration(-1));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.ttlvValueInterval(-1));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientCreate(0, 0));
        assertThrows(InvalidInputException.class, () -> NativeExtensionRegistry.clientBatchAddItem(0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchResponseItemAt(0, -1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.usageAllocationContextCreate(0, 1, 0));
    }

    @Test
    void nativeAttributeRequestBridgesRejectMalformedReferenceArrays() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[] {-1},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {0},
                        new byte[][] {null}, new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {99}, new long[] {0},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {1}, new long[] {0x1_0000_0000L},
                        new byte[][] {null}, new byte[][] {null}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {1},
                        new byte[][] {"KMIPKit".getBytes(StandardCharsets.UTF_8)},
                        new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, new int[] {2}, new long[] {0},
                        new byte[][] {new byte[] {(byte) 0xff}},
                        new byte[][] {"Comment".getBytes(StandardCharsets.UTF_8)}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        99, null, 0, new int[0], new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, null, new long[0], new byte[0][], new byte[0][]));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        2, null, 1, new int[0], new long[0], new byte[0][], new byte[0][]));
        int[] tooManyKinds = new int[100_001];
        long[] tooManyTags = new long[tooManyKinds.length];
        byte[][] tooManyVendors = new byte[tooManyKinds.length][];
        byte[][] tooManyNames = new byte[tooManyKinds.length][];
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetAttributes(
                        0, null, 0, tooManyKinds, tooManyTags, tooManyVendors, tooManyNames));
    }

    @Test
    void nativeAttributeRequestCreateRejectsInvalidOperationsAndFields() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        99, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 99, null, 0, 0, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        1, 0, null, 0, 0, 0, 0, 0, null, null, 0x1_0000_0000L, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        0, 0, null, 0, 0, 1, 0, 0, null, null, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        1, 0, null, 0, 0, 0, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        2, 0, null, 0, 0, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        3, 0, null, 0, 0, 1, 1, 1, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        4, 0, null, 0, 1, 1, 0, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.attributeRequestCreate(
                        2, 2, null, 1, 0, 0, 0, 0, null, null, 0, 0));
    }

    @Test
    void nativeNamedAttributeAdjustAndDeleteReachTheirAbiValidationPaths() {
        NativeExtensionRegistry.ensureLoaded();
        byte[] vendor = "KMIPKit".getBytes(StandardCharsets.UTF_8);
        byte[] name = "Comment".getBytes(StandardCharsets.UTF_8);

        long adjusted = NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                0, null, 0, 2, 0, vendor, name, 1, 0);
        assertNotEquals(0, adjusted);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, adjusted);

        long deleted = NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                0, null, 0, 0, 2, 0, vendor, name);
        assertNotEquals(0, deleted);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, deleted);

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 99, 0, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, -1, null, null, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, 0, null, null, 0x1_0000_0000L, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 1, 0, null, name, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        99, null, 0, 2, 0, vendor, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        2, null, 1, 2, 0, vendor, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemAdjustAttribute(
                        0, null, 0, 2, 0, new byte[] {(byte) 0xff}, name, 1, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        0, null, 0, 0, 99, 0, null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        0, null, 0, 0, 1, 0, vendor, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        2, null, 1, 0, 0, 0, null, null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                        99, null, 0, 0, 2, 0, vendor, name));

        long deletedTag = NativeExtensionRegistry.clientBatchItemDeleteAttribute(
                0, null, 0, 0, 1, 0x0042_000a, null, null);
        assertNotEquals(0, deletedTag);
        NativeExtensionRegistry.release(NativeExtensionRegistry.BATCH_ITEM, deletedTag);
    }

    @Test
    void nativePkcs11BridgeRejectsMissingTtlvInputs() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemPkcs11(0, 0, null, 0));
    }

    @Test
    void nativeRekeyBridgeRejectsInvalidIdentifiersAndOffsets() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRekey(99, null, 0, false, 0, 0, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRekey(
                        1, new byte[] {'k'}, 0, true, -1, 0, 0));
    }

    @Test
    void nativeClientOperationBridgesRejectMalformedIdentifiersAndAbsentValues() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCheck(
                        4, null, 0, false, 0, false, 0, false, 0));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCheck(
                        0, null, 0, false, 0, false, 0, false, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemGetUsageAllocation(4, null, 0, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemCreateSplitKey(
                        1, 2, 1, 1, 3, null, (long) Integer.MAX_VALUE + 1,
                        null, 0, 0));
    }

    @Test
    void nativeReleaseIgnoresNullAndForgedPositiveJniHandles() {
        assertDoesNotThrow(() -> NativeExtensionRegistry.release(NativeExtensionRegistry.TAG, 0));
        assertDoesNotThrow(() -> NativeExtensionRegistry.release(NativeExtensionRegistry.TAG, 0x1000L));
    }

    @Test
    void nativeErrorClearResetsOnlyCallingThreadSnapshot() throws Exception {
        RuntimeMetadata.clearLastError();
        ExecutorService worker = Executors.newSingleThreadExecutor();
        try {
            assertThrows(ResourceLimitException.class,
                    () -> RuntimeMetadata.capabilityIsAvailable("x".repeat(129)));
            RuntimeMetadata.ErrorSnapshot callerError = new RuntimeMetadata.ErrorSnapshot(6, 1, 10, 0);
            assertEquals(callerError, RuntimeMetadata.lastErrorSnapshot());

            worker.submit(() -> {
                assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
                assertThrows(InvalidInputException.class,
                        () -> NativeExtensionRegistry.clientBatchItemExtensionCount(0));
                assertEquals(new RuntimeMetadata.ErrorSnapshot(4, 1, 1, 0), RuntimeMetadata.lastErrorSnapshot());

                RuntimeMetadata.clearLastError();
                assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
            }).get(10, TimeUnit.SECONDS);

            assertEquals(callerError, RuntimeMetadata.lastErrorSnapshot());
            RuntimeMetadata.clearLastError();
            assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
        } finally {
            worker.shutdownNow();
            RuntimeMetadata.clearLastError();
        }
    }

    @Test
    void nativeRegistryLookupsAndOptionalInformationUseTypedResults() {
        long identity = 0;
        long compatibility = 0;
        long rootRawTag = 0;
        long rootTag = 0;
        long childRawTag = 0;
        long childTag = 0;
        long rootPath = 0;
        long path = 0;
        long scalar = 0;
        long discriminator = 0;
        long scalarSchema = 0;
        long childRule = 0;
        long childSchema = 0;
        long rootRule = 0;
        long schema = 0;
        long definition = 0;
        long registry = 0;
        long definitionAt = 0;
        long definitionForIdentity = 0;
        long codecLimits = 0;
        try {
            identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "registry", "1");
            compatibility = NativeExtensionRegistry.compatibilityCreate(
                    2, 1, 2, 1, "0.0.0", "99.0.0");
            rootRawTag = NativeExtensionRegistry.rawTagCreate(0x420001);
            rootTag = NativeExtensionRegistry.rawTagChecked(rootRawTag);
            childRawTag = NativeExtensionRegistry.rawTagCreate(0x420002);
            childTag = NativeExtensionRegistry.rawTagChecked(childRawTag);
            rootPath = NativeExtensionRegistry.ttlvPathCreate(rootTag);
            path = NativeExtensionRegistry.ttlvPathWithChildTag(rootPath, childTag);
            scalar = NativeExtensionRegistry.ttlvValueTextString(
                    "marker".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
            discriminator = NativeExtensionRegistry.discriminatorCreate(path, scalar);
            scalarSchema = NativeExtensionRegistry.extensionSchemaScalar(TtlvItemType.TextString.code());
            childRule = NativeExtensionRegistry.extensionChildRuleRequired(childTag, scalarSchema);
            childSchema = NativeExtensionRegistry.extensionSchemaStructure(
                    new long[] {childRule}, new long[0], false);
            rootRule = NativeExtensionRegistry.extensionChildRuleRequired(rootTag, childSchema);
            schema = NativeExtensionRegistry.extensionSchemaStructure(new long[] {rootRule}, new long[0], false);
            definition = NativeExtensionRegistry.extensionDefinitionCreate(
                    identity, compatibility, discriminator, schema);
            registry = NativeExtensionRegistry.clientExtensionRegistryCreate(
                    new long[] {definition}, REGISTRY_LIMITS);

            definitionAt = NativeExtensionRegistry.clientExtensionRegistryDefinitionAt(registry, 0);
            definitionForIdentity = NativeExtensionRegistry.clientExtensionRegistryDefinitionForIdentity(
                    registry, identity);
            assertNotEquals(0, definitionAt);
            assertNotEquals(0, definitionForIdentity);
            assertEquals(0, NativeExtensionRegistry.extensionDefinitionInformation(definition));
            codecLimits = NativeExtensionRegistry.codecLimitsCreate(1_024, 10, 100);
            assertNotEquals(0, codecLimits);

            long duplicateDefinition = definition;
            assertThrows(DuplicateExtensionKeyException.class,
                    () -> NativeExtensionRegistry.clientExtensionRegistryCreate(
                            new long[] {duplicateDefinition, duplicateDefinition}, REGISTRY_LIMITS));
            assertThrows(ExtensionCompatibilityException.class,
                    () -> NativeExtensionRegistry.compatibilityCreate(
                            2, 2, 2, 1, "0.0.0", "99.0.0"));
            assertThrows(InvalidExtensionIdentityException.class,
                    () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "", "1"));
        } finally {
            release(26, codecLimits);
            release(NativeExtensionRegistry.DEFINITION, definitionForIdentity);
            release(NativeExtensionRegistry.DEFINITION, definitionAt);
            release(NativeExtensionRegistry.REGISTRY, registry);
            release(NativeExtensionRegistry.DEFINITION, definition);
            release(NativeExtensionRegistry.SCHEMA, schema);
            release(NativeExtensionRegistry.CHILD_RULE, rootRule);
            release(NativeExtensionRegistry.SCHEMA, childSchema);
            release(NativeExtensionRegistry.CHILD_RULE, childRule);
            release(NativeExtensionRegistry.SCHEMA, scalarSchema);
            release(NativeExtensionRegistry.DISCRIMINATOR, discriminator);
            release(NativeExtensionRegistry.TTLV_VALUE, scalar);
            release(NativeExtensionRegistry.PATH, path);
            release(NativeExtensionRegistry.PATH, rootPath);
            release(NativeExtensionRegistry.TAG, childTag);
            release(NativeExtensionRegistry.RAW_TAG, childRawTag);
            release(NativeExtensionRegistry.TAG, rootTag);
            release(NativeExtensionRegistry.RAW_TAG, rootRawTag);
            release(NativeExtensionRegistry.COMPATIBILITY, compatibility);
            release(NativeExtensionRegistry.IDENTITY, identity);
        }
    }

    private static void release(int handleKind, long handle) {
        if (handle != 0) {
            NativeExtensionRegistry.release(handleKind, handle);
        }
    }
}
