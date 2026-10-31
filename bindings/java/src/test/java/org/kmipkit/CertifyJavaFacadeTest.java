package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertNotNull;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.UniqueIdentifier;

final class CertifyJavaFacadeTest {
    @Test
    void certifyBuilderPreservesOpenRequestTypeAndDoesNotChangeCallerBytes() {
        byte[] certificateRequest = {0x30, 0x00, 0x41, 0x42};
        byte[] original = certificateRequest.clone();

        try (ClientBatchItem item = ClientBatchItem.certify(
                UniqueIdentifier.textString("public-key-7"),
                0xfedc_ba98L,
                certificateRequest,
                null,
                null)) {
            assertNotNull(item);
        }

        assertArrayEquals(original, certificateRequest);
    }

    @Test
    void certifyBuilderAllowsIdentifierOnlyRequestWithoutRequestType() {
        try (ClientBatchItem item = ClientBatchItem.certify(
                UniqueIdentifier.integer(-17), null, null, null, null)) {
            assertNotNull(item);
        }
    }

    @Test
    void certifyBuilderAcceptsUnsignedIdentifierEnumeration() {
        try (ClientBatchItem item = ClientBatchItem.certify(
                UniqueIdentifier.enumeration(0xffff_ffffL), null, null, null, null)) {
            assertNotNull(item);
        }
    }
}
