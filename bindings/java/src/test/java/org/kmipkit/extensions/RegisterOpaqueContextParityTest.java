package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertDoesNotThrow;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

/** Consumer parity checks for KMIPKIT-0044 Register context selection. */
final class RegisterOpaqueContextParityTest {
    private static final int OPAQUE_OBJECT_TYPE = 8;
    private static final int OPAQUE_OBJECT_TAG = 0x42005B;
    private static final int OPAQUE_DATA_TYPE_TAG = 0x420059;
    private static final int OPAQUE_DATA_VALUE_TAG = 0x42005A;
    private static final int CRYPTOGRAPHIC_ALGORITHM_TAG = 0x420028;
    private static final int CRYPTOGRAPHIC_LENGTH_TAG = 0x42002A;
    private static final int CRYPTOGRAPHIC_USAGE_MASK_TAG = 0x42002C;
    private static final int UNKNOWN_ATTRIBUTE_TAG = 0x541234;
    private static final CodecLimits LIMITS = CodecLimits.defaults();

    @Test
    void genericOpaqueObjectNeedsNoCryptographicAttributes() {
        var request = RegisterRequest.create(OPAQUE_OBJECT_TYPE, TtlvStructure.create(), opaqueObject());

        assertDoesNotThrow(() -> {
            try (ClientBatchItem ignored = ClientBatchItem.register(request)) {
                // The generic context must remain valid without crypto attributes.
            }
        });
    }

    @Test
    void managedOpaqueObjectAcceptsAllRequiredAttributesAndOrderedUnknownData() {
        var request = RegisterRequest.create(OPAQUE_OBJECT_TYPE, attributes(true, true, true), opaqueObject())
                .withOpaqueObjectContext(OpaqueObjectContext.MANAGED_CRYPTOGRAPHIC_OBJECT);

        assertDoesNotThrow(() -> {
            try (ClientBatchItem ignored = ClientBatchItem.register(request)) {
                // The unknown Enumeration and vendor Attribute stay in their supplied TTLV input.
            }
        });
    }

    @Test
    void managedOpaqueObjectRejectsEachMissingRequiredAttribute() {
        for (int missing = 0; missing < 3; missing++) {
            var request = RegisterRequest.create(
                            OPAQUE_OBJECT_TYPE,
                            attributes(missing != 0, missing != 1, missing != 2),
                            opaqueObject())
                    .withOpaqueObjectContext(OpaqueObjectContext.MANAGED_CRYPTOGRAPHIC_OBJECT);

            assertThrows(InvalidInputException.class, () -> ClientBatchItem.register(request));
        }
    }

    @Test
    void contextIsRejectedForNonOpaqueObjectTypes() {
        var request = RegisterRequest.create(2, attributes(true, true, true), opaqueObject())
                .withOpaqueObjectContext(OpaqueObjectContext.GENERIC);

        assertThrows(InvalidInputException.class, () -> ClientBatchItem.register(request));
    }

    @Test
    void registerAdaptersRejectNullInputsAndContext() {
        assertThrows(
                InvalidInputException.class,
                () -> RegisterRequest.create(OPAQUE_OBJECT_TYPE, null, null));

        var request = RegisterRequest.create(
                OPAQUE_OBJECT_TYPE, TtlvStructure.create(), opaqueObject());
        try (request) {
            assertThrows(
                    InvalidInputException.class,
                    () -> request.withOpaqueObjectContext(null));
        }

        assertThrows(InvalidInputException.class, () -> ClientBatchItem.register(null));
    }

    private static TtlvItem opaqueObject() {
        TtlvStructure fields = TtlvStructure.create();
        fields = TtlvStructure.withItem(
                fields,
                item(OPAQUE_DATA_TYPE_TAG, TtlvValue.Enumeration(0x80000001L)),
                LIMITS);
        fields = TtlvStructure.withItem(
                fields,
                item(OPAQUE_DATA_VALUE_TAG, TtlvValue.ByteString("opaque-data".getBytes(), LIMITS)),
                LIMITS);
        return item(OPAQUE_OBJECT_TAG, TtlvValue.structure(fields, LIMITS));
    }

    private static TtlvStructure attributes(boolean usageMask, boolean algorithm, boolean length) {
        TtlvStructure attributes = TtlvStructure.create();
        if (usageMask) {
            attributes = TtlvStructure.withItem(
                    attributes, item(CRYPTOGRAPHIC_USAGE_MASK_TAG, TtlvValue.Integer(3)), LIMITS);
        }
        if (algorithm) {
            attributes = TtlvStructure.withItem(
                    attributes,
                    item(CRYPTOGRAPHIC_ALGORITHM_TAG, TtlvValue.Enumeration(3)),
                    LIMITS);
        }
        if (length) {
            attributes = TtlvStructure.withItem(
                    attributes, item(CRYPTOGRAPHIC_LENGTH_TAG, TtlvValue.Integer(256)), LIMITS);
        }
        return TtlvStructure.withItem(
                attributes,
                item(UNKNOWN_ATTRIBUTE_TAG, TtlvValue.Enumeration(0xE1020304L)),
                LIMITS);
    }

    private static TtlvItem item(int rawTag, TtlvValue value) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }
}
