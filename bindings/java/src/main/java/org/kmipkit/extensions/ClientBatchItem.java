package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/** Typed request batch item with caller-ordered registry-validated extensions. */
public final class ClientBatchItem {
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

    public ClientBatchItem withExtension(ClientRequestMessageExtension extension) {
        NativeExtensionRegistry.ensureLoaded();
        long[] consumed = NativeHandle.transferAll(handle, extension.handleOwner());
        long nextHandle = NativeExtensionRegistry.clientBatchItemWithExtension(
                consumed[0], consumed[1]);
        ArrayList<ClientRequestMessageExtension> next = new ArrayList<>(extensions);
        next.add(extension);
        return new ClientBatchItem(nextHandle, next);
    }

    List<ClientRequestMessageExtension> extensions() {
        return extensions;
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
}
