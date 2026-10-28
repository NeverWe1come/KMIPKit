package org.kmipkit.extensions;

import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;

/** Production KMIP client constructed from validated transport and extension configurations. */
public final class Client implements AutoCloseable {
    private final NativeHandle handle;

    private Client(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT);
    }

    /** Creates a client; construction validates configuration and performs no network exchange. */
    public static Client create(ClientConfiguration extensions, TransportConfiguration transport) {
        if (extensions == null || transport == null) {
            throw new InvalidInputException("client configuration is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeClient = NativeHandle.withNativeHandles(() -> {
            long extensionConfiguration = extensions.nativeHandleOwner().get();
            long transportConfiguration = transport.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientCreate(extensionConfiguration, transportConfiguration);
        });
        return new Client(nativeClient);
    }

    /** Executes one caller-ordered typed batch using the canonical TTLV codec limits. */
    public ClientBatchResponse execute(ClientBatch batch, CodecLimits limits)
            throws ClientOperationException {
        if (batch == null || limits == null) {
            throw new InvalidInputException("client batch inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientExecute(client, consumedBatch,
                    NativeExtensionRegistry.codecLimitValues(limits));
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    /** Executes one caller-ordered typed batch with exchange-local timeout overrides. */
    public ClientBatchResponse executeWithOptions(
            ClientBatch batch, CodecLimits limits, RequestOptions options)
            throws ClientOperationException {
        if (batch == null || limits == null || options == null) {
            throw new InvalidInputException("client batch inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] timeoutValues = timeoutValues(options);
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientExecuteWithOptions(
                    client, consumedBatch, NativeExtensionRegistry.codecLimitValues(limits), timeoutValues);
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    /** Executes one caller-owned Encrypt, MAC, or Sign item under local Usage Allocation accounting. */
    public ClientBatchResponse executeGuardedProtectionItem(UsageAllocationContext context,
            ClientBatchItem item, long cost, CodecLimits limits) throws ClientOperationException {
        return executeGuardedProtectionItemInternal(context, item, cost, null, limits);
    }

    /** Executes one guarded protection item with the exact caller-selected KMIP Asynchronous Indicator. */
    public ClientBatchResponse executeGuardedProtectionItemWithAsynchronousIndicator(
            UsageAllocationContext context, ClientBatchItem item, long cost,
            long asynchronousIndicator, CodecLimits limits) throws ClientOperationException {
        if (asynchronousIndicator < 0 || asynchronousIndicator > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        return executeGuardedProtectionItemInternal(
                context, item, cost, asynchronousIndicator, limits);
    }

    private ClientBatchResponse executeGuardedProtectionItemInternal(UsageAllocationContext context,
            ClientBatchItem item, long cost, Long asynchronousIndicator, CodecLimits limits)
            throws ClientOperationException {
        if (context == null || item == null || limits == null || cost <= 0) {
            throw new InvalidInputException("guarded protection inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long allocation = context.nativeHandleOwner().get();
            long consumedItem = item.nativeHandleOwner().transfer();
            long[] codecLimits = NativeExtensionRegistry.codecLimitValues(limits);
            return asynchronousIndicator == null
                    ? NativeExtensionRegistry.clientExecuteGuardedProtectionItem(
                            client, allocation, consumedItem, cost, codecLimits)
                    : NativeExtensionRegistry.clientExecuteGuardedProtectionItemWithAsynchronousIndicator(
                            client, allocation, consumedItem, cost, asynchronousIndicator, codecLimits);
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    private static long[] timeoutValues(RequestOptions options) {
        TransportConfiguration.Timeout[] values = {
            options.connectTimeout(), options.writeTimeout(), options.readTimeout(), options.totalTimeout()
        };
        long[] encoded = new long[values.length * 3];
        for (int index = 0; index < values.length; index++) {
            TransportConfiguration.Timeout value = values[index];
            encoded[index * 3] = value.kind();
            encoded[index * 3 + 1] = value.seconds();
            encoded[index * 3 + 2] = value.nanoseconds();
        }
        return encoded;
    }

    @Override
    public void close() {
        handle.close();
    }
}
