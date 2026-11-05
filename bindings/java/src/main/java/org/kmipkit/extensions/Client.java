package org.kmipkit.extensions;

import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.jspecify.annotations.Nullable;

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
        return execute(batch, limits, null, false);
    }

    /** Executes one typed batch using an optional per-exchange encoding override. */
    public ClientBatchResponse executeWithOptions(ClientBatch batch, CodecLimits limits,
            @Nullable WireEncoding encodingOverride) throws ClientOperationException {
        return execute(batch, limits, encodingOverride, true);
    }

    private ClientBatchResponse execute(ClientBatch batch, CodecLimits limits,
            @Nullable WireEncoding encodingOverride, boolean useOptionsApi)
            throws ClientOperationException {
        if (batch == null || limits == null) {
            throw new InvalidInputException("client batch inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            long[] nativeLimits = NativeExtensionRegistry.codecLimitValues(limits);
            if (!useOptionsApi) {
                return NativeExtensionRegistry.clientExecute(client, consumedBatch, nativeLimits);
            }
            int nativeOverride = encodingOverride == null ? -1 : encodingOverride.nativeValue();
            return NativeExtensionRegistry.clientExecuteWithOptions(
                    client, consumedBatch, nativeLimits, nativeOverride);
        });
        return new ClientBatchResponse(response);
    }

    @Override
    public void close() {
        handle.close();
    }
}
