package org.kmipkit.extensions;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.message.RequestBatchItemView;
import org.kmipkit.message.RequestMessage;
import org.kmipkit.message.ResponseBatchItemView;
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
import org.kmipkit.ttlv.TtlvCodec;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

import java.io.ByteArrayOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.ByteBuffer;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.security.KeyFactory;
import java.security.KeyStore;
import java.security.PrivateKey;
import java.security.SecureRandom;
import java.security.cert.Certificate;
import java.security.cert.CertificateFactory;
import java.security.cert.X509Certificate;
import java.security.spec.PKCS8EncodedKeySpec;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Base64;
import java.util.Collections;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import java.util.concurrent.atomic.AtomicReference;
import javax.net.ssl.KeyManagerFactory;
import javax.net.ssl.SSLContext;
import javax.net.ssl.SSLSocket;
import javax.net.ssl.TrustManagerFactory;

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
        assertFalse(VersionGate.validateOperation(2, 1, 0x1_0000_0001L));
        assertTrue(VersionGate.operationSpan(0x1_0000_0001L).isEmpty());
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

    private static byte[] buildS2cBatchItem1Bytes() throws Exception {
        ByteArrayOutputStream putPayload = new ByteArrayOutputStream();
        putPayload.writeBytes(ttlvTestItem(0x00420070, 0x05, intVal(2))); // PutFunction = Replace (2)
        putPayload.writeBytes(ttlvTestItem(0x00420094, 0x07, "sym-key-uid".getBytes(StandardCharsets.UTF_8))); // UniqueIdentifier
        putPayload.writeBytes(ttlvTestItem(0x00420076, 0x07, "replaced-sym-uid".getBytes(StandardCharsets.UTF_8))); // ReplacedUniqueIdentifier
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
        return ttlvTestItem(0x0042000F, 0x01, item1.toByteArray());
    }

    private static byte[] buildS2cBatchItem2Bytes() throws Exception {
        ByteArrayOutputStream queryPayload = new ByteArrayOutputStream();
        queryPayload.writeBytes(ttlvTestItem(0x00420074, 0x05, intVal(1))); // Operations (1)
        queryPayload.writeBytes(ttlvTestItem(0x00420074, 0x05, intVal(3))); // ServerInformation (3)

        ByteArrayOutputStream item2 = new ByteArrayOutputStream();
        item2.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x00000018))); // Query
        item2.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-query".getBytes(StandardCharsets.UTF_8)));
        item2.writeBytes(ttlvTestItem(0x00420079, 0x01, queryPayload.toByteArray()));
        return ttlvTestItem(0x0042000F, 0x01, item2.toByteArray());
    }

    private static byte[] buildS2cBatchItem3Bytes() throws Exception {
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
        return ttlvTestItem(0x0042000F, 0x01, item3.toByteArray());
    }

    private static byte[] buildS2cBatchItem4Bytes() throws Exception {
        ByteArrayOutputStream attrSt = new ByteArrayOutputStream();
        attrSt.writeBytes(ttlvTestItem(0x00420008, 0x07, "CustomAttr".getBytes(StandardCharsets.UTF_8)));
        byte[] attrStruct = ttlvTestItem(0x00420125, 0x01, attrSt.toByteArray());

        ByteArrayOutputStream notifyPayload = new ByteArrayOutputStream();
        notifyPayload.writeBytes(ttlvTestItem(0x00420094, 0x07, "notify-uid".getBytes(StandardCharsets.UTF_8)));
        notifyPayload.writeBytes(attrStruct);
        notifyPayload.writeBytes(ttlvTestItem(0x42013b, 0x01,
                ttlvTestItem(0x42000a, 0x07, "first".getBytes(StandardCharsets.UTF_8))));
        notifyPayload.writeBytes(ttlvTestItem(0x42013b, 0x01,
                ttlvTestItem(0x42000a, 0x07, "second".getBytes(StandardCharsets.UTF_8))));

        ByteArrayOutputStream item4 = new ByteArrayOutputStream();
        item4.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x0000001B))); // Notify
        item4.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-notify".getBytes(StandardCharsets.UTF_8)));
        item4.writeBytes(ttlvTestItem(0x00420079, 0x01, notifyPayload.toByteArray()));
        return ttlvTestItem(0x0042000F, 0x01, item4.toByteArray());
    }

    private static byte[] buildS2cBatchItem5Bytes() throws Exception {
        ByteArrayOutputStream rolePayload = new ByteArrayOutputStream();
        rolePayload.writeBytes(ttlvTestItem(0x00420151, 0x05, intVal(2))); // EndpointRole = SERVER (2)

        rolePayload.writeBytes(ttlvTestItem(0x42009d, 0x07,
                "server 🔑".getBytes(StandardCharsets.UTF_8)));
        ByteArrayOutputStream item5 = new ByteArrayOutputStream();
        item5.writeBytes(ttlvTestItem(0x0042005C, 0x05, intVal(0x00000032))); // SetEndpointRole
        item5.writeBytes(ttlvTestItem(0x00420093, 0x08, "item-role".getBytes(StandardCharsets.UTF_8)));
        item5.writeBytes(ttlvTestItem(0x00420079, 0x01, rolePayload.toByteArray()));
        return ttlvTestItem(0x0042000F, 0x01, item5.toByteArray());
    }

    private static byte[] buildS2cRequestHeaderBytes(int batchCount) throws Exception {
        ByteArrayOutputStream hdrPv = new ByteArrayOutputStream();
        hdrPv.writeBytes(ttlvTestItem(0x0042006A, 0x02, intVal(2)));
        hdrPv.writeBytes(ttlvTestItem(0x0042006B, 0x02, intVal(1)));
        ByteArrayOutputStream header = new ByteArrayOutputStream();
        header.writeBytes(ttlvTestItem(0x00420069, 0x01, hdrPv.toByteArray()));
        header.writeBytes(ttlvTestItem(0x0042000D, 0x02, intVal(batchCount)));
        return ttlvTestItem(0x00420077, 0x01, header.toByteArray());
    }

    private static byte[] buildFullS2cBatchBytes() throws Exception {
        byte[] batchItem1 = buildS2cBatchItem1Bytes();
        byte[] batchItem2 = buildS2cBatchItem2Bytes();
        byte[] batchItem3 = buildS2cBatchItem3Bytes();
        byte[] batchItem4 = buildS2cBatchItem4Bytes();
        byte[] batchItem5 = buildS2cBatchItem5Bytes();
        byte[] requestHeader = buildS2cRequestHeaderBytes(5);

        ByteArrayOutputStream root = new ByteArrayOutputStream();
        root.writeBytes(requestHeader);
        root.writeBytes(batchItem1);
        root.writeBytes(batchItem2);
        root.writeBytes(batchItem3);
        root.writeBytes(batchItem4);
        root.writeBytes(batchItem5);

        return ttlvTestItem(0x00420078, 0x01, root.toByteArray());
    }

    private static RequestMessage buildTestS2cRequestMessage() throws Exception {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure root = TtlvStructure.create();
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cRequestHeaderBytes(5), limits), limits);
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cBatchItem1Bytes(), limits), limits);
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cBatchItem2Bytes(), limits), limits);
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cBatchItem3Bytes(), limits), limits);
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cBatchItem4Bytes(), limits), limits);
        root = TtlvStructure.withItem(root, TtlvCodec.decodeWithLimits(buildS2cBatchItem5Bytes(), limits), limits);
        return RequestMessage.fromTtlv(root);
    }

    private static TtlvStructure buildTestResponseBatchItem(long operation, TtlvStructure payload) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure item = TtlvStructure.create();
        try (RawTag opTag = RawTag.fromRaw(0x42005C);
             Tag checkedOpTag = opTag.checked();
             TtlvValue opVal = TtlvValue.Enumeration(operation);
             TtlvItem opItem = TtlvItem.create(checkedOpTag, opVal, limits)) {
            item = TtlvStructure.withItem(item, opItem, limits);
        }
        try (RawTag stTag = RawTag.fromRaw(0x42007F);
             Tag checkedStTag = stTag.checked();
             TtlvValue stVal = TtlvValue.Enumeration(0);
             TtlvItem stItem = TtlvItem.create(checkedStTag, stVal, limits)) {
            item = TtlvStructure.withItem(item, stItem, limits);
        }
        if (payload != null) {
            try (RawTag plTag = RawTag.fromRaw(0x42007C);
                 Tag checkedPlTag = plTag.checked();
                 TtlvValue plVal = TtlvValue.structure(payload, limits);
                 TtlvItem plItem = TtlvItem.create(checkedPlTag, plVal, limits)) {
                item = TtlvStructure.withItem(item, plItem, limits);
            }
        }
        return item;
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
                public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                    discCalled.set(true);
                    assertTrue(clientVersions.contains(new ProtocolVersion(2, 1)));
                    supportedVersionsOut.add(new ProtocolVersion(2, 1));
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    notifyCalled.set(true);
                    assertNotNull(uniqueIdentifier);
                    assertEquals("notify-uid", new String(uniqueIdentifier, StandardCharsets.UTF_8));
                    assertNotNull(attributesTtlv);
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    putCalled.set(true);
                    assertEquals(2L, putFunction);
                    assertEquals("sym-key-uid", new String(uniqueIdentifier, StandardCharsets.UTF_8));
                    assertEquals("replaced-sym-uid", new String(replacedUniqueIdentifier, StandardCharsets.UTF_8));
                    assertNotNull(objectTtlv);
                    assertNotNull(attributesTtlv);
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    queryCalled.set(true);
                    outCapabilities.operations(List.of(0x0000001CL, 0x00000018L))
                            .objectTypes(List.of(0x01L, 0x02L))
                            .vendorIdentification("KMIPKitJavaQuery");
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                    roleCalled.set(true);
                    assertEquals(2L, targetRole);
                    return S2cOperationResult.success();
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, handler)) {
                assertNotNull(response);
                assertEquals(5, response.batchItemCount());
                for (int i = 0; i < 5; i++) {
                    try (ResponseBatchItemView item = response.batchItemAt(i)) {
                        assertEquals(0L, item.resultStatus().orElseThrow(), "Item " + i + " resultStatus failed, reason: " + item.resultReason() + ", msg: " + item.resultMessage());
                        assertTrue(item.resultReason().isEmpty(), "Item " + i + " reason not empty");
                    }
                }
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
                public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                    supportedVersionsOut.add(new ProtocolVersion(2, 1));
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    // Intentionally leave empty
                    outCapabilities.operations(List.of())
                            .objectTypes(List.of())
                            .vendorIdentification(null);
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                    return S2cOperationResult.success();
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, handler)) {
                assertNotNull(response);
                try (ResponseBatchItemView qItem = response.batchItemAt(3)) {
                    assertEquals(0L, qItem.resultStatus().orElseThrow());
                }
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
                public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                    return S2cOperationResult.failure(0x05L, "version mismatch unsupported");
                }

                @Override
                public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return S2cOperationResult.failure(0x01L, "notify object not found");
                }

                @Override
                public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return S2cOperationResult.failure(0x02L, "put permission denied");
                }

                @Override
                public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    return S2cOperationResult.failure(0x05L, "query unsupported");
                }

                @Override
                public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                    return S2cOperationResult.failure(0x08L, "invalid role request");
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, failingHandler)) {
                assertNotNull(response);
                assertEquals(5, response.batchItemCount());

                // Item 0: Put
                try (ResponseBatchItemView item = response.batchItemAt(0)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x02L, item.resultReason().orElseThrow());
                    assertEquals("put permission denied", item.resultMessage().orElseThrow());
                }
                // Item 1: Query
                try (ResponseBatchItemView item = response.batchItemAt(1)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x05L, item.resultReason().orElseThrow());
                    assertEquals("query unsupported", item.resultMessage().orElseThrow());
                }
                // Item 2: DiscoverVersions
                try (ResponseBatchItemView item = response.batchItemAt(2)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x05L, item.resultReason().orElseThrow());
                    assertEquals("version mismatch unsupported", item.resultMessage().orElseThrow());
                }
                // Item 3: Notify
                try (ResponseBatchItemView item = response.batchItemAt(3)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x01L, item.resultReason().orElseThrow());
                    assertEquals("notify object not found", item.resultMessage().orElseThrow());
                }
                // Item 4: SetEndpointRole
                try (ResponseBatchItemView item = response.batchItemAt(4)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x08L, item.resultReason().orElseThrow());
                    assertEquals("invalid role request", item.resultMessage().orElseThrow());
                }
            }
        }
    }

    @Test
    void serverRequestDispatcherCustomFailureWithoutMessage() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler pendingHandler = new ServerRequestHandler() {
                @Override
                public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                    supportedVersionsOut.add(new ProtocolVersion(2, 1));
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    return S2cOperationResult.failure(0x04L);
                }

                @Override
                public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    return S2cOperationResult.success();
                }

                @Override
                public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                    return S2cOperationResult.success();
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, pendingHandler)) {
                assertNotNull(response);
                try (ResponseBatchItemView item = response.batchItemAt(0)) {
                    assertEquals(1L, item.resultStatus().orElseThrow());
                    assertEquals(0x04L, item.resultReason().orElseThrow());
                    assertTrue(item.resultMessage().isEmpty());
                }
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
                public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                    throw new RuntimeException("DiscoverVersions unexpected error");
                }

                @Override
                public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                    throw new RuntimeException("Notify unexpected error");
                }

                @Override
                public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                    throw new RuntimeException("Put unexpected error");
                }

                @Override
                public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                    throw new RuntimeException("Query unexpected error");
                }

                @Override
                public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                    throw new RuntimeException("SetEndpointRole unexpected error");
                }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, throwingHandler)) {
                assertNotNull(response);
                assertEquals(5, response.batchItemCount());
                for (int i = 0; i < 5; i++) {
                    try (ResponseBatchItemView item = response.batchItemAt(i)) {
                        assertEquals(1L, item.resultStatus().orElseThrow());
                        assertEquals(0x100L, item.resultReason().orElseThrow());
                        assertEquals("Handler exception", item.resultMessage().orElseThrow());
                    }
                }
            }
        }
    }

    @Test
    void serverRequestDispatcherHandlerNullReturn() throws Exception {
        byte[] batchBytes = buildFullS2cBatchBytes();
        CodecLimits limits = CodecLimits.defaults();

        try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(batchBytes, limits);
             ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {

            ServerRequestHandler nullHandler = new ServerRequestHandler() {
                @Override public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) { return null; }
                @Override public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) { return null; }
                @Override public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) { return null; }
                @Override public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) { return null; }
                @Override public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) { return null; }
            };

            try (ResponseMessage response = dispatcher.dispatch(batch, nullHandler)) {
                assertNotNull(response);
                assertEquals(5, response.batchItemCount());
                for (int i = 0; i < 5; i++) {
                    try (ResponseBatchItemView item = response.batchItemAt(i)) {
                        assertEquals(1L, item.resultStatus().orElseThrow());
                        assertEquals(0x100L, item.resultReason().orElseThrow());
                        assertEquals("Handler returned null", item.resultMessage().orElseThrow());
                    }
                }
            }
        }
    }

    @Test
    void s2cOperationResultValidation() {
        S2cOperationResult success = S2cOperationResult.success();
        assertTrue(success.isSuccess());
        assertEquals(0L, success.status());
        assertTrue(success.reason().isEmpty());
        assertTrue(success.message().isEmpty());

        S2cOperationResult fail = S2cOperationResult.failure(0x05L, "custom fail");
        assertFalse(fail.isSuccess());
        assertEquals(1L, fail.status());
        assertEquals(0x05L, fail.reason().orElseThrow());
        assertEquals("custom fail", fail.message().orElseThrow());

        assertEquals(fail, S2cOperationResult.of(1L, 0x05L, "custom fail"));
        assertEquals(fail.hashCode(), S2cOperationResult.of(1L, 0x05L, "custom fail").hashCode());
        assertTrue(fail.toString().contains("custom fail"));

        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(0L, 0x01L, null));
        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(1L, null, "msg"));
        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(-1L, null, null));
        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(0x1_0000_0000L, null, null));
        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(1L, -1L, null));
        assertThrows(IllegalArgumentException.class, () -> S2cOperationResult.of(1L, 0x1_0000_0000L, null));
    }

    @Test
    void clientListenerRawTlsLoopbackRoundTrip() throws Exception {
        checkRawTlsRoundTrip("TLSv1.3", false);
        checkRawTlsRoundTrip("TLSv1.2", true);
    }

    private static void checkRawTlsRoundTrip(String protocol, boolean compatibility) throws Exception {
        byte[] serverCert = testFixture("server-cert.pem");
        byte[] serverKey = testFixture("server-key.pem");
        byte[] clientCert = testFixture("client-cert.pem");
        byte[] clientKey = testFixture("client-key.pem");

        try (ListenerTransportConfiguration config = ListenerTransportConfiguration.builder()
                .bind("127.0.0.1", 0)
                .tls12Compat(compatibility)
                .serverIdentityPem(new byte[][] {serverCert}, serverKey)
                .trustCertificatesPem(new byte[][] {clientCert})
                .readTimeout(Duration.ofSeconds(10))
                .writeTimeout(Duration.ofSeconds(10))
                .build();
             ClientListener listener = ClientListener.bindRawTls(config)) {

            int port = listener.port();
            assertTrue(port > 0, "bound port should be positive");

            // Test negative duration rejection on listener.accept
            assertThrows(InvalidInputException.class, () -> listener.accept(Duration.ofMillis(-1)));

            byte[] batchBytes = buildFullS2cBatchBytes();
            SSLContext clientCtx = clientMtlsContext(clientCert, clientKey, serverCert);

            ExecutorService exec = Executors.newSingleThreadExecutor();
            try {
                Future<byte[]> clientFuture = exec.submit(() -> {
                    try (SSLSocket socket = (SSLSocket) clientCtx.getSocketFactory().createSocket("127.0.0.1", port)) {
                        socket.setSoTimeout(10_000);
                        socket.setEnabledProtocols(new String[]{protocol});
                        socket.startHandshake();
                        assertEquals(protocol, socket.getSession().getProtocol());
                        OutputStream out = socket.getOutputStream();
                        out.write(batchBytes);
                        out.flush();

                        InputStream in = socket.getInputStream();
                        byte[] hdr = in.readNBytes(8);
                        assertEquals(8, hdr.length);
                        int payloadLen = ByteBuffer.wrap(hdr, 4, 4).getInt();
                        byte[] payload = in.readNBytes(payloadLen);
                        assertEquals(payloadLen, payload.length);

                        ByteArrayOutputStream fullResp = new ByteArrayOutputStream();
                        fullResp.write(hdr);
                        fullResp.write(payload);
                        return fullResp.toByteArray();
                    }
                });

                try (ClientListenerConnection conn = listener.accept(Duration.ofSeconds(10))) {
                    assertNotNull(conn);

                    // Test negative duration rejection on recvBatch
                    CodecLimits limits = CodecLimits.defaults();
                    assertThrows(InvalidInputException.class, () -> conn.recvBatch(limits, Duration.ofMillis(-1)));

                    try (IncomingServerBatch batch = conn.recvBatch(limits, Duration.ofSeconds(10));
                         ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create()) {
                        assertEquals(5, batch.itemCount());

                        ServerRequestHandler handler = new ServerRequestHandler() {
                            @Override
                            public S2cOperationResult handleDiscoverVersions(ItemView item, List<ProtocolVersion> clientVersions, List<ProtocolVersion> supportedVersionsOut) {
                                supportedVersionsOut.add(new ProtocolVersion(2, 1));
                                return S2cOperationResult.success();
                            }

                            @Override
                            public S2cOperationResult handleNotify(ItemView item, byte[] uniqueIdentifier, byte[] attributesTtlv) {
                                return S2cOperationResult.success();
                            }

                            @Override
                            public S2cOperationResult handlePut(ItemView item, long putFunction, byte[] uniqueIdentifier, byte[] replacedUniqueIdentifier, byte[] objectTtlv, byte[] attributesTtlv) {
                                return S2cOperationResult.failure(0x02L, "net-put-denied");
                            }

                            @Override
                            public S2cOperationResult handleQuery(ItemView item, long queryFunctionFlags, S2cCapabilities outCapabilities) {
                                return S2cOperationResult.success();
                            }

                            @Override
                            public S2cOperationResult handleSetEndpointRole(ItemView item, long targetRole) {
                                return S2cOperationResult.success();
                            }
                        };

                        try (ResponseMessage resp = dispatcher.dispatch(batch, handler)) {
                            // Test negative duration rejection on sendResponse
                            assertThrows(InvalidInputException.class, () -> conn.sendResponse(resp, Duration.ofMillis(-1)));
                            conn.sendResponse(resp, Duration.ofSeconds(10));
                        }
                    }
                }

                byte[] clientReceivedBytes = clientFuture.get(10, TimeUnit.SECONDS);
                assertNotNull(clientReceivedBytes);
                assertTrue(clientReceivedBytes.length > 8);

                String receivedStr = new String(clientReceivedBytes, StandardCharsets.UTF_8);
                assertTrue(receivedStr.contains("net-put-denied"), "client must receive the custom failure message over TLS");
            } finally {
                exec.shutdownNow();
            }
        }
    }

    @Test
    void s2cUtilitiesDiscoverVersionsNegotiate() {
        ProtocolVersion req21 = new ProtocolVersion(2, 1);
        List<ProtocolVersion> supported = List.of(
                new ProtocolVersion(1, 4),
                new ProtocolVersion(2, 0),
                new ProtocolVersion(2, 1));

        Optional<ProtocolVersion> negotiated = S2cUtilities.negotiateDiscoverVersions(req21, supported);
        assertTrue(negotiated.isPresent());
        assertEquals(2, negotiated.get().major());
        assertEquals(1, negotiated.get().minor());

        // Higher version requested
        ProtocolVersion req30 = new ProtocolVersion(3, 0);
        List<ProtocolVersion> serverVersions = List.of(
                new ProtocolVersion(2, 1),
                new ProtocolVersion(3, 0));
        Optional<ProtocolVersion> negotiated30 = S2cUtilities.negotiateDiscoverVersions(req30, serverVersions);
        assertTrue(negotiated30.isPresent());
        assertEquals(3, negotiated30.get().major());
        assertEquals(0, negotiated30.get().minor());

        // No mutual version matches
        ProtocolVersion req20 = new ProtocolVersion(2, 0);
        List<ProtocolVersion> disjoint = List.of(new ProtocolVersion(2, 1), new ProtocolVersion(3, 0));
        Optional<ProtocolVersion> none = S2cUtilities.negotiateDiscoverVersions(req20, disjoint);
        assertTrue(none.isEmpty());

        // Parameter validation
        assertThrows(InvalidInputException.class, () -> S2cUtilities.negotiateDiscoverVersions(null, supported));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.negotiateDiscoverVersions(req21, null));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.negotiateDiscoverVersions(
                new ProtocolVersion(-1, 0), supported));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.negotiateDiscoverVersions(
                req21, Collections.singletonList(null)));
    }

    @Test
    void discoversAllVersionsInLocalOrder() {
        var v21 = new ProtocolVersion(2, 1);
        var v30 = new ProtocolVersion(3, 0);
        var v20 = new ProtocolVersion(2, 0);
        var supported = List.of(v30, v21, v20);
        assertEquals(supported, S2cUtilities.negotiateDiscoverVersionsList(List.of(), supported));
        assertEquals(List.of(v30, v21), S2cUtilities.negotiateDiscoverVersionsList(List.of(v21, v30), supported));
        assertEquals(List.of(), S2cUtilities.negotiateDiscoverVersionsList(List.of(v30), List.of(v21)));
    }

    @Test
    void incomingDiscoverVersionsUsesReportedBufferLengths() throws Exception {
        byte[] version = ttlvTestItem(0x420069, 1,
                concat(ttlvTestItem(0x42006A, 2, intVal(2)), ttlvTestItem(0x42006B, 2, intVal(1))));
        ByteArrayOutputStream versions = new ByteArrayOutputStream();
        for (int i = 0; i < 1025; i++) versions.writeBytes(version);
        byte[] id = new byte[257];
        java.util.Arrays.fill(id, (byte) 0xA5);
        byte[] item = ttlvTestItem(0x42000F, 1,
                concat(ttlvTestItem(0x42005C, 5, intVal(0x1E)), ttlvTestItem(0x420093, 8, id),
                        ttlvTestItem(0x420079, 1, versions.toByteArray())));
        byte[] request = ttlvTestItem(0x420078, 1, concat(buildS2cRequestHeaderBytes(1), item));
        var expected = Collections.nCopies(1025, new ProtocolVersion(2, 1));
        ServerRequestHandler handler = (ServerRequestHandler) java.lang.reflect.Proxy.newProxyInstance(
                getClass().getClassLoader(), new Class<?>[]{ServerRequestHandler.class},
                (proxy, method, args) -> {
                    assertEquals("handleDiscoverVersions", method.getName());
                    assertEquals(expected, args[1]);
                    return S2cOperationResult.success();
                });
        try (var batch = IncomingServerBatch.decodeTtlv(request, CodecLimits.defaults());
             var dispatcher = ServerRequestDispatcher.create();
             var response = dispatcher.dispatch(batch, handler);
             var result = response.batchItemAt(0)) {
            assertEquals(expected, batch.itemDiscoverVersions(0));
            assertArrayEquals(id, batch.itemUniqueBatchItemId(0));
            assertEquals(0, result.resultStatus().orElseThrow());
        }
    }

    private static byte[] concat(byte[]... parts) {
        var bytes = new ByteArrayOutputStream();
        for (byte[] part : parts) bytes.writeBytes(part);
        return bytes.toByteArray();
    }

    @Test
    void s2cUtilitiesPutEffectiveFunction() {
        // Replace (2) degrades to New (1) if object does not exist locally
        assertEquals(1L, S2cUtilities.effectivePutFunction(2L, false));
        // Replace (2) stays Replace (2) if object exists locally
        assertEquals(2L, S2cUtilities.effectivePutFunction(2L, true));
        // New (1) always remains New (1)
        assertEquals(1L, S2cUtilities.effectivePutFunction(1L, false));
        assertEquals(1L, S2cUtilities.effectivePutFunction(1L, true));
        // Custom PutFunction remains unchanged
        assertEquals(0x80000001L, S2cUtilities.effectivePutFunction(0x80000001L, false));

        // Bounds checks
        assertThrows(InvalidInputException.class, () -> S2cUtilities.effectivePutFunction(-1L, false));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.effectivePutFunction(0x100000000L, true));
    }

    @Test
    void s2cUtilitiesQueryRespondTtlv() {
        // Default capabilities with vendor ID
        try (TtlvStructure struct = S2cUtilities.buildQueryResponseTtlv(1L, "KMIPKitVendor")) {
            assertNotNull(struct);
            assertTrue(struct.handle() != 0);
        }

        // Default capabilities without vendor ID
        try (TtlvStructure struct = S2cUtilities.buildQueryResponseTtlv(3L)) {
            assertNotNull(struct);
            assertTrue(struct.handle() != 0);
        }

        // Custom S2cCapabilities
        S2cCapabilities caps = new S2cCapabilities()
                .operations(List.of(0x0000001CL, 0x00000018L))
                .objectTypes(List.of(1L, 2L, 7L))
                .vendorIdentification("CustomVendor");
        try (TtlvStructure struct = S2cUtilities.buildQueryResponseTtlv(7L, caps)) {
            assertNotNull(struct);
            assertTrue(struct.handle() != 0);
        }

        // Deliberately empty lists in S2cCapabilities
        S2cCapabilities emptyCaps = new S2cCapabilities();
        try (TtlvStructure struct = S2cUtilities.buildQueryResponseTtlv(1L, emptyCaps)) {
            assertNotNull(struct);
            assertTrue(struct.handle() != 0);
        }

        try (var info = extensionForQuery(TtlvItemType.TextString)) {
            var extensions = new S2cCapabilities().extensionInformation(List.of(info));
            try (var response = S2cUtilities.buildQueryResponseTtlv(0x08, extensions);
                 var fields = response.view()) {
                assertEquals(0, fields.itemCount());
            }
        }

        // Validation bounds
        assertThrows(InvalidInputException.class, () -> S2cUtilities.buildQueryResponseTtlv(-1L, "Vendor"));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.buildQueryResponseTtlv(0x100000000L, "Vendor"));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.buildQueryResponseTtlv(1L, (S2cCapabilities) null));
    }

    @Test
    void extensionQueryListAndMapInAllProtocolVersions() throws Exception {
        try (var info = extensionForQuery(TtlvItemType.TextString);
             var dispatcher = ServerRequestDispatcher.create()) {
            var capabilities = new S2cCapabilities().extensionInformation(List.of(info));
            ServerRequestHandler handler = (ServerRequestHandler) java.lang.reflect.Proxy.newProxyInstance(
                    getClass().getClassLoader(), new Class<?>[]{ServerRequestHandler.class},
                    (proxy, method, args) -> {
                        assertEquals("handleQuery", method.getName());
                        assertEquals(0x30L, args[1]);
                        ((S2cCapabilities) args[2]).extensionInformation(List.of(info));
                        return S2cOperationResult.success();
                    });

            for (var version : List.of(new ProtocolVersion(2, 0), new ProtocolVersion(2, 1),
                    new ProtocolVersion(3, 0))) {
                for (long flags : new long[]{0x10, 0x20, 0x30}) {
                    try (var response = S2cUtilities.buildQueryResponseTtlv(version, flags, capabilities);
                         var payload = response.view()) {
                        assertExtensionInformation(payload, flags != 0x10, TtlvItemType.TextString.code());
                    }
                }

                try (var batch = IncomingServerBatch.decodeTtlv(extensionQueryBatch(version), CodecLimits.defaults());
                     var response = dispatcher.dispatch(batch, handler);
                     var item = response.batchItemAt(0)) {
                    assertEquals(0, item.resultStatus().orElseThrow());
                    item.withResponsePayload(payload -> {
                        assertExtensionInformation(payload, true, TtlvItemType.TextString.code());
                        return true;
                    });
                }
            }
        }
    }

    @Test
    @SuppressWarnings("deprecation")
    void extensionQueryRejectsLegacyTagsAndDraftTypesInKmip2() throws Exception {
        var legacy = new S2cCapabilities().extensionTags(List.of(0x540001L));
        assertThrows(InvalidExtensionSchemaException.class,
                () -> S2cUtilities.buildQueryResponseTtlv(0x20, legacy));
        try (var info = extensionForQuery(TtlvItemType.NameReference);
             var dispatcher = ServerRequestDispatcher.create()) {
            var capabilities = new S2cCapabilities().extensionInformation(List.of(info));
            for (var version : List.of(new ProtocolVersion(2, 0), new ProtocolVersion(2, 1))) {
                assertThrows(InvalidExtensionSchemaException.class,
                        () -> S2cUtilities.buildQueryResponseTtlv(version, 0x20, capabilities));
                ServerRequestHandler handler = (ServerRequestHandler) java.lang.reflect.Proxy.newProxyInstance(
                        getClass().getClassLoader(), new Class<?>[]{ServerRequestHandler.class},
                        (proxy, method, args) -> {
                            ((S2cCapabilities) args[2]).extensionInformation(List.of(info));
                            return S2cOperationResult.success();
                        });
                try (var batch = IncomingServerBatch.decodeTtlv(extensionQueryBatch(version), CodecLimits.defaults());
                     var response = dispatcher.dispatch(batch, handler);
                     var item = response.batchItemAt(0)) {
                    assertEquals(1, item.resultStatus().orElseThrow());
                    assertEquals(7, item.resultReason().orElseThrow());
                }
            }
            try (var response = S2cUtilities.buildQueryResponseTtlv(
                         new ProtocolVersion(3, 0), 0x20, capabilities);
                 var payload = response.view()) {
                assertExtensionInformation(payload, true, TtlvItemType.NameReference.code());
            }
        }
    }

    private static ExtensionInformation extensionForQuery(TtlvItemType itemType) {
        return ExtensionInformation.with_type(
                ExtensionInformation.with_tag(ExtensionInformation.create("Tenant🔑"), 0x540001), itemType);
    }

    private static byte[] extensionQueryBatch(ProtocolVersion version) throws Exception {
        byte[] headerVersion = ttlvTestItem(0x420069, 1,
                concat(ttlvTestItem(0x42006A, 2, intVal(version.major())),
                        ttlvTestItem(0x42006B, 2, intVal(version.minor()))));
        byte[] headerFields = version.major() == 3 ? headerVersion
                : concat(headerVersion, ttlvTestItem(0x42000D, 2, intVal(1)));
        byte[] header = ttlvTestItem(0x420077, 1, headerFields);
        byte[] item = ttlvTestItem(0x42000F, 1,
                concat(ttlvTestItem(0x42005C, 5, intVal(0x18)),
                        ttlvTestItem(0x420079, 1, concat(ttlvTestItem(0x420074, 5, intVal(5)),
                                ttlvTestItem(0x420074, 5, intVal(6))))));
        return ttlvTestItem(0x420078, 1, concat(header, item));
    }

    private static void assertExtensionInformation(TtlvStructureView payload, boolean map, int itemType) {
        assertEquals(1, payload.itemCount());
        try (var item = payload.itemAt(0); var typeTag = item.tag(); var value = item.value();
             var fields = value.structure(); var name = fields.itemAt(0); var nameTag = name.tag();
             var nameValue = name.value()) {
            assertEquals(0x4200a4L, typeTag.raw());
            assertEquals(0x4200a5L, nameTag.raw());
            assertEquals("Tenant🔑", new String(nameValue.bytes(), StandardCharsets.UTF_8));
            assertEquals(map ? 3 : 1, fields.itemCount());
            if (map) {
                try (var tag = fields.itemAt(1); var tagTag = tag.tag(); var tagValue = tag.value();
                     var type = fields.itemAt(2); var extensionTypeTag = type.tag(); var typeValue = type.value()) {
                    assertEquals(0x4200a6L, tagTag.raw());
                    assertEquals(0x540001, tagValue.integer());
                    assertEquals(0x4200a7L, extensionTypeTag.raw());
                    assertEquals(itemType, typeValue.enumeration());
                }
            }
        }
    }

    @Test
    void s2cUtilitiesRequestValidationAndBuildResponseTtlv() throws Exception {
        try (RequestMessage reqMsg = buildTestS2cRequestMessage()) {
            assertEquals(5, reqMsg.batchItemCount());

            // Item 0 is Put
            try (RequestBatchItemView item0 = reqMsg.batchItemAt(0)) {
                assertTrue(S2cUtilities.validatePutRequest(item0));
                assertFalse(S2cUtilities.validateNotifyRequest(item0));
                assertFalse(S2cUtilities.validateQueryRequest(item0));
                assertFalse(S2cUtilities.validateDiscoverVersionsRequest(item0));
            }

            // Item 1 is Query
            try (RequestBatchItemView item1 = reqMsg.batchItemAt(1)) {
                assertTrue(S2cUtilities.validateQueryRequest(item1));
                assertFalse(S2cUtilities.validatePutRequest(item1));
                assertFalse(S2cUtilities.validateNotifyRequest(item1));
                assertFalse(S2cUtilities.validateDiscoverVersionsRequest(item1));
            }

            // Item 2 is DiscoverVersions
            try (RequestBatchItemView item2 = reqMsg.batchItemAt(2)) {
                assertTrue(S2cUtilities.validateDiscoverVersionsRequest(item2));
                assertFalse(S2cUtilities.validatePutRequest(item2));
                assertFalse(S2cUtilities.validateNotifyRequest(item2));
                assertFalse(S2cUtilities.validateQueryRequest(item2));
            }

            // Item 3 is Notify
            try (RequestBatchItemView item3 = reqMsg.batchItemAt(3)) {
                assertTrue(S2cUtilities.validateNotifyRequest(item3));
                assertFalse(S2cUtilities.validatePutRequest(item3));
                assertFalse(S2cUtilities.validateQueryRequest(item3));
                assertFalse(S2cUtilities.validateDiscoverVersionsRequest(item3));
            }
        }

        // Test buildResponseTtlv
        try (TtlvStructure payload1 = S2cUtilities.buildQueryResponseTtlv(1L, "Vendor1");
             TtlvStructure batchItem1 = buildTestResponseBatchItem(0x18, payload1);
             TtlvStructure response = S2cUtilities.buildResponseTtlv(
                     new ProtocolVersion(2, 1), List.of(batchItem1))) {
            assertNotNull(response);
            assertTrue(response.handle() != 0);
        }

        // Negative parameter validation
        assertThrows(InvalidInputException.class, () -> S2cUtilities.validateNotifyRequest(null));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.validatePutRequest(null));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.validateQueryRequest(null));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.validateDiscoverVersionsRequest(null));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.buildResponseTtlv(null, Collections.emptyList()));
        assertThrows(InvalidInputException.class, () -> S2cUtilities.buildResponseTtlv(new ProtocolVersion(2, 1), null));
    }

    @Test
    void richServerCallbacksPreserveDeletedReferencesAndEndpointVendor() throws Exception {
        for (String responseVendor : List.of("", "response\0🔑")) {
            AtomicBoolean notifySeen = new AtomicBoolean();
            AtomicBoolean roleSeen = new AtomicBoolean();
            ServerRequestHandler handler = (ServerRequestHandler) java.lang.reflect.Proxy.newProxyInstance(
                    getClass().getClassLoader(), new Class<?>[] {ServerRequestHandler.class},
                    (proxy, method, args) -> {
                        if (method.getName().equals("handleNotify")) {
                            assertEquals(4, args.length);
                            try (TtlvItem references = TtlvCodec.decode((byte[]) args[3]);
                                 TtlvItemView item = references.view();
                                 TtlvValueView value = item.value();
                                 var children = value.structure()) {
                                assertEquals(2, children.itemCount());
                                for (int index = 0; index < 2; index++) {
                                    try (var reference = children.itemAt(index);
                                         var referenceValue = reference.value();
                                         var fields = referenceValue.structure();
                                         var field = fields.itemAt(0);
                                         var text = field.value()) {
                                        assertEquals(index == 0 ? "first" : "second",
                                                new String(text.bytes(), StandardCharsets.UTF_8));
                                    }
                                }
                            }
                            notifySeen.set(true);
                        }
                        if (method.getName().equals("handleSetEndpointRole")) {
                            assertEquals(4, args.length);
                            assertEquals("server 🔑", args[2]);
                            ((S2cEndpointRoleResponse) args[3]).endpointRole(1).vendorIdentification(responseVendor);
                            roleSeen.set(true);
                        }
                        return S2cOperationResult.success();
                    });
            try (IncomingServerBatch batch = IncomingServerBatch.decodeTtlv(buildFullS2cBatchBytes(), CodecLimits.defaults());
                 ServerRequestDispatcher dispatcher = ServerRequestDispatcher.create();
                 ResponseMessage response = dispatcher.dispatch(batch, handler);
                 ResponseBatchItemView role = response.batchItemAt(4)) {
                assertTrue(notifySeen.get());
                assertTrue(roleSeen.get());
                assertEquals(0, role.resultStatus().orElseThrow());
                role.withResponsePayload(payload -> {
                    for (long index = 0; index < payload.itemCount(); index++) {
                        try (var field = payload.itemAt(index); var tag = field.tag(); var value = field.value()) {
                            if (tag.raw() == 0x420151) assertEquals(1, value.enumeration());
                            if (tag.raw() == 0x42009d) {
                                assertEquals(responseVendor, new String(value.bytes(), StandardCharsets.UTF_8));
                            }
                        }
                    }
                    assertEquals(2, payload.itemCount());
                    return true;
                });
            }
        }
    }

    private static byte[] testFixture(String filename) throws Exception {
        Path directory = Path.of("").toAbsolutePath();
        while (directory != null) {
            Path candidate = directory.resolve(
                    Path.of("bindings", "python", "tests", "fixtures", "login-mtls", filename));
            if (Files.isRegularFile(candidate)) return Files.readAllBytes(candidate);
            directory = directory.getParent();
        }
        throw new IllegalStateException("shared deterministic mTLS fixture is missing: " + filename);
    }

    private static X509Certificate testCertificate(byte[] pem) throws Exception {
        return (X509Certificate) CertificateFactory.getInstance("X.509")
                .generateCertificate(new java.io.ByteArrayInputStream(pem));
    }

    private static PrivateKey testPrivateKey(byte[] pem) throws Exception {
        String encoded = new String(pem, StandardCharsets.US_ASCII)
                .replace("-----BEGIN PRIVATE KEY-----", "")
                .replace("-----END PRIVATE KEY-----", "")
                .replaceAll("\\s", "");
        byte[] der = Base64.getDecoder().decode(encoded);
        return KeyFactory.getInstance("RSA").generatePrivate(new PKCS8EncodedKeySpec(der));
    }

    private static SSLContext clientMtlsContext(
            byte[] clientCertPem, byte[] clientKeyPem, byte[] serverCertPem) throws Exception {
        X509Certificate clientCert = testCertificate(clientCertPem);
        X509Certificate serverCert = testCertificate(serverCertPem);
        char[] pwd = "password".toCharArray();

        KeyStore clientStore = KeyStore.getInstance("PKCS12");
        clientStore.load(null, pwd);
        clientStore.setKeyEntry("client", testPrivateKey(clientKeyPem), pwd, new Certificate[] {clientCert});
        KeyManagerFactory kmf = KeyManagerFactory.getInstance(KeyManagerFactory.getDefaultAlgorithm());
        kmf.init(clientStore, pwd);

        KeyStore trustStore = KeyStore.getInstance("PKCS12");
        trustStore.load(null, pwd);
        trustStore.setCertificateEntry("server", serverCert);
        TrustManagerFactory tmf = TrustManagerFactory.getInstance(TrustManagerFactory.getDefaultAlgorithm());
        tmf.init(trustStore);

        SSLContext ctx = SSLContext.getInstance("TLSv1.3");
        ctx.init(kmf.getKeyManagers(), tmf.getTrustManagers(), new SecureRandom());
        return ctx;
    }
}
