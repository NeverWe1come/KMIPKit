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
        long nativeHandle = NativeExtensionRegistry.extensionDefinitionCreate(
                identity.handle(), compatibility.handle(), discriminator.handle(), schema.handle());
        return new ExtensionDefinition(identity, compatibility, discriminator, schema, null, nativeHandle);
    }

    public static ValidatedExtensionValue validate(
            ExtensionDefinition definition, TtlvStructure value, CodecLimits limits) {
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.extensionDefinitionValidate(
                definition.handle(), value.handle(), NativeExtensionRegistry.codecLimitValues(limits));
        return new ValidatedExtensionValue(nativeHandle, definition.identity, value);
    }

    public static ExtensionDefinition withInformation(
            ExtensionDefinition definition, ExtensionInformation information) {
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.extensionDefinitionWithInformation(
                definition.handle.transfer(), information.handle());
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

    ExtensionIdentity identityValue() {
        return identity;
    }
}
