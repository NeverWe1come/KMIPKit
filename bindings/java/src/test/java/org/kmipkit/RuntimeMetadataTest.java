package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
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
        assertFalse(RuntimeMetadata.capabilityIsAvailable("unknown\u0000capability"));
        assertFalse(RuntimeMetadata.capabilityIsAvailable("unknown-\uD83D\uDD10-capability"));
    }

    @Test
    void exposesSafeErrorSnapshotWithoutClearingItUntilRequested() {
        RuntimeMetadata.clearLastError();
        try {
            RuntimeMetadata.capabilityIsAvailable("x".repeat(129));
        } catch (ResourceLimitException expected) {
            // The safe snapshot is checked below; native details remain private.
        }

        RuntimeMetadata.ErrorSnapshot expected = new RuntimeMetadata.ErrorSnapshot(6, 1, 10, 0);
        assertEquals(expected, RuntimeMetadata.lastErrorSnapshot());
        assertEquals(expected, RuntimeMetadata.lastErrorSnapshot());

        RuntimeMetadata.clearLastError();
        assertEquals(new RuntimeMetadata.ErrorSnapshot(0, 0, 0, 0), RuntimeMetadata.lastErrorSnapshot());
    }
}
