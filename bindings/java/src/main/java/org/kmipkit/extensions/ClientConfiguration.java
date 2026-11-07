package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.InvalidInputException;
import org.kmipkit.internal.NativeHandle;
import java.util.List;

/** Client configuration retaining one immutable extension registry. */
public final class ClientConfiguration implements AutoCloseable {
    private final NativeHandle handle;
    private final List<ExtensionDefinition> definitions;
    private final ExtensionRegistryLimits limits;
    private final WireEncoding wireEncoding;

    private ClientConfiguration(long nativeHandle, List<ExtensionDefinition> definitions,
            ExtensionRegistryLimits limits, WireEncoding wireEncoding) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CONFIGURATION);
        this.definitions = List.copyOf(definitions);
        this.limits = limits;
        this.wireEncoding = wireEncoding;
    }

    public static ClientConfiguration create(ClientExtensionRegistry extensionRegistry) {
        NativeExtensionRegistry.ensureLoaded();
        List<ExtensionDefinition> definitions = extensionRegistry.definitionsSnapshot();
        ExtensionRegistryLimits limits = extensionRegistry.limits();
        long registryHandle = extensionRegistry.transferHandle();
        long nativeHandle = NativeExtensionRegistry.clientConfigurationCreate(registryHandle);
        WireEncoding encoding = WireEncoding.fromNative(
                NativeExtensionRegistry.clientConfigurationWireEncoding(nativeHandle));
        return new ClientConfiguration(nativeHandle, definitions, limits, encoding);
    }

    /** Returns a new immutable configuration using the selected default encoding. */
    public ClientConfiguration withWireEncoding(WireEncoding encoding) {
        if (encoding == null) {
            throw new InvalidInputException("wire encoding is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = handle.withValue(value ->
                NativeExtensionRegistry.clientConfigurationWithWireEncoding(value, encoding.nativeValue()));
        return new ClientConfiguration(nativeHandle, definitions, limits, encoding);
    }

    /** Returns the default encoding, which is TTLV for a newly created configuration. */
    public WireEncoding wireEncoding() {
        return wireEncoding;
    }

    public ClientExtensionRegistry extensionRegistry() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(value -> ClientExtensionRegistry.fromNative(
                NativeExtensionRegistry.clientConfigurationExtensionRegistry(value), definitions, limits));
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }
}
