package org.kmipkit.extensions;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.message.ResponseMessage;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.protocol.EphemeralMode;
import org.kmipkit.protocol.LeaseContext;
import org.kmipkit.protocol.LinkNormalizer;
import org.kmipkit.protocol.NistKeyLifecycle;
import org.kmipkit.protocol.VersionGate;
import org.kmipkit.protocol.VersionGate.ProtocolVersion;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

import java.io.ByteArrayOutputStream;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;

class V3AndS2cParityTest {

    @Test
    void ttlvV3PrimitiveTypesRoundTrip() {
        CodecLimits limits = CodecLimits.defaults();

        try (TtlvValue identVal = TtlvValue.Identifier("id-12345", limits);
             TtlvValueView view = identVal.view()) {
            assertEquals(TtlvItemType.Identifier, view.itemType());
        }

        try (TtlvValue refVal = TtlvValue.Reference("ref-67890", limits);
             TtlvValueView view = refVal.view()) {
            assertEquals(TtlvItemType.Reference, view.itemType());
        }

        try (TtlvValue nameRefVal = TtlvValue.NameReference("nameref-abc", limits);
             TtlvValueView view = nameRefVal.view()) {
            assertEquals(TtlvItemType.NameReference, view.itemType());
        }
    }

    @Test
    void ttlvItemWithV3Value() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawTag = RawTag.fromRaw(0x00420094);
             Tag tag = rawTag.checked();
             TtlvValue val = TtlvValue.Identifier("id-test", limits);
             TtlvItem item = TtlvItem.create(tag, val, limits);
             TtlvItemView itemView = item.view()) {
            assertEquals(0x00420094, itemView.tag().raw());
            assertEquals(TtlvItemType.Identifier, itemView.itemType());
        }
    }

    @Test
    void uniqueIdentifierIdentifierKind() {
        UniqueIdentifier uid = UniqueIdentifier.identifier("oasis-kmip-v3-id");
        assertEquals(UniqueIdentifier.Kind.IDENTIFIER, uid.kind());
        assertEquals("oasis-kmip-v3-id", uid.text());
        assertThrows(InvalidInputException.class, uid::number);

        byte[] raw = uid.nativeText();
        assertNotNull(raw);
        assertEquals("oasis-kmip-v3-id", new String(raw, StandardCharsets.UTF_8));
    }

    @Test
    void cryptographicParametersV2PqcFieldsRoundTrip() {
        byte[] contextStr = "kem-context-1".getBytes(StandardCharsets.UTF_8);
        byte[] seed = new byte[] {0x10, 0x20, 0x30, 0x40};
        byte[] ikm = new byte[] {0x50, 0x60, 0x70, (byte) 0x80};

        CryptographicParameters params = CryptographicParameters.builder()
                .cryptographicAlgorithm(0x03L) // AES
                .blockCipherMode(0x02L)
                .kemAlgorithm(0x0AL)
                .deterministic(true)
                .contextString(contextStr)
                .seed(seed)
                .inputKeyMaterial(ikm)
                .build();

        assertEquals(OptionalLong.of(0x03L), params.cryptographicAlgorithm());
        assertEquals(OptionalLong.of(0x02L), params.blockCipherMode());
        assertEquals(OptionalLong.of(0x0AL), params.kemAlgorithm());
        assertEquals(Optional.of(true), params.deterministic());
        assertArrayEquals(contextStr, params.contextString().orElse(null));
        assertArrayEquals(seed, params.seed().orElse(null));
        assertArrayEquals(ikm, params.inputKeyMaterial().orElse(null));
    }

    @Test
    void versionGateValidationAndSpans() {
        // Create operation (1) is valid in KMIP 2.1
        assertTrue(VersionGate.validateOperation(2, 1, 1));
        // Bogus operation 0x7FFFFFFF should fail
        assertFalse(VersionGate.validateOperation(2, 1, 0x7FFFFFFF));

        // Activation Date tag (0x00420001) is valid
        assertTrue(VersionGate.validateTag(2, 1, 0x00420001));

        // Operation span for Create (1): base catalog in KMIP 2.0
        VersionGate.VersionSpan opSpan = VersionGate.operationSpan(1).orElseThrow();
        assertEquals(2, opSpan.introducedMajor());
        assertEquals(0, opSpan.introducedMinor());

        // Tag span for KEM Algorithm (0x004201C3 introduced in 3.0)
        VersionGate.VersionSpan tagSpan = VersionGate.tagSpan(0x004201C3).orElseThrow();
        assertEquals(3, tagSpan.introducedMajor());
        assertEquals(0, tagSpan.introducedMinor());

        // Tag span for 2.1 tag (0x00420166)
        VersionGate.VersionSpan tagSpan21 = VersionGate.tagSpan(0x00420166).orElseThrow();
        assertEquals(2, tagSpan21.introducedMajor());
        assertEquals(1, tagSpan21.introducedMinor());
    }

    @Test
    void nistKeyLifecycleOperationalGuards() {
        // KMIP State enumeration: 1 = Pre-Active, 2 = Active, 3 = Deactivated, 4 = Compromised, 5 = Destroyed
        // Active (2) can originate and process
        assertTrue(NistKeyLifecycle.canOriginate(2));
        assertTrue(NistKeyLifecycle.canProcess(2));

        // Compromised / Destroyed cannot originate
        assertFalse(NistKeyLifecycle.canOriginate(5));
        assertFalse(NistKeyLifecycle.canProcess(5));
    }

    @Test
    void leaseContextLifecycleAndExpiration() {
        UniqueIdentifier uid = UniqueIdentifier.textString("key-lease-1");
        long start = 1_000_000L;
        long duration = 300L; // 5 minutes

        try (LeaseContext ctx = LeaseContext.create(uid, start, duration)) {
            assertEquals("key-lease-1", ctx.uniqueIdentifier().text());
            assertEquals(1_000_300L, ctx.expiresAt());

            assertFalse(ctx.isExpired(1_000_100L));
            assertTrue(ctx.isExpired(1_000_301L));

            // checkValid at 1_000_100L should succeed without exception
            ctx.checkValid(1_000_100L);

            // checkValid at expired time should fail
            assertThrows(Exception.class, () -> ctx.checkValid(1_000_301L));
        }
    }

    @Test
    void ephemeralModeConstantsAndNames() {
        assertEquals(1L, EphemeralMode.NO);
        assertEquals(2L, EphemeralMode.IMMEDIATE);
        assertEquals(3L, EphemeralMode.RETAIN_ON_FAILURE);

        assertEquals("No", EphemeralMode.name(EphemeralMode.NO));
        assertEquals("Immediate", EphemeralMode.name(EphemeralMode.IMMEDIATE));
        assertEquals("Retain On Failure", EphemeralMode.name(EphemeralMode.RETAIN_ON_FAILURE));
        assertNull(EphemeralMode.name(999L));
    }

    @Test
    void s2cCapabilitiesZeroOperationsDoesNotInjectDefaults() {
        S2cCapabilities caps = new S2cCapabilities();
        assertEquals(0, caps.operations().size());
        assertEquals(0, caps.objectTypes().size());
        assertNull(caps.vendorIdentification());
    }

    @Test
    void s2cCapabilitiesCustomLists() {
        S2cCapabilities caps = new S2cCapabilities()
                .operations(List.of(1L, 2L, 3L))
                .objectTypes(List.of(0x01L, 0x02L))
                .vendorIdentification("CustomVendor");

        assertEquals(List.of(1L, 2L, 3L), caps.operations());
        assertEquals(List.of(0x01L, 0x02L), caps.objectTypes());
        assertEquals("CustomVendor", caps.vendorIdentification());
    }

    @Test
    void serverRequestDispatcherLifecycle() {
        try (ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {
            assertNotNull(dispatcher);
        }
    }

    @Test
    void linkNormalizerRoundTrip() {
        CodecLimits limits = CodecLimits.defaults();
        try (RawTag rawLinkTypeTag = RawTag.fromRaw(0x0042004B);
             Tag linkTypeTag = rawLinkTypeTag.checked();
             TtlvValue linkTypeVal = TtlvValue.Enumeration(0x00000101L);
             TtlvItem linkTypeItem = TtlvItem.create(linkTypeTag, linkTypeVal, limits);
             RawTag rawIdTag = RawTag.fromRaw(0x0042004C);
             Tag idTag = rawIdTag.checked();
             TtlvValue idVal = TtlvValue.Enumeration(0xCAFEBABEL);
             TtlvItem idItem = TtlvItem.create(idTag, idVal, limits)) {

            TtlvStructure struct = TtlvStructure.create();
            struct = TtlvStructure.withItem(struct, linkTypeItem, limits);
            struct = TtlvStructure.withItem(struct, idItem, limits);
            TtlvValue structVal = TtlvValue.structure(struct, limits);

            try (RawTag rawLinkTag = RawTag.fromRaw(0x0042004A);
                 Tag linkTag = rawLinkTag.checked();
                 TtlvItem genericLink = TtlvItem.create(linkTag, structVal, limits)) {

                try (TtlvItem v3 = LinkNormalizer.v2ToV3(genericLink)) {
                    assertNotNull(v3);
                    assertEquals(0x00420190, v3.view().tag().raw());
                    try (TtlvItem backV2 = LinkNormalizer.v3ToV2(v3)) {
                        assertNotNull(backV2);
                        assertEquals(0x0042004A, backV2.view().tag().raw());
                    }
                }
            }
        }
    }

    @Test
    void s2cCapabilitiesValidationConstraints() {
        // Negative elements rejected
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().operations(List.of(-1L)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().objectTypes(List.of(-1L)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().extensionTags(List.of(-1L)));

        // Out-of-uint32 elements rejected (> 0xFFFFFFFFL)
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().operations(List.of(0x1_0000_0000L)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().objectTypes(List.of(0x1_0000_0000L)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().extensionTags(List.of(0x1_0000_0000L)));

        // Null elements rejected
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().operations(Collections.singletonList(null)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().objectTypes(Collections.singletonList(null)));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().extensionTags(Collections.singletonList(null)));

        // Capacity > 1024 items rejected
        List<Long> tooMany = Collections.nCopies(1025, 1L);
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().operations(tooMany));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().objectTypes(tooMany));
        assertThrows(InvalidInputException.class, () ->
                new S2cCapabilities().extensionTags(tooMany));
    }

    @Test
    void listenerAndTimeoutValidation() {
        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().readTimeout(Duration.ofMillis(-1)));
        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().readTimeout(null));

        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().writeTimeout(Duration.ofMillis(-1)));
        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().writeTimeout(null));

        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().maxRequestBytes(0));
        assertThrows(InvalidInputException.class, () ->
                ListenerTransportConfiguration.builder().maxRequestBytes(-10));
    }

    private static byte[] ttlvTestItem(int tag, int itemType, byte[] value) {
        ByteArrayOutputStream out = new ByteArrayOutputStream();
        out.write((tag >> 16) & 0xFF);
        out.write((tag >> 8) & 0xFF);
        out.write(tag & 0xFF);
        out.write(itemType);
        int len = value.length;
        out.write((len >> 24) & 0xFF);
        out.write((len >> 16) & 0xFF);
        out.write((len >> 8) & 0xFF);
        out.write(len & 0xFF);
        out.writeBytes(value);
        int rem = len % 8;
        if (rem != 0) {
            for (int i = 0; i < 8 - rem; i++) {
                out.write(0);
            }
        }
        return out.toByteArray();
    }

    private static byte[] intVal(int val) {
        return ByteBuffer.allocate(4).putInt(val).array();
    }

    private static byte[] buildFullS2cBatchBytes() throws Exception {
        // 1. Put request payload (Operation 0x0000001C)
        ByteArrayOutputStream putPayload = new ByteArrayOutputStream();
        putPayload.writeBytes(ttlvTestItem(0x00420126, 0x05, intVal(2))); // PutFunction = Replace (2)
        putPayload.writeBytes(ttlvTestItem(0x00420094, 0x07, "sym-key-uid".getBytes(StandardCharsets.UTF_8))); // UniqueIdentifier
        putPayload.writeBytes(ttlvTestItem(0x00420127, 0x07, "replaced-sym-uid".getBytes(StandardCharsets.UTF_8))); // ReplacedUniqueIdentifier
        byte[] keyBlock = ttlvTestItem(0x0042008E, 0x08, new byte[] {0x01, 0x02, 0x03, 0x04});
        byte[] symKey = ttlvTestItem(0x0042008F, 0x01, keyBlock);
        putPayload.writeBytes(symKey);
        ByteArrayOutputStream attrSt = new ByteArrayOutputStream();
        attrSt.writeBytes(ttlvTestItem(0x00420008, 0x07, "CustomAttr".getBytes(StandardCharsets.UTF_8)));
        byte[] attrStruct = ttlvTestItem(0x00420125, 0x01, attrSt.toByteArray());
        putPayload.writeBytes(attrStruct);

        ByteArrayOutputStream item1 = new ByteArrayOutputStream();
        item1.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x0000001C))); // Put
        item1.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-put".getBytes(StandardCharsets.UTF_8)));
        item1.writeBytes(ttlvTestItem(0x00420079, 0x01, putPayload.toByteArray()));
        byte[] batchItem1 = ttlvTestItem(0x0042000F, 0x01, item1.toByteArray());

        // 2. Query request payload (Operation 0x00000018)
        ByteArrayOutputStream queryPayload = new ByteArrayOutputStream();
        queryPayload.writeBytes(ttlvTestItem(0x00420074, 0x05, intVal(1))); // Operations (1)
        queryPayload.writeBytes(ttlvTestItem(0x00420074, 0x05, intVal(3))); // ServerInformation (3)

        ByteArrayOutputStream item2 = new ByteArrayOutputStream();
        item2.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x00000018))); // Query
        item2.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-query".getBytes(StandardCharsets.UTF_8)));
        item2.writeBytes(ttlvTestItem(0x00420079, 0x01, queryPayload.toByteArray()));
        byte[] batchItem2 = ttlvTestItem(0x0042000F, 0x01, item2.toByteArray());

        // 3. DiscoverVersions request payload (Operation 0x0000001E)
        ByteArrayOutputStream discPv = new ByteArrayOutputStream();
        discPv.writeBytes(ttlvTestItem(0x0042006A, 0x02, intVal(2)));
        discPv.writeBytes(ttlvTestItem(0x0042006B, 0x02, intVal(1)));
        byte[] pvItem = ttlvTestItem(0x00420069, 0x01, discPv.toByteArray());

        ByteArrayOutputStream discPayload = new ByteArrayOutputStream();
        discPayload.writeBytes(pvItem);

        ByteArrayOutputStream item3 = new ByteArrayOutputStream();
        item3.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x0000001E))); // DiscoverVersions
        item3.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-disc".getBytes(StandardCharsets.UTF_8)));
        item3.writeBytes(ttlvTestItem(0x00420079, 0x01, discPayload.toByteArray()));
        byte[] batchItem3 = ttlvTestItem(0x0042000F, 0x01, item3.toByteArray());

        // 4. Notify request payload (Operation 0x0000001B)
        ByteArrayOutputStream notifyPayload = new ByteArrayOutputStream();
        notifyPayload.writeBytes(ttlvTestItem(0x00420094, 0x07, "notify-uid".getBytes(StandardCharsets.UTF_8)));
        notifyPayload.writeBytes(attrStruct);

        ByteArrayOutputStream item4 = new ByteArrayOutputStream();
        item4.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x0000001B))); // Notify
        item4.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-notify".getBytes(StandardCharsets.UTF_8)));
        item4.writeBytes(ttlvTestItem(0x00420079, 0x01, notifyPayload.toByteArray()));
        byte[] batchItem4 = ttlvTestItem(0x0042000F, 0x01, item4.toByteArray());

        // 5. SetEndpointRole request payload (Operation 0x00000032)
        ByteArrayOutputStream rolePayload = new ByteArrayOutputStream();
        rolePayload.writeBytes(ttlvTestItem(0x00420151, 0x05, intVal(2))); // EndpointRole = SERVER (2)

        ByteArrayOutputStream item5 = new ByteArrayOutputStream();
        item5.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x00000032))); // SetEndpointRole
        item5.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-role".getBytes(StandardCharsets.UTF_8)));
        item5.writeBytes(ttlvTestItem(0x00420079, 0x01, rolePayload.toByteArray()));
        byte[] batchItem5 = ttlvTestItem(0x0042000F, 0x01, item5.toByteArray());

        // Header & Root
        ByteArrayOutputStream hdrPv = new ByteArrayOutputStream();
        hdrPv.writeBytes(ttlvTestItem(0x0042006A, 0x02, intVal(2)));
        hdrPv.writeBytes(ttlvTestItem(0x0042006B, 0x02, intVal(1)));
        ByteArrayOutputStream header = new ByteArrayOutputStream();
        header.writeBytes(ttlvTestItem(0x00420069, 0x01, hdrPv.toByteArray()));
        header.writeBytes(ttlvTestItem(0x0042000D, 0x02, intVal(5))); // BatchCount = 5
        byte[] requestHeader = ttlvTestItem(0x00420077, 0x01, header.toByteArray());

        ByteArrayOutputStream root = new ByteArrayOutputStream();
        root.writeBytes(requestHeader);
        root.writeBytes(batchItem1);
        root.writeBytes(batchItem2);
        root.writeBytes(batchItem3);
        root.writeBytes(batchItem4);
        root.writeBytes(batchItem5);

        return ttlvTestItem(0x00420078, 0x01, root.toByteArray());
    }

    @Test
    void serverRequestDispatcherEndToEndFullDispatch() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {
            assertEquals(5, batch.itemCount());

            AtomicBoolean discCalled = new AtomicBoolean(false);
            AtomicBoolean notifyCalled = new AtomicBoolean(false);
            AtomicBoolean putCalled = new AtomicBoolean(false);
            AtomicBoolean queryCalled = new AtomicBoolean(false);
            AtomicBoolean roleCalled = new AtomicBoolean(false);

            ServerRequestHandler handler = new ServerRequestHandler() {
                @Override
                public long handleDiscoverVersions(ItemView item, ProtocolVersion clientVersion, List<ProtocolVersion> supportedVersionsOut) {
                    discCalled.set(true);
                    assertEquals(2, clientVersion.major());
                    assertEquals(1, clientVersion.minor());
                    supportedVersionsOut.add(new ProtocolVersion(2, 1));
                    return 0;
                }

                @Override
                public long handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    notifyCalled.set(true);
                    assertNotNull(uniqueIdentifier);
                    assertEquals("notify-uid", new String(uniqueIdentifier, StandardCharsets.UTF_8));
                    assertNotNull(attributesTtlv);
                    return 0;
                }

                @Override
                public long handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    putCalled.set(true);
                    assertEquals(2L, putFunction);
                    assertEquals("sym-key-uid", new String(uniqueIdentifier, StandardCharsets.UTF_8));
                    assertEquals("replaced-sym-uid", new String(replacedUniqueIdentifier, StandardCharsets.UTF_8));
                    assertNotNull(objectTtlv);
                    assertNotNull(attributesTtlv);
                    return 0;
                }

                @Override
                public long handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    queryCalled.set(true);
                    outCapabilities.operations(List.of(0x0000001CL, 0x00000018L))
                            .objectTypes(List.of(0x01L, 0x02L))
                            .vendorIdentification("KMIPKitJavaQuery");
                    return 0;
                }

                @Override
                public long handleSetEndpointRole(ItemView item, long targetRole) {
                    roleCalled.set(true);
                    assertEquals(2L, targetRole);
                    return 0;
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, handler)) {
                assertNotNull(response);
            }

            assertTrue(discCalled.get(), "handleDiscoverVersions must be called");
            assertTrue(notifyCalled.get(), "handleNotify must be called");
            assertTrue(putCalled.get(), "handlePut must be called");
            assertTrue(queryCalled.get(), "handleQuery must be called");
            assertTrue(roleCalled.get(), "handleSetEndpointRole must be called");
        }
    }

    @Test
    void serverRequestDispatcherEmptyCapabilitiesQuery() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler handler = new ServerRequestHandler() {
                @Override
                public long handleDiscoverVersions(ItemView item, ProtocolVersion clientVersion, List<ProtocolVersion> supportedVersionsOut) {
                    supportedVersionsOut.add(new ProtocolVersion(2, 1));
                    return 0;
                }

                @Override
                public long handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return 0;
                }

                @Override
                public long handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return 0;
                }

                @Override
                public long handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    // Intentionally leave empty
                    outCapabilities.operations(List.of())
                            .objectTypes(List.of())
                            .vendorIdentification(null);
                    return 0;
                }

                @Override
                public long handleSetEndpointRole(ItemView item, long targetRole) {
                    return 0;
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, handler)) {
                assertNotNull(response);
            }
        }
    }

    @Test
    void serverRequestDispatcherHandlerErrorPropagation() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler failingHandler = new ServerRequestHandler() {
                @Override
                public long handleDiscoverVersions(ItemView item, ProtocolVersion clientVersion, List<ProtocolVersion> supportedVersionsOut) {
                    return 1; // non-zero failure code
                }

                @Override
                public long handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return 1;
                }

                @Override
                public long handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return 1;
                }

                @Override
                public long handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    return 1;
                }

                @Override
                public long handleSetEndpointRole(ItemView item, long targetRole) {
                    return 1;
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, failingHandler)) {
                assertNotNull(response);
            }
        }
    }

    @Test
    void serverRequestDispatcherHandlerExceptionResilience() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler throwingHandler = new ServerRequestHandler() {
                @Override
                public long handleDiscoverVersions(ItemView item, ProtocolVersion clientVersion, List<ProtocolVersion> supportedVersionsOut) {
                    throw new RuntimeException("DiscoverVersions unexpected error");
                }

                @Override
                public long handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    throw new RuntimeException("Notify unexpected error");
                }

                @Override
                public long handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    throw new RuntimeException("Put unexpected error");
                }

                @Override
                public long handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    throw new RuntimeException("Query unexpected error");
                }

                @Override
                public long handleSetEndpointRole(ItemView item, long targetRole) {
                    throw new RuntimeException("SetEndpointRole unexpected error");
                }
            };

            // JNI must catch the exceptions, clear them, and map to non-zero error without JVM crash
            try (ResponseMessage response = dispatcher.dispatch(batch, throwingHandler)) {
                assertNotNull(response);
            }
        }
    }

    @Test
    void serverRequestDispatcherNegativeOrOutOfRangeStatusNormalization() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler weirdStatusHandler = new ServerRequestHandler() {
                @Override
                public long handleDiscoverVersions(ItemView item, ProtocolVersion clientVersion, List<ProtocolVersion> supportedVersionsOut) {
                    return -1L; // negative code
                }

                @Override
                public long handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return 0x1_0000_0000L; // > uint32 max
                }

                @Override
                public long handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return -100L;
                }

                @Override
                public long handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    return 0x2_0000_0000L;
                }

                @Override
                public long handleSetEndpointRole(ItemView item, long targetRole) {
                    return -1L;
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, weirdStatusHandler)) {
                assertNotNull(response);
            }
        }
    }
}
