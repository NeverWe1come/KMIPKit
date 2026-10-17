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
        return new ClientBatchResponse(response);
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
        return new ClientBatchResponse(response);
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
