package org.kmipkit.credentials;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicReference;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValue;
import org.kmipkit.ttlv.TtlvValueView;
import org.junit.jupiter.api.Test;

final class AttestationCredentialFacadeTest {
    private static final int NONCE_TAG = 0x0042_00C8;
    private static final int NONCE_ID_TAG = 0x0042_00C9;
    private static final int NONCE_VALUE_TAG = 0x0042_00CA;
    private static final int ATTESTATION_TYPE_TAG = 0x0042_00C7;
    private static final int MEASUREMENT_TAG = 0x0042_00CB;
    private static final int ASSERTION_TAG = 0x0042_00CC;

    @Test
    void attestationCredentialPreservesBothEvidenceFieldsAndScopedOwnership() {
        TtlvStructure source = attestationStructure(true, true);
        try (TtlvStructureView sourceView = source.view()) {
            assertThrows(InvalidInputException.class,
                    () -> AttestationCredential.fromTtlv(source));
            assertEquals(4L, sourceView.itemCount());
        }

        AtomicReference<TtlvStructureView> borrowed = new AtomicReference<>();
        try (AttestationCredential credential = AttestationCredential.fromTtlv(source)) {
            assertTrue(sourceIsConsumed(source));
            assertEquals("AttestationCredential[redacted]", credential.toString());
            long itemCount = credential.withTtlv(view -> {
                borrowed.set(view);
                assertThrows(InvalidInputException.class, credential::intoTtlv);
                return Long.valueOf(view.itemCount());
            });
            assertEquals(4L, itemCount);
            assertThrows(InvalidInputException.class, borrowed.get()::itemCount);
        }

        TtlvStructure secondSource = attestationStructure(true, true);
        AttestationCredential credential = AttestationCredential.fromTtlv(secondSource);
        TtlvStructure restored = credential.intoTtlv();
        try (restored; TtlvStructureView view = restored.view()) {
            assertEquals(List.of(NONCE_TAG, ATTESTATION_TYPE_TAG, MEASUREMENT_TAG, ASSERTION_TAG),
                    readTags(view));
            assertAttestationValues(view);
        }
    }

    @Test
    void attestationCredentialRequiresAtLeastOneEvidenceField() {
        TtlvStructure source = attestationStructure(false, false);
        CredentialValidationException error = assertThrows(
                CredentialValidationException.class,
                () -> AttestationCredential.fromTtlv(source));
        assertEquals(CredentialValidationErrorKind.MISSING_FIELD, error.kind());
    }

    @Test
    void attestationCredentialAcceptsMeasurementOrAssertionAlone() {
        for (boolean measurement : List.of(true, false)) {
            TtlvStructure source = attestationStructure(measurement, !measurement);
            try (AttestationCredential credential = AttestationCredential.fromTtlv(source)) {
                assertEquals(3L, credential.withTtlv(TtlvStructureView::itemCount));
            }
        }
    }

    private static boolean sourceIsConsumed(TtlvStructure source) {
        return assertThrows(InvalidInputException.class, source::view) != null;
    }

    private static List<Integer> readTags(TtlvStructureView view) {
        List<Integer> tags = new ArrayList<>();
        for (long index = 0; index < view.itemCount(); index++) {
            try (TtlvItemView item = view.itemAt(index); Tag tag = item.tag()) {
                tags.add(tag.raw());
            }
        }
        return tags;
    }

    private static void assertAttestationValues(TtlvStructureView view) {
        try (TtlvItemView item = view.itemAt(1); TtlvValueView value = item.value()) {
            assertEquals(0xF123_4567L, value.enumeration());
        }
        try (TtlvItemView item = view.itemAt(0);
                TtlvValueView value = item.value();
                TtlvStructureView nonce = value.structure()) {
            assertArrayEquals(new byte[] {0x01, 0x02, (byte) 0xFE}, readBytes(nonce, 0));
            assertArrayEquals(new byte[] {(byte) 0xA1, 0x00, (byte) 0xFF}, readBytes(nonce, 1));
        }
        assertArrayEquals(new byte[] {0x10, 0x20, 0x30}, readBytes(view, 2));
        assertArrayEquals(new byte[] {0x40, 0x50, 0x60}, readBytes(view, 3));
    }

    private static byte[] readBytes(TtlvStructureView view, long index) {
        try (TtlvItemView item = view.itemAt(index); TtlvValueView value = item.value()) {
            byte[] bytes = new byte[Math.toIntExact(value.byteLength())];
            for (int offset = 0; offset < bytes.length; offset++) {
                bytes[offset] = (byte) value.byteAt(offset);
            }
            return bytes;
        }
    }

    private static TtlvStructure attestationStructure(
            boolean includeMeasurement, boolean includeAssertion) {
        CodecLimits limits = CodecLimits.defaults();
        TtlvStructure nonce = TtlvStructure.create();
        nonce = append(nonce, NONCE_ID_TAG,
                TtlvValue.ByteString(new byte[] {0x01, 0x02, (byte) 0xFE}, limits), limits);
        nonce = append(nonce, NONCE_VALUE_TAG,
                TtlvValue.ByteString(new byte[] {(byte) 0xA1, 0x00, (byte) 0xFF}, limits), limits);

        TtlvStructure structure = TtlvStructure.create();
        structure = append(structure, NONCE_TAG, TtlvValue.structure(nonce, limits), limits);
        structure = append(structure, ATTESTATION_TYPE_TAG,
                TtlvValue.Enumeration(0xF123_4567L), limits);
        if (includeMeasurement) {
            structure = append(structure, MEASUREMENT_TAG,
                    TtlvValue.ByteString(new byte[] {0x10, 0x20, 0x30}, limits), limits);
        }
        if (includeAssertion) {
            structure = append(structure, ASSERTION_TAG,
                    TtlvValue.ByteString(new byte[] {0x40, 0x50, 0x60}, limits), limits);
        }
        return structure;
    }

    private static TtlvStructure append(
            TtlvStructure structure, int rawTag, TtlvValue value, CodecLimits limits) {
        try (value; RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            TtlvItem item = TtlvItem.create(tag, value, limits);
            return TtlvStructure.withItem(structure, item, limits);
        }
    }
}
