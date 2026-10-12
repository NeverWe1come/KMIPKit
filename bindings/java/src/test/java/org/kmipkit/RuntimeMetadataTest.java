package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;

final class RuntimeMetadataTest {
    @Test
    void reportsSynchronizedProductAndAbiVersions() {
        assertEquals(new RuntimeMetadata.Version(1, 0, 0, 1), RuntimeMetadata.version());
    }

    @Test
    void reportsOnlyCapabilitiesWithReadyNativeMappings() {
        assertTrue(RuntimeMetadata.capabilityIsAvailable("generic_ttlv_model"));
        assertFalse(RuntimeMetadata.capabilityIsAvailable("unknown_capability"));
    }

    @Test
    void treatsUnknownCapabilityContainingNulAsUnavailable() {
        assertFalse(RuntimeMetadata.capabilityIsAvailable("unknown\u0000capability"));
    }

    @Test
    void treatsUnknownCapabilityContainingSupplementaryCodePointAsUnavailable() {
        assertFalse(RuntimeMetadata.capabilityIsAvailable("unknown-\uD83D\uDD10-capability"));
    }

    @Test
    void limitsCapabilityIdentifierByStandardUtf8ByteLength() {
        assertFalse(RuntimeMetadata.capabilityIsAvailable("🔐".repeat(32)));
        assertThrows(
                ResourceLimitException.class,
                () -> RuntimeMetadata.capabilityIsAvailable("🔐".repeat(33)));
    }

    @Test
    void exposesSafeNativeErrorSnapshotWithoutClearingItUntilRequested() {
        RuntimeMetadata.clearLastError();
        assertThrows(
                ResourceLimitException.class,
                () -> RuntimeMetadata.capabilityIsAvailable("x".repeat(129)));

        RuntimeMetadata.ErrorSnapshot expected = new RuntimeMetadata.ErrorSnapshot(6, 1, 10, 0);
        assertEquals(expected, RuntimeMetadata.lastErrorSnapshot());
        assertEquals(expected, RuntimeMetadata.lastErrorSnapshot());

        RuntimeMetadata.clearLastError();
        assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
    }

}
