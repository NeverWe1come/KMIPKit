package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertThrows;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.Client;
import org.kmipkit.extensions.ClientBatch;
import org.kmipkit.extensions.ClientBatchItem;
import org.kmipkit.extensions.RequestOptions;
import org.kmipkit.extensions.TransportConfiguration;
import java.util.OptionalLong;

/** Exercises the shared Java client and ordered batch facade over the native ABI. */
final class ClientFacadeContractTest {
    @Test
    void requestOptionsKeepDefaultZeroAndUnboundedTimeoutsDistinct() {
        RequestOptions options = new RequestOptions(
                TransportConfiguration.Timeout.defaultValue(),
                TransportConfiguration.Timeout.bounded(java.time.Duration.ZERO),
                TransportConfiguration.Timeout.unbounded(),
                TransportConfiguration.Timeout.bounded(
                        java.time.Duration.ofSeconds(2).plusNanos(5)));

        assertEquals(0, options.connectTimeout().kind());
        assertEquals(1, options.writeTimeout().kind());
        assertEquals(2, options.readTimeout().kind());
        assertEquals(2, options.totalTimeout().seconds());
        assertEquals(5, options.totalTimeout().nanoseconds());
    }

    @Test
    void requestOptionsRejectsEveryMissingTimeoutValue() {
        TransportConfiguration.Timeout timeout = TransportConfiguration.Timeout.defaultValue();

        assertThrows(InvalidInputException.class,
                () -> new RequestOptions(null, timeout, timeout, timeout));
        assertThrows(InvalidInputException.class,
                () -> new RequestOptions(timeout, null, timeout, timeout));
        assertThrows(InvalidInputException.class,
                () -> new RequestOptions(timeout, timeout, null, timeout));
        assertThrows(InvalidInputException.class,
                () -> new RequestOptions(timeout, timeout, timeout, null));
    }

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
    void requestTimestampGetterPreservesAbsenceAndSignedBoundaries() {
        try (ClientBatch batch = ClientBatch.create()) {
            assertEquals(OptionalLong.empty(), batch.requestTimestamp());
            batch.requestTimestamp(Long.MIN_VALUE);
            assertEquals(OptionalLong.of(Long.MIN_VALUE), batch.requestTimestamp());
            batch.requestTimestamp(Long.MAX_VALUE);
            assertEquals(OptionalLong.of(Long.MAX_VALUE), batch.requestTimestamp());
        }
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
    void uniqueBatchItemIdRejectsNullAndOversizedInputsBeforeConsumingItem() {
        try (ClientBatchItem item = ClientBatchItem.discoverVersions()) {
            assertThrows(InvalidInputException.class, () -> item.withUniqueBatchItemId(null));
            assertThrows(ResourceLimitException.class,
                    () -> item.withUniqueBatchItemId(new byte[16 * 1024 * 1024 + 1]));
            assertEquals(0, item.extensionCount(), "rejected inputs must retain the item handle");
        }
    }

    @Test
    void nativeBatchSetterBoundaryRejectsNullHandlesAndInvalidUnsignedValues() {
        NativeExtensionRegistry.ensureLoaded();
        long batch = NativeExtensionRegistry.clientBatchCreate();
        try {
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(0, 0));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(batch, -1));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetAsynchronousIndicator(batch, 0x1_0000_0000L));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetRequestTimestamp(0, 1));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetOrderOption(0, true));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchAddErrorContinuationValue(batch, -1));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchAddErrorContinuationValue(batch, 0x1_0000_0000L));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetClientCorrelationValue(batch, null));
            assertThrows(InvalidInputException.class,
                    () -> NativeExtensionRegistry.clientBatchSetClientCorrelationValue(
                            batch, new byte[16 * 1024 * 1024 + 1]));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.CLIENT_BATCH, batch);
        }
    }

    @Test
    void nativeUniqueBatchIdBoundaryRejectsNullHandlesAndIds() {
        NativeExtensionRegistry.ensureLoaded();
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemWithUniqueBatchItemId(0, new byte[] {1}));

        long nullIdItem = NativeExtensionRegistry.clientBatchItemDiscoverVersions();
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.clientBatchItemWithUniqueBatchItemId(nullIdItem, null));

        long oversizedIdItem = NativeExtensionRegistry.clientBatchItemDiscoverVersions();
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.clientBatchItemWithUniqueBatchItemId(
                        oversizedIdItem, new byte[16 * 1024 * 1024 + 1]));
    }

    @Test
    void clientOperationExceptionsExposeStableRedactedMetadata() {
        ClientOperationException generic = new ClientOperationException(7, 91, 92, 2);
        assertEquals(7, generic.statusCode());
        assertEquals(91, generic.categoryCode());
        assertEquals(92, generic.causeCode());
        assertEquals(2, generic.deliveryStateCode());
        assertFalse(generic.getMessage().contains("secret-diagnostic-sentinel"));

        ProtocolException protocol = new ProtocolException(8, 2, 4, 3);
        TransportException transport = new TransportException(9, 3, 8, 2);
        ServerResultException server = new ServerResultException(10, 4, 0, 0);
        assertEquals(8, protocol.statusCode());
        assertEquals(2, protocol.categoryCode());
        assertEquals(9, transport.statusCode());
        assertEquals(3, transport.categoryCode());
        assertEquals(10, server.statusCode());
        assertEquals(4, server.categoryCode());
        assertEquals(0, server.deliveryStateCode());
    }
}
