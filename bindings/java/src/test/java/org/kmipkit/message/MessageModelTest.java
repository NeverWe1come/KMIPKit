package org.kmipkit.message;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;
import static org.junit.jupiter.api.Assertions.assertArrayEquals;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;
import java.util.OptionalLong;
import java.util.concurrent.atomic.AtomicReference;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.RawTag;
import org.kmipkit.ttlv.Tag;
import org.kmipkit.ttlv.TtlvItem;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvItemView;
import org.kmipkit.ttlv.TtlvStructureView;
import org.kmipkit.ttlv.TtlvValueView;
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
    private static final int MAXIMUM_RESPONSE_SIZE = 0x420050;
    private static final int CLIENT_CORRELATION_VALUE = 0x420105;
    private static final int ATTESTATION_CAPABLE_INDICATOR = 0x4200D3;
    private static final int ATTESTATION_TYPE = 0x4200C7;
    private static final int AUTHENTICATION = 0x42000C;
    private static final int BATCH_COUNT = 0x42000D;
    private static final int EPHEMERAL = 0x420154;
    private static final int UNIQUE_BATCH_ITEM_ID = 0x420093;
    private static final int OPERATION = 0x42005C;
    private static final int REQUEST_PAYLOAD = 0x420079;
    private static final int TIME_STAMP = 0x420092;
    private static final int NONCE = 0x4200C8;
    private static final int SERVER_HASHED_PASSWORD = 0x420155;
    private static final int SERVER_CORRELATION_VALUE = 0x420106;
    private static final int RESULT_REASON = 0x42007E;
    private static final int RESULT_MESSAGE = 0x42007D;
    private static final int ASYNCHRONOUS_CORRELATION_VALUE = 0x420006;
    private static final int RESULT_STATUS = 0x42007F;
    private static final int RESPONSE_PAYLOAD = 0x42007C;
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
            assertEquals(OptionalInt.of(65536), header.maximumResponseSize());
            assertEquals(Optional.of("client-correlation"), header.clientCorrelationValue());
            assertEquals(OptionalLong.of(RAW_UNKNOWN), header.asynchronousIndicator());
            assertEquals(RAW_UNKNOWN, header.effectiveAsynchronousIndicator());
            assertEquals(Optional.of(true), header.attestationCapableIndicator());
            assertTrue(header.effectiveAttestationCapableIndicator());
            assertEquals(List.of(RAW_UNKNOWN, 7L), header.attestationTypes());
            assertEquals(Optional.of(1L), header.withAuthentication(TtlvStructureView::itemCount));
            assertTrue(header.batchOrderOption().isEmpty());
            assertTrue(header.effectiveBatchOrderOption());
            assertTrue(header.batchErrorContinuationOption().isEmpty());
            assertEquals(2, header.effectiveBatchErrorContinuationOption());
            assertTrue(header.timeStamp().isEmpty());

            assertEquals(OptionalLong.of(RAW_UNKNOWN), item.operation());
            assertEquals(Optional.of(true), item.ephemeral());
            assertArrayEquals(new byte[] {1, 0, 2}, item.uniqueBatchItemId().orElseThrow());
            assertEquals(Optional.of(1L), item.withRequestPayload(TtlvStructureView::itemCount));
            assertEquals(1, item.extensionCount());
            try (MessageExtensionView extension = item.extensionAt(0);
                    TtlvStructure value = extension.ttlv();
                    var view = value.view()) {
                assertEquals(3, view.itemCount());
                assertEquals(List.of(VENDOR_IDENTIFICATION, CRITICALITY_INDICATOR, VENDOR_EXTENSION),
                        tags(view));
                assertEquals(Optional.of("example"), extension.vendorIdentification());
                assertEquals(Optional.of(false), extension.criticalityIndicator());
                try (TtlvItemView vendorExtension = view.itemAt(2);
                        TtlvValueView vendorExtensionValue = vendorExtension.value();
                        TtlvStructureView vendorValue = vendorExtensionValue.structure()) {
                    assertEquals(VENDOR_TAG, tagAt(vendorValue, 0));
                }
                assertEquals(Optional.of(VENDOR_TAG), extension.withVendorExtension(
                        vendorValue -> tagAt(vendorValue, 0)));
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
            assertEquals(Optional.of(1L), header.withNonce(TtlvStructureView::itemCount));
            assertEquals(List.of(RAW_UNKNOWN, 11L), header.attestationTypes());
            assertEquals(Optional.of("client-response"), header.clientCorrelationValue());
            assertEquals(Optional.of("server-response"), header.serverCorrelationValue());

            AtomicReference<byte[]> hashedPasswordCopy = new AtomicReference<>();
            assertEquals(Optional.of(3), header.withServerHashedPassword(value -> {
                hashedPasswordCopy.set(value);
                assertArrayEquals(new byte[] {9, 8, 7}, value);
                return value.length;
            }));
            assertArrayEquals(new byte[] {0, 0, 0}, hashedPasswordCopy.get());

            assertEquals(OptionalLong.of(RAW_UNKNOWN), item.resultStatus());
            assertArrayEquals(new byte[] {4, 5}, item.uniqueBatchItemId().orElseThrow());
            assertEquals(OptionalLong.of(RAW_UNKNOWN), item.resultReason());
            assertEquals(Optional.of("result-message"), item.resultMessage());
            assertArrayEquals(new byte[] {0, 3, 1}, item.asynchronousCorrelationValue().orElseThrow());
            assertEquals(Optional.of(1L), item.withResponsePayload(TtlvStructureView::itemCount));
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
        assertTrue(error.fieldIndex().isEmpty());
        assertFalse(error.getMessage().contains("payload-secret-sentinel"));
        assertFalse(error.toString().contains("payload-secret-sentinel"));
    }

    private static TtlvStructure requestWithExtension() {
        TtlvStructure version = structure(
                integer(PROTOCOL_VERSION_MAJOR, 2), integer(PROTOCOL_VERSION_MINOR, 1));
        TtlvStructure header = structure(
                nested(PROTOCOL_VERSION, version),
                integer(MAXIMUM_RESPONSE_SIZE, 65536),
                text(CLIENT_CORRELATION_VALUE, "client-correlation"),
                enumeration(ASYNCHRONOUS_INDICATOR, RAW_UNKNOWN),
                bool(ATTESTATION_CAPABLE_INDICATOR, true),
                enumeration(ATTESTATION_TYPE, RAW_UNKNOWN),
                enumeration(ATTESTATION_TYPE, 7),
                nested(AUTHENTICATION, structure(integer(0x540010, 1))),
                integer(BATCH_COUNT, 1));

        TtlvStructure vendorValue = structure(integer(VENDOR_TAG, 47));
        TtlvStructure extension = structure(
                text(VENDOR_IDENTIFICATION, "example"),
                bool(CRITICALITY_INDICATOR, false),
                nested(VENDOR_EXTENSION, vendorValue));
        TtlvStructure payload = structure(integer(0x540002, 99));
        TtlvStructure item = structure(
                enumeration(OPERATION, RAW_UNKNOWN),
                bool(EPHEMERAL, true),
                byteString(UNIQUE_BATCH_ITEM_ID, new byte[] {1, 0, 2}),
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
                nested(NONCE, structure(integer(0x540020, 17))),
                byteString(SERVER_HASHED_PASSWORD, new byte[] {9, 8, 7}),
                enumeration(ATTESTATION_TYPE, RAW_UNKNOWN),
                enumeration(ATTESTATION_TYPE, 11),
                text(CLIENT_CORRELATION_VALUE, "client-response"),
                text(SERVER_CORRELATION_VALUE, "server-response"),
                integer(BATCH_COUNT, 1));
        TtlvStructure payload = structure(integer(0x540021, 19));
        TtlvStructure item = structure(
                enumeration(OPERATION, RAW_UNKNOWN),
                byteString(UNIQUE_BATCH_ITEM_ID, new byte[] {4, 5}),
                enumeration(RESULT_STATUS, RAW_UNKNOWN),
                enumeration(RESULT_REASON, RAW_UNKNOWN),
                text(RESULT_MESSAGE, "result-message"),
                byteString(ASYNCHRONOUS_CORRELATION_VALUE, new byte[] {0, 3, 1}),
                nested(RESPONSE_PAYLOAD, payload));
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
        TtlvStructure value = TtlvStructure.create();
        for (TtlvItem item : items) {
            value = TtlvStructure.withItem(value, item, LIMITS);
        }
        return value;
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

    private static TtlvItem byteString(int tag, byte[] value) {
        return item(tag, TtlvValue.ByteString(value, LIMITS));
    }

    private static TtlvItem item(int rawTag, TtlvValue value) {
        try (RawTag raw = RawTag.fromRaw(rawTag); Tag tag = raw.checked()) {
            return TtlvItem.create(tag, value, LIMITS);
        }
    }

    private static List<Integer> tags(TtlvStructureView structure) {
        List<Integer> tags = new ArrayList<>();
        for (long index = 0; index < structure.itemCount(); index++) {
            tags.add(tagAt(structure, index));
        }
        return tags;
    }

    private static int tagAt(TtlvStructureView structure, long index) {
        try (TtlvItemView item = structure.itemAt(index); var tag = item.tag()) {
            return tag.raw();
        }
    }
}
