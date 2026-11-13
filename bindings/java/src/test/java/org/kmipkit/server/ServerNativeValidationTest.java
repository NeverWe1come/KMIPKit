package org.kmipkit.server;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.InvalidInputException;

final class ServerNativeValidationTest {
    @Test
    void nativeConfigurationRejectsInvalidDiscriminatorsAndInputArrays() {
        assertThrows(InvalidInputException.class, () -> ServerNative.configurationCreate(
                -1, 1, null, -1, -1, null, null, null, null, null, -1));
        assertThrows(InvalidInputException.class, () -> ServerNative.configurationCreate(
                1, 1, "127.0.0.1", 0, 1, null, new byte[] {1},
                new byte[][] {{1}}, new byte[0][], "", 0));
        assertThrows(InvalidInputException.class, () -> ServerNative.configurationCreate(
                1, 1, "127.0.0.1", 0, 1, new byte[][] {null}, new byte[] {1},
                new byte[][] {{1}}, new byte[0][], "", 0));
        assertThrows(InvalidInputException.class, () -> ServerNative.configurationCreate(
                1, 1, "127.0.0.1", 0, 1, new byte[][] {{1}}, new byte[] {1},
                null, new byte[0][], "", 0));
    }

    @Test
    void nativeListenerAndConnectionMethodsRejectInvalidTimeouts() {
        assertThrows(InvalidInputException.class, () -> ServerNative.listenerAccept(0, -1, 0));
        assertThrows(InvalidInputException.class, () -> ServerNative.listenerAccept(0, 0, -1));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.listenerAccept(0, 0, 1_000_000_000));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.connectionReceive(0, new long[0], -1, 0));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.connectionReceive(0, new long[0], 0, 1_000_000_000));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.connectionSend(0, 0, -1, 0));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.connectionSend(0, 0, 0, 1_000_000_000));
    }

    @Test
    void nativeTypedReadersRejectInvalidHandlesAndIndexes() {
        assertThrows(InvalidInputException.class, () -> ServerNative.batchCount(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.batchItemAt(0, -1));
        assertThrows(InvalidInputException.class, () -> ServerNative.requestUniqueIdentifier(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.discoverVersionsCount(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.discoverVersionAt(0, -1));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.notifyDeletedAttributeReferenceCount(0));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.notifyDeletedAttributeReferenceAt(0, -1));
        assertThrows(InvalidInputException.class, () -> ServerNative.notifyAttributes(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.putObjectType(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.queryFunctionCount(0));
        assertThrows(InvalidInputException.class, () -> ServerNative.queryFunctionAt(0, -1));
        assertThrows(InvalidInputException.class, () -> ServerNative.endpointRole(0));
    }

    @Test
    void nativeResponseFactoriesRejectMalformedCollections() {
        assertThrows(InvalidInputException.class,
                () -> ServerNative.responseDiscoverVersions(0, null));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.responseDiscoverVersions(0, new int[][] {{2}}));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.responseDiscoverVersions(0, new int[][] {{256, 1}}));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.responseQuery(0, new int[] {-1}, null, null, null));
        assertThrows(InvalidInputException.class,
                () -> ServerNative.responseEndpointRole(0, -1, null));
        assertThrows(InvalidInputException.class, () -> ServerNative.responseNotify(0));
    }
}
