package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.TransportConfiguration;

/** Exercises the shared Java client and ordered batch facade over the native ABI. */
final class ClientFacadeContractTest {
    @Test
    void clientCreationRejectsMissingConfigurationsBeforeLoadingNativeCode() {
        assertThrows(InvalidInputException.class, () -> Client.create(null, null));
    }

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

    @Test
    void asynchronousIndicatorRejectsValuesOutsideUint32BeforeNativeCalls() {
        try (ClientBatch batch = ClientBatch.create()) {
            assertThrows(InvalidInputException.class, () -> batch.asynchronousIndicator(-1));
            assertThrows(
                    InvalidInputException.class,
                    () -> batch.asynchronousIndicator(0x1_0000_0000L));
        }
    }

    @Test
    void batchMapsAllSharedRequestMetadataOptions() {
        try (ClientBatch batch = ClientBatch.create()) {
            batch.requestTimestamp(1_800_000_000L)
                    .ordered(true)
                    .addErrorContinuationValue(0)
                    .clientCorrelationValue(new byte[] {(byte) 0xc3, (byte) 0xa9});
        }
    }

    @Test
    void batchOptionsRejectInvalidUnsignedValuesAndNullCorrelationBytes() {
        try (ClientBatch batch = ClientBatch.create()) {
            assertThrows(
                    InvalidInputException.class,
                    () -> batch.addErrorContinuationValue(-1));
            assertThrows(
                    InvalidInputException.class,
                    () -> batch.addErrorContinuationValue(0x1_0000_0000L));
            assertThrows(
                    InvalidInputException.class,
                    () -> batch.clientCorrelationValue(null));
        }
    }

    @Test
    void clientCorrelationRequiresValidUtf8AndEnforcesTheInputLimit() {
        try (ClientBatch batch = ClientBatch.create()) {
            assertThrows(
                    InvalidInputException.class,
                    () -> batch.clientCorrelationValue(new byte[] {(byte) 0xc3, 0x28}));
            assertThrows(
                    ResourceLimitException.class,
                    () -> batch.clientCorrelationValue(new byte[16 * 1024 * 1024 + 1]));
        }
    }

    @Test
    void typedBatchItemCanCarryCallerSelectedUniqueBatchItemId() {
        try (ClientBatch batch = ClientBatch.create();
                ClientBatchItem item = ClientBatchItem.discoverVersions()
                        .withUniqueBatchItemId(new byte[] {0x01, 0x02, 0x03})) {
            batch.add(item);
            assertEquals(1, batch.itemCount());
        }
    }

    @Test
    void clientOperationExceptionsExposeStableRedactedMetadata() {
        ClientOperationException generic = new ClientOperationException(7, 91, 92, 2);
        assertEquals(7, generic.statusCode());
        assertEquals(91, generic.categoryCode());
        assertEquals(92, generic.causeCode());
        assertEquals(2, generic.deliveryStateCode());
        assertFalse(generic.getMessage().contains("secret-diagnostic-sentinel"));

        ServerResultException server = new ServerResultException(8, 3, 4, 0);
        TransportException transport = new TransportException(9, 5, 6, 2);
        assertEquals(8, server.statusCode());
        assertEquals(9, transport.statusCode());
    }
}
