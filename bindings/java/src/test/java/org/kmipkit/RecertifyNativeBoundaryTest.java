package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;

final class RecertifyNativeBoundaryTest {
    @Test
    void rejectsInvalidUniqueIdentifierBeforeCreatingNativeRequest() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRecertify(
                    99, null, 0, 0, null, 0, false, 0, null, false, 0, 0, 0));
    }

    @Test
    void rejectsNegativeCertificateRequestTypeAtNativeBoundary() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemRecertify(
                    0, null, 0, 0, null, 0, true, -1, null, false, 0, 0, 0));
    }

    @Test
    void rejectsNullNativeResponseHandle() {
        NativeExtensionRegistry.ensureLoaded();

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemResponseRecertify(0));
    }
}
