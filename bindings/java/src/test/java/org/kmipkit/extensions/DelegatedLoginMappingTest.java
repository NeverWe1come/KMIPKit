package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ttlv.TtlvStructure;
import org.junit.jupiter.api.Test;

final class DelegatedLoginMappingTest {
    @Test
    void requestBuilderMapsGenericRightsAndEveryOptionalTable199Field() {
        try (TtlvStructure rights = TtlvStructure.create();
             TtlvStructure usageLimits = TtlvStructure.create();
             ClientBatchItem item = ClientBatchItem.delegatedLogin(
                     rights, 0xffff_ffffL, null, Integer.MIN_VALUE, usageLimits)) {
            assertTrue(item != null);
        }
    }

    @Test
    void requestBuilderMapsDateTimeAndAllowsOmittedOptionalFields() {
        try (TtlvStructure rights = TtlvStructure.create();
             ClientBatchItem dateTime = ClientBatchItem.delegatedLogin(
                     rights, null, Long.MIN_VALUE, null, null);
             ClientBatchItem omitted = ClientBatchItem.delegatedLogin(
                     rights, null, null, null, null)) {
            assertTrue(dateTime != null);
            assertTrue(omitted != null);
        }
    }

    @Test
    void requestBuilderRejectsConflictingOrOutOfRangeTypedValues() {
        try (TtlvStructure rights = TtlvStructure.create()) {
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.delegatedLogin(rights, -1L, null, null, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.delegatedLogin(rights, 0x1_0000_0000L,
                            null, null, null));
            assertThrows(InvalidInputException.class,
                    () -> ClientBatchItem.delegatedLogin(rights, 1L, 2L, null, null));
        }
    }

    @Test
    void responseMappingPreservesUnknownTicketTypeAndRedactsSecretValue() {
        NativeExtensionRegistry.ensureLoaded();
        long ticketTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] secret = "ticket-value-must-not-appear-in-text".getBytes();
        byte[] nativeSecret = secret.clone();

        try (ClientBatchItemResult.DelegatedLoginResponse response = ClientBatchItemResult
                .parseDelegatedLoginResponse(new Object[] {
                    new long[] {1, 1, 0xfeed_beefL}, nativeSecret,
                    new long[] {ticketTtlv}
                })
                .orElseThrow()) {
            assertEquals(0xfeed_beefL, response.ticketTypeRaw().orElseThrow());
            assertArrayEquals(secret, response.ticketValue().orElseThrow());
            assertTrue(response.ticketTtlv().orElseThrow().handle() > 0);
            assertFalse(response.toString().contains(new String(secret)));
            assertArrayEquals(new byte[secret.length], nativeSecret);
        }
    }

    @Test
    void responseMappingAllowsAValidFailureWithoutTicket() {
        ClientBatchItemResult.DelegatedLoginResponse response = ClientBatchItemResult
                .parseDelegatedLoginResponse(new Object[] {
                    new long[] {1, 0, 0}, null, new long[] {0}
                })
                .orElseThrow();

        try (response) {
            assertTrue(response.ticketTypeRaw().isEmpty());
            assertTrue(response.ticketValue().isEmpty());
            assertTrue(response.ticketTtlv().isEmpty());
            assertFalse(response.toString().contains("ticket-value"));
        }
    }

    @Test
    void responseMappingWipesSecretWhenNativeTupleValidationFails() {
        byte[] secret = "malformed-native-ticket-secret".getBytes();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseDelegatedLoginResponse(new Object[] {
                    new long[] {1, 1, 0xfeed_beefL}, secret, "invalid ticket handle"
                }));
        assertArrayEquals(new byte[secret.length], secret);
    }
}
