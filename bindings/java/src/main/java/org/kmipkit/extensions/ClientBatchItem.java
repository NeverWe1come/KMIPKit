package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;

/** Typed request batch item with caller-ordered registry-validated extensions. */
public final class ClientBatchItem implements AutoCloseable {
    private static final int MAX_UNIQUE_BATCH_ITEM_ID_BYTES = 16 * 1024 * 1024;

    private final NativeHandle handle;
    private final List<ClientRequestMessageExtension> extensions;

    private ClientBatchItem(long nativeHandle, List<ClientRequestMessageExtension> extensions) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.BATCH_ITEM);
        this.extensions = List.copyOf(extensions);
    }

    /** Creates the supported Discover Versions request batch item. */
    public static ClientBatchItem discoverVersions() {
        NativeExtensionRegistry.ensureLoaded();
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemDiscoverVersions(), List.of());
    }

    /** Creates a Register request batch item and consumes the request handle. */
    public static ClientBatchItem register(RegisterRequest request) {
        NativeExtensionRegistry.ensureLoaded();
        if (request == null) {
            throw new InvalidInputException("Register request is invalid");
        }
        long consumed = NativeHandle.transferAll(request.nativeHandleOwner())[0];
        return new ClientBatchItem(NativeExtensionRegistry.clientBatchItemRegister(consumed), List.of());
    }

    public ClientBatchItem withExtension(ClientRequestMessageExtension extension) {
        NativeExtensionRegistry.ensureLoaded();
        long[] consumed = NativeHandle.transferAll(handle, extension.handleOwner());
        long nextHandle = NativeExtensionRegistry.clientBatchItemWithExtension(
                consumed[0], consumed[1]);
        ArrayList<ClientRequestMessageExtension> next = new ArrayList<>(extensions);
        next.add(extension);
        return new ClientBatchItem(nextHandle, next);
    }

    /** Returns a new item carrying the caller-selected opaque KMIP identifier bytes. */
    public ClientBatchItem withUniqueBatchItemId(byte[] identifier) {
        if (identifier == null) {
            throw new InvalidInputException("unique batch item ID is invalid");
        }
        if (identifier.length > MAX_UNIQUE_BATCH_ITEM_ID_BYTES) {
            throw new ResourceLimitException("unique batch item ID exceeds the configured limit");
        }
        byte[] copy = identifier.clone();
        NativeExtensionRegistry.ensureLoaded();
        long consumedItem = handle.transfer();
        long nextHandle = NativeExtensionRegistry.clientBatchItemWithUniqueBatchItemId(
                consumedItem, copy);
        return new ClientBatchItem(nextHandle, extensions);
    }

    List<ClientRequestMessageExtension> extensions() {
        return extensions;
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    /** Returns the number of attached vendor extensions. */
    public long extensionCount() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::clientBatchItemExtensionCount);
    }

    /** Returns a copied identity for the extension at {@code index}. */
    public ExtensionIdentity extensionIdentityAt(long index) {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> ExtensionIdentity.fromNative(
                NativeExtensionRegistry.clientBatchItemExtensionIdentityAt(value, index)));
    }

    /** Returns the extension's caller-selected Criticality Indicator. */
    public boolean extensionCriticalityIndicatorAt(long index) {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value ->
                NativeExtensionRegistry.clientBatchItemExtensionCriticalityIndicatorAt(value, index));
    }
    @Override
    public void close() {
        handle.close();
    }
}
