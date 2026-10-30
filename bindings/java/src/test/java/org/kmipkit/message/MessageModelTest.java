package org.kmipkit.message;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.OptionalLong;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvValue;

final class MessageModelTest {
    private static final CodecLimits LIMITS = CodecLimits.defaults();
    private static final int REQUEST_HEADER = 0x420077;
    private static final int RESPONSE_HEADER = 0x42007A;
    private static final int BATCH_ITEM = 0x42000F;
    private static final int PROTOCOL_VERSION = 0x420069;
    private static final int PROTOCOL_VERSION_MAJOR = 0x42006A;
    private static final int PROTOCOL_VERSION_MINOR = 0x42006B;
    private static final int ASYNCHRONOUS_INDICATOR = 0x420007;
    private static final int BATCH_COUNT = 0x42000D;
    private static final int OPERATION = 0x42005C;
    private static final int REQUEST_PAYLOAD = 0x420079;
    private static final int TIME_STAMP = 0x420092;
    private static final int RESULT_STATUS = 0x42007F;
    private static final int MESSAGE_EXTENSION = 0x420051;
    private static final int VENDOR_IDENTIFICATION = 0x42009D;
    private static final int CRITICALITY_INDICATOR = 0x420026;
    private static final int VENDOR_EXTENSION = 0x42009C;
    private static final int VENDOR_TAG = 0x540001;
    private static final long RAW_UNKNOWN = 0xF1234567L;

    @Test
    void requestViewsPreserveRawEnumerationsDefaultsAndOwnerLifetime() {
        RequestMessage request = RequestMessage.fromTtlv(requestWithExtension());
        RequestHeaderView header = request.header();
        RequestBatchItemView item = request.batchItemAt(0);
        request.close();

        try (header; item) {
            assertEquals(2, header.protocolVersion().major());
            assertEquals(1, header.protocolVersion().minor());
            assertEquals(1, header.batchCount());
            assertEquals(OptionalLong.of(RAW_UNKNOWN), header.asynchronousIndicator());
            assertEquals(RAW_UNKNOWN, header.effectiveAsynchronousIndicator());
            assertTrue(header.batchOrderOption().isEmpty());
            assertTrue(header.effectiveBatchOrderOption());
            assertTrue(header.batchErrorContinuationOption().isEmpty());
            assertEquals(2, header.effectiveBatchErrorContinuationOption());

            assertEquals(OptionalLong.of(RAW_UNKNOWN), item.operation());
            assertEquals(1, item.extensionCount());
            try (MessageExtensionView extension = item.extensionAt(0);
                    TtlvStructure value = extension.ttlv();
                    var view = value.view()) {
                assertEquals(3, view.itemCount());
            }
        }
    }

    @Test
    void responseViewsPreserveTimestampAndUnknownResultStatus() {
        try (ResponseMessage response = ResponseMessage.fromTtlv(responseMessage());
                ResponseHeaderView header = response.header();
                ResponseBatchItemView item = response.batchItemAt(0)) {
            assertEquals(2, header.protocolVersion().major());
            assertEquals(1, header.protocolVersion().minor());
            assertEquals(123456789L, header.timeStamp());
            assertEquals(1, header.batchCount());
            assertEquals(OptionalLong.of(RAW_UNKNOWN), item.resultStatus());
            assertEquals(1, response.batchItemCount());
        }
    }

    @Test
    void validationFailureExposesOnlyStableKindAndIndexes() {
        MessageValidationException error = assertThrows(
                MessageValidationException.class,
                () -> RequestMessage.fromTtlv(invalidRequestWithPayloadSentinel()));

        assertEquals(1, error.kindCode());
        assertEquals(OptionalLong.of(0), error.structureIndex());
        assertFalse(error.getMessage().contains("payload-secret-sentinel"));
        assertFalse(error.toString().contains("payload-secret-sentinel"));
    }

    private static TtlvStructure requestWithExtension() {
        TtlvStructure version = structure(
                integer(PROTOCOL_VERSION_MAJOR, 2), integer(PROTOCOL_VERSION_MINOR, 1));
        TtlvStructure header = structure(
                nested(PROTOCOL_VERSION, version),
                enumeration(ASYNCHRONOUS_INDICATOR, RAW_UNKNOWN),
                integer(BATCH_COUNT, 1));

        TtlvStructure vendorValue = structure(integer(VENDOR_TAG, 47));
        TtlvStructure extension = structure(
                text(VENDOR_IDENTIFICATION, "example"),
                bool(CRITICALITY_INDICATOR, false),
                nested(VENDOR_EXTENSION, vendorValue));
        TtlvStructure payload = structure();
        TtlvStructure item = structure(
                enumeration(OPERATION, RAW_UNKNOWN),
                nested(REQUEST_PAYLOAD, payload),
                nested(MESSAGE_EXTENSION, extension));
        return message(REQUEST_HEADER, header, item);
    }

    private static TtlvStructure responseMessage() {
        TtlvStructure version = structure(
                integer(PROTOCOL_VERSION_MAJOR, 2), integer(PROTOCOL_VERSION_MINOR, 1));
        TtlvStructure header = structure(
                nested(PROTOCOL_VERSION, version),
                datetime(TIME_STAMP, 123456789L),
                integer(BATCH_COUNT, 1));
        TtlvStructure item = structure(
                enumeration(OPERATION, RAW_UNKNOWN),
                enumeration(RESULT_STATUS, RAW_UNKNOWN));
        return message(RESPONSE_HEADER, header, item);
    }

    private static TtlvStructure invalidRequestWithPayloadSentinel() {
        TtlvStructure version = structure(
                integer(PROTOCOL_VERSION_MAJOR, 2), integer(PROTOCOL_VERSION_MINOR, 1));
        TtlvStructure header = structure(nested(PROTOCOL_VERSION, version));
        TtlvStructure payload = structure(text(0x540002, "payload-secret-sentinel"));
        TtlvStructure item = structure(
                enumeration(OPERATION, RAW_UNKNOWN), nested(REQUEST_PAYLOAD, payload));
        return message(REQUEST_HEADER, header, item);
    }

    private static TtlvStructure message(int headerTag, TtlvStructure header, TtlvStructure item) {
        return structure(nested(headerTag, header), nested(BATCH_ITEM, item));
    }

    private static TtlvStructure structure(TtlvItem... items) {
        try (TtlvStructure empty = TtlvStructure.create()) {
            TtlvStructure value = empty;
            for (TtlvItem item : items) {
                value = TtlvStructure.withItem(value, item, LIMITS);
            }
            return value;
        }
    }

    private static TtlvItem nested(int tag, TtlvStructure structure) {
        return item(tag, TtlvValue.structure(structure, LIMITS));
    }

    private static TtlvItem integer(int tag, int value) {
        return item(tag, TtlvValue.Integer(value));
    }

    private static TtlvItem enumeration(int tag, long value) {
        return item(tag, TtlvValue.Enumeration(value));
    }

    private static TtlvItem bool(int tag, boolean value) {
        return item(tag, TtlvValue.Boolean(value));
    }

    private static TtlvItem datetime(int tag, long value) {
        return item(tag, TtlvValue.DateTime(value));
    }

    private static TtlvItem text(int tag, String value) {
        return item(tag, TtlvValue.TextString(value.getBytes(java.nio.charset.StandardCharsets.UTF_8), LIMITS));
    }

    private static TtlvItem item(int rawTag, TtlvValue value) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }
}
