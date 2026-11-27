package org.kmipkit.extensions;

import java.util.List;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;

/** Client configuration retaining one immutable extension registry and embedded codec limits. */
public final class ClientConfiguration implements AutoCloseable {
    private final NativeHandle handle;
    private final List<ExtensionDefinition> definitions;
    private final ExtensionRegistryLimits limits;
    private final WireEncoding wireEncoding;
    private final ProtocolVersion protocolVersion;
    private final CodecLimits codecLimits;

    private ClientConfiguration(long nativeHandle, List<ExtensionDefinition> definitions,
            ExtensionRegistryLimits limits, WireEncoding wireEncoding,
            ProtocolVersion protocolVersion, CodecLimits codecLimits) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CONFIGURATION);
        this.definitions = List.copyOf(definitions);
        this.limits = limits;
        this.wireEncoding = wireEncoding;
        this.protocolVersion = protocolVersion;
        this.codecLimits = codecLimits;
    }

    /** Creates a default configuration with an empty extension registry and default codec limits. */
    public static ClientConfiguration defaults() {
        try (ClientExtensionRegistry registry = ClientExtensionRegistry.create(
                List.of(), ExtensionRegistryLimits.defaults())) {
            return create(registry);
        }
    }

    public static ClientConfiguration create(ClientExtensionRegistry extensionRegistry) {
        NativeExtensionRegistry.ensureLoaded();
        List<ExtensionDefinition> definitions = extensionRegistry.definitionsSnapshot();
        ExtensionRegistryLimits limits = extensionRegistry.limits();
        long registryHandle = extensionRegistry.transferHandle();
        long nativeHandle = NativeExtensionRegistry.clientConfigurationCreate(registryHandle);
        WireEncoding encoding = WireEncoding.fromNative(
                NativeExtensionRegistry.clientConfigurationWireEncoding(nativeHandle));
        return new ClientConfiguration(nativeHandle, definitions, limits, encoding,
                new ProtocolVersion(2, 1), CodecLimits.defaults());
    }

    /** Returns a new immutable configuration using the selected default encoding. */
    public ClientConfiguration withWireEncoding(WireEncoding encoding) {
        if (encoding == null) {
            throw new InvalidInputException("wire encoding is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = handle.withValue(value ->
                NativeExtensionRegistry.clientConfigurationWithWireEncoding(value, encoding.nativeValue()));
        return new ClientConfiguration(
                nativeHandle, definitions, limits, encoding, protocolVersion, codecLimits);
    }

    /** Returns a new immutable configuration using the selected KMIP protocol version. */
    public ClientConfiguration withProtocolVersion(ProtocolVersion version) {
        if (version == null) {
            throw new InvalidInputException("protocol version is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = handle.withValue(value ->
                NativeExtensionRegistry.clientConfigurationWithProtocolVersion(
                        value, version.major(), version.minor()));
        return new ClientConfiguration(
                nativeHandle, definitions, limits, wireEncoding, version, codecLimits);
    }

    /** Returns a new immutable configuration using the selected default codec limits. */
    public ClientConfiguration withCodecLimits(CodecLimits nextLimits) {
        if (nextLimits == null) {
            throw new InvalidInputException("codec limits are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] values = NativeExtensionRegistry.codecLimitValues(nextLimits);
        long nativeHandle = handle.withValue(value ->
                NativeExtensionRegistry.clientConfigurationWithCodecLimits(value, values));
        return new ClientConfiguration(
                nativeHandle, definitions, limits, wireEncoding, protocolVersion, nextLimits);
    }

    /** Returns the default encoding, which is TTLV for a newly created configuration. */
    public WireEncoding wireEncoding() {
        return wireEncoding;
    }

    /** Returns the configured protocol version, which defaults to KMIP 2.1. */
    public ProtocolVersion protocolVersion() {
        return protocolVersion;
    }

    /** Returns the configured codec limits, which default to {@link CodecLimits#defaults()}. */
    public CodecLimits codecLimits() {
        return codecLimits;
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
