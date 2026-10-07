package org.kmipkit.extensions;

import java.util.Optional;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;
import org.kmipkit.ttlv.TtlvStructure;

/** Complete data-only extension identity, compatibility, discriminator, and schema. */
public final class ExtensionDefinition {
    private final NativeHandle handle;
    private final ExtensionIdentity identity;
    private final Compatibility compatibility;
    private final Discriminator discriminator;
    private final ExtensionSchema schema;
    private final ExtensionInformation information;

    private ExtensionDefinition(ExtensionIdentity identity, Compatibility compatibility,
            Discriminator discriminator, ExtensionSchema schema, ExtensionInformation information,
            long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.DEFINITION);
        this.identity = identity;
        this.compatibility = compatibility;
        this.discriminator = discriminator;
        this.schema = schema;
        this.information = information;
    }

    public static ExtensionDefinition create(ExtensionIdentity identity, Compatibility compatibility,
            Discriminator discriminator, ExtensionSchema schema) {
        NativeExtensionRegistry.ensureLoaded();
        NativeHandle[] owners = {
                identity.nativeHandleOwner(), compatibility.nativeHandleOwner(),
                discriminator.nativeHandleOwner(), schema.nativeHandleOwner()
        };
        long nativeHandle = NativeHandle.withValues(owners, handles ->
                NativeExtensionRegistry.extensionDefinitionCreate(
                        handles[0], handles[1], handles[2], handles[3]));
        return new ExtensionDefinition(identity, compatibility, discriminator, schema, null, nativeHandle);
    }

    public static ValidatedExtensionValue validate(
            ExtensionDefinition definition, TtlvStructure value, CodecLimits limits) {
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.extensionDefinitionValidate(
                        definition.handle.get(), value.handle(),
                        NativeExtensionRegistry.codecLimitValues(limits)));
        return new ValidatedExtensionValue(nativeHandle, definition.identity, value);
    }

    public static ExtensionDefinition withInformation(
            ExtensionDefinition definition, ExtensionInformation information) {
        NativeExtensionRegistry.ensureLoaded();
        long definitionHandle = definition.handle.transfer();
        long nativeHandle = information.nativeHandleOwner().withValue(infoHandle ->
                NativeExtensionRegistry.extensionDefinitionWithInformation(definitionHandle, infoHandle));
        return new ExtensionDefinition(definition.identity, definition.compatibility,
                definition.discriminator, definition.schema, information, nativeHandle);
    }

    public static ExtensionIdentity identity(ExtensionDefinition definition) {
        definition.handle.get();
        return definition.identity;
    }

    public static Optional<ExtensionInformation> information(ExtensionDefinition definition) {
        definition.handle.get();
        return Optional.ofNullable(definition.information);
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    ExtensionIdentity identityValue() {
        return identity;
    }
}
