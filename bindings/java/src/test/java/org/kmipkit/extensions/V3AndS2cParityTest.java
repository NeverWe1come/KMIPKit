package org.kmipkit.extensions;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.protocol.CryptographicParameters;
import org.kmipkit.protocol.EphemeralMode;
import org.kmipkit.protocol.LeaseContext;
import org.kmipkit.protocol.LinkNormalizer;
import org.kmipkit.protocol.NistKeyLifecycle;
import org.kmipkit.protocol.VersionGate;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;

import java.nio.charset.StandardCharsets;
import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;

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
}
