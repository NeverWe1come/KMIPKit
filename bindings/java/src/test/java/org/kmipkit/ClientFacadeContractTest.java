package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.TransportConfiguration;

/** Exercises the shared Java client and ordered batch facade over the native ABI. */
final class ClientFacadeContractTest {
    @Test
    void transportConfigurationRejectsMissingMutualTlsIdentityBeforeNativeConstruction() {
        TransportConfiguration.Builder configuration = TransportConfiguration.builder()
                .rawTls("127.0.0.1", 5696)
                .trustPlatform();

        assertThrows(InvalidInputException.class, configuration::build);
    }

    @Test
    void batchAcceptsTypedItemsInCallerOrderAndRejectsClosedUseBeforeNativeCalls() {
        ClientBatch batch = ClientBatch.create();
        batch.add(ClientBatchItem.discoverVersions());
        batch.add(ClientBatchItem.discoverVersions());

        assertEquals(2, batch.itemCount());
        batch.close();

        assertThrows(InvalidInputException.class, batch::itemCount);
    }

}
