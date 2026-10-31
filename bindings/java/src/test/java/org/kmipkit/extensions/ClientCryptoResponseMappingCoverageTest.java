package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;
import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class ClientCryptoResponseMappingCoverageTest {
    private static final byte[] IDENTIFIER = "server-key".getBytes(StandardCharsets.UTF_8);
    private static final byte[] OUTPUT = {0x11, 0x00, (byte) 0xee};
    private static final byte[] CORRELATION = {0x21, 0x00, (byte) 0xf1};

    @Test
    void successfulCryptoResponsesExposeCopiedTypedFields() {
        ClientBatchItemResult.HashResponse hash = ClientBatchItemResult.parseHashResponse(
                new Object[] {new long[] {1, 1, 1}, OUTPUT, CORRELATION}).orElseThrow();
        ClientBatchItemResult.MacResponse mac = ClientBatchItemResult.parseMacResponse(
                new Object[] {new long[] {1, 1, 1, 0, 0, 1, 1}, IDENTIFIER, OUTPUT, CORRELATION})
                .orElseThrow();
        ClientBatchItemResult.MacVerifyResponse macVerify = ClientBatchItemResult
                .parseMacVerifyResponse(new Object[] {
                    new long[] {1, 1, 1, 0, 0, 1, 0xe102_0304L, 1}, IDENTIFIER, CORRELATION
                }).orElseThrow();
        ClientBatchItemResult.SignResponse sign = ClientBatchItemResult.parseSignResponse(
                new Object[] {new long[] {1, 1, 1, 0, 0, 1, 1}, IDENTIFIER, OUTPUT, CORRELATION})
                .orElseThrow();
        ClientBatchItemResult.SignatureVerifyResponse signatureVerify = ClientBatchItemResult
                .parseSignatureVerifyResponse(new Object[] {
                    new long[] {1, 1, 1, 0, 0, 1, 0xf123_4567L, 1, 1},
                    IDENTIFIER, OUTPUT, CORRELATION
                }).orElseThrow();

        assertArrayEquals(OUTPUT, hash.data().orElseThrow());
        assertArrayEquals(CORRELATION, hash.correlationValue().orElseThrow());
        assertEquals("server-key", mac.uniqueIdentifier().text());
        assertArrayEquals(OUTPUT, mac.macData().orElseThrow());
        assertArrayEquals(CORRELATION, mac.correlationValue().orElseThrow());
        assertEquals(0xe102_0304L, macVerify.validityIndicatorRaw().orElseThrow());
        assertArrayEquals(CORRELATION, macVerify.correlationValue().orElseThrow());
        assertArrayEquals(OUTPUT, sign.signatureData().orElseThrow());
        assertArrayEquals(CORRELATION, sign.correlationValue().orElseThrow());
        assertEquals(0xf123_4567L, signatureVerify.validityIndicatorRaw().orElseThrow());
        assertArrayEquals(OUTPUT, signatureVerify.recoveredData().orElseThrow());
        assertArrayEquals(CORRELATION, signatureVerify.correlationValue().orElseThrow());

        byte[] callerCopy = mac.correlationValue().orElseThrow();
        callerCopy[0] = 0;
        assertArrayEquals(CORRELATION, mac.correlationValue().orElseThrow());
        assertFalse(mac.toString().contains("server-key"));
    }

    @Test
    void malformedNativeResponseFlagsAndAbsentFieldsAreRejected() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseHashResponse(
                        new Object[] {new long[] {2, 0, 0}, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseHashResponse(
                        new Object[] {new long[] {0, 1, 0}, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseHashResponse(
                        new Object[] {new long[] {0, 0, 0}, OUTPUT, null}));
    }

    @Test
    void malformedNativeIdentifiersAndUnsignedFieldsAreRejected() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacResponse(
                        new Object[] {new long[] {1, 0, 0, 0, 0, 0, 0}, null, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacResponse(
                        new Object[] {new long[] {1, 0, 1, 0, 0, 0, 0}, null, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacResponse(
                        new Object[] {new long[] {1, 1, 2, 0, 0, 0, 0}, IDENTIFIER, null, null}));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacVerifyResponse(new Object[] {
                    new long[] {1, 1, 1, 0, 0, 0, 1, 0}, IDENTIFIER, null
                }));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacVerifyResponse(new Object[] {
                    new long[] {1, 1, 1, 0, 0, 1, 0x1_0000_0000L, 0}, IDENTIFIER, null
                }));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseSignResponse(
                        new Object[] {new long[] {1, 1, 1, 0, 0, 0, 0}, IDENTIFIER, OUTPUT, null}));
    }
}
