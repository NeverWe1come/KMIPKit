package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Optional;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Immutable per-client vendor extension registry. */
public final class ClientExtensionRegistry implements AutoCloseable {
    private final NativeHandle handle;
    private final List<ExtensionDefinition> definitions;
    private final ExtensionRegistryLimits limits;

    private ClientExtensionRegistry(long nativeHandle, List<ExtensionDefinition> definitions,
            ExtensionRegistryLimits limits) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.REGISTRY);
        ArrayList<ExtensionDefinition> sorted = new ArrayList<>(definitions);
        sorted.sort(Comparator.comparing(definition -> definition.identityValue().toString()));
        this.definitions = List.copyOf(sorted);
        this.limits = limits;
    }

    public static ClientExtensionRegistry create(
            List<ExtensionDefinition> definitions, ExtensionRegistryLimits limits) {
        List<ExtensionDefinition> snapshot = List.copyOf(definitions);
        NativeExtensionRegistry.ensureLoaded();
        NativeHandle[] owners = snapshot.stream()
                .map(ExtensionDefinition::nativeHandleOwner)
                .toArray(NativeHandle[]::new);
        long nativeHandle = NativeHandle.withValues(owners, handles ->
                NativeExtensionRegistry.clientExtensionRegistryCreate(handles, limits.nativeValues()));
        return new ClientExtensionRegistry(nativeHandle, snapshot, limits);
    }

    static ClientExtensionRegistry fromNative(long nativeHandle,
            List<ExtensionDefinition> definitions, ExtensionRegistryLimits limits) {
        return new ClientExtensionRegistry(nativeHandle, definitions, limits);
    }

    public ExtensionRecognition inspect(String vendorIdentifier, TtlvStructure value, CodecLimits codecLimits) {
        NativeExtensionRegistry.ensureLoaded();
        long recognition = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientExtensionRegistryInspect(
                        handle.get(), vendorIdentifier, value.handle(),
                        NativeExtensionRegistry.codecLimitValues(codecLimits)));
        return new ExtensionRecognition(recognition, this);
    }

    public long definitionCount() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(registryHandle -> {
            NativeExtensionRegistry.clientExtensionRegistryDefinitionCount(registryHandle);
            return definitions.size();
        });
    }

    public Optional<ExtensionDefinition> definitionAt(long index) {
        handle.get();
        if (index < 0 || index >= definitions.size()) {
            return Optional.empty();
        }
        return Optional.of(definitions.get(Math.toIntExact(index)));
    }

    public RegisteredExtensionValue validateExtensionValue(
            ExtensionIdentity identity, TtlvStructure value, CodecLimits codecLimits) {
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.clientExtensionRegistryValidate(
                        handle.get(), identity.handle(), value.handle(),
                        NativeExtensionRegistry.codecLimitValues(codecLimits)));
        return new RegisteredExtensionValue(nativeHandle, identity, this, value);
    }

    public static Optional<ExtensionDefinition> definitionForIdentity(
            ClientExtensionRegistry registry, ExtensionIdentity identity) {
        return NativeHandle.withValues(new NativeHandle[] {
                registry.handle, identity.nativeHandleOwner()
        }, ignored -> registry.definitions.stream()
                        .filter(definition -> definition.identityValue().equals(identity))
                        .findFirst());
    }

    public ExtensionRegistryLimits limits() {
        handle.get();
        return limits;
    }

    List<ExtensionDefinition> definitionsSnapshot() {
        handle.get();
        return definitions;
    }

    long handle() {
        return handle.get();
    }

    long transferHandle() {
        return handle.transfer();
    }

    @Override
    public void close() {
        handle.close();
    }
}
