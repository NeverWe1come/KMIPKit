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

final class LoginMappingTest {
    @Test
    void requestBuilderMapsEveryOptionalTable253FieldAndAllowsOmissions() {
        try (TtlvStructure usageLimits = TtlvStructure.create();
             ClientBatchItem item = ClientBatchItem.login(
                     0xffff_ffffL, null, Integer.MIN_VALUE, usageLimits);
             ClientBatchItem omitted = ClientBatchItem.login(null, null, null, null)) {
            assertTrue(item != null);
            assertTrue(omitted != null);
        }
    }

    @Test
    void requestBuilderMapsDateTimeAndRejectsConflictingOrOutOfRangeValues() {
        try (ClientBatchItem dateTime = ClientBatchItem.login(null, Long.MIN_VALUE, null, null)) {
            assertTrue(dateTime != null);
        }
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.login(-1L, null, null, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.login(0x1_0000_0000L, null, null, null));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItem.login(1L, 2L, null, null));
    }

    @Test
    void responseMappingPreservesUnknownTicketTypeAndRedactsSecretValue() {
        NativeExtensionRegistry.ensureLoaded();
        long ticketTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] secret = "login-ticket-value-must-not-appear-in-text".getBytes();
        byte[] nativeSecret = secret.clone();

        try (ClientBatchItemResult.LoginResponse response = ClientBatchItemResult
                .parseLoginResponse(new Object[] {
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
    void responseMappingAllowsFailureWithoutTicket() {
        ClientBatchItemResult.LoginResponse response = ClientBatchItemResult
                .parseLoginResponse(new Object[] {
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
    void responseMappingRejectsInconsistentAbsentAndPresentTicketFields() {
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseLoginResponse(new Object[] {
                    new long[] {1, 0, 0}, null, new long[] {2}
                }));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseLoginResponse(new Object[] {
                    new long[] {0, 1, 3}, new byte[] {1}, new long[] {0}
                }));
        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseLoginResponse(new Object[] {
                    new long[] {1, 1, -1}, new byte[] {1}, new long[] {0}
                }));
    }

    @Test
    void responseMappingWipesSecretWhenNativeTupleValidationFails() {
        byte[] secret = "malformed-login-ticket-secret".getBytes();

        assertThrows(InvalidInputException.class,
                () -> ClientBatchItemResult.parseLoginResponse(new Object[] {
                    new long[] {1, 1, 0xfeed_beefL}, secret, "invalid ticket handle"
                }));
        assertArrayEquals(new byte[secret.length], secret);
    }

    @Test
    void responseMappingCleansNativeResultWhenParserEntryFails() {
        NativeExtensionRegistry.ensureLoaded();
        long ticketTtlv = NativeExtensionRegistry.ttlvStructureCreate();
        byte[] secret = "login-ticket-handoff-secret".getBytes();
        long[] ticketHandle = {ticketTtlv};

        AssertionError thrown = assertThrows(AssertionError.class,
                () -> ClientBatchItemResult.parseLoginResponseAfterNativeReturn(
                        new Object[] {
                            new long[] {1, 1, 0xfeed_beefL}, secret, ticketHandle
                        }, () -> {
                            throw new AssertionError("injected parser factory failure");
                        }));

        assertEquals("injected parser factory failure", thrown.getMessage());
        assertArrayEquals(new byte[secret.length], secret);
        assertArrayEquals(new long[] {0}, ticketHandle);
        TtlvStructure stale = TtlvStructure.fromNative(ticketTtlv, null);
        assertThrows(InvalidInputException.class, stale::view);
        stale.close();
    }

    @Test
    void nativeResponseBridgeRejectsAnInvalidItemHandle() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseLogin(0));
    }
}
