package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import java.util.List;

/** Client configuration retaining one immutable extension registry. */
public final class ClientConfiguration implements AutoCloseable {
    private final NativeHandle handle;
    private final List<ExtensionDefinition> definitions;
    private final ExtensionRegistryLimits limits;

    private ClientConfiguration(long nativeHandle, List<ExtensionDefinition> definitions,
            ExtensionRegistryLimits limits) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CONFIGURATION);
        this.definitions = List.copyOf(definitions);
        this.limits = limits;
    }

    public static ClientConfiguration create(ClientExtensionRegistry extensionRegistry) {
        NativeExtensionRegistry.ensureLoaded();
        List<ExtensionDefinition> definitions = extensionRegistry.definitionsSnapshot();
        ExtensionRegistryLimits limits = extensionRegistry.limits();
        long registryHandle = extensionRegistry.transferHandle();
        long nativeHandle = NativeExtensionRegistry.clientConfigurationCreate(registryHandle);
        return new ClientConfiguration(nativeHandle, definitions, limits);
    }

    public ClientExtensionRegistry extensionRegistry() {
        NativeExtensionRegistry.ensureLoaded();
        long registryHandle = NativeExtensionRegistry.clientConfigurationExtensionRegistry(handle.get());
        return ClientExtensionRegistry.fromNative(
                registryHandle, definitions, limits);
    }

    @Override
    public void close() {
        handle.close();
    }
}
