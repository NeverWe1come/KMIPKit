package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class ClientBatchItemResultValidationTest {
    @Test
    void parsesAbsentExportResponse() {
        Object[] nativeResult = {new long[] {0, 0, 0}, null, new long[] {0, 0, 0}};

        assertTrue(ClientBatchItemResult.parseExportResponse(nativeResult).isEmpty());
    }

    @Test
    void parsesAbsentMacVerifyValidityIndicator() {
        long[] fields = {1, 1, 1, 0, 0, 0, 0, 0};
        byte[] identifier = "key-id".getBytes(StandardCharsets.UTF_8);
        Object[] nativeResult = {fields, identifier, null};

        ClientBatchItemResult.MacVerifyResponse response =
                ClientBatchItemResult.parseMacVerifyResponse(nativeResult).orElseThrow();

        assertTrue(response.validityIndicatorRaw().isEmpty());
    }

    @Test
    void parsesAbsentSignatureVerifyValidityIndicator() {
        long[] fields = {1, 1, 1, 0, 0, 0, 0, 0, 0};
        byte[] identifier = "key-id".getBytes(StandardCharsets.UTF_8);
        Object[] nativeResult = {fields, identifier, null, null};

        ClientBatchItemResult.SignatureVerifyResponse response =
                ClientBatchItemResult.parseSignatureVerifyResponse(nativeResult).orElseThrow();

        assertTrue(response.validityIndicatorRaw().isEmpty());
    }

    @Test
    void rejectsAbsentOptionalUnsignedValueWhenNativeValueIsNonzero() {
        long[] fields = {1, 1, 1, 0, 0, 0, 7, 0};
        byte[] identifier = "key-id".getBytes(StandardCharsets.UTF_8);
        Object[] nativeResult = {fields, identifier, null};

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseMacVerifyResponse(nativeResult));
    }
}
