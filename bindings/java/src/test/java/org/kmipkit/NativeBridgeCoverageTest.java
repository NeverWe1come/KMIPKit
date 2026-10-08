package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertNotEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;

import java.nio.charset.StandardCharsets;

import org.junit.jupiter.api.Test;
import org.kmipkit.extensions.DuplicateExtensionKeyException;
import org.kmipkit.extensions.ExtensionCompatibilityException;
import org.kmipkit.extensions.InvalidExtensionIdentityException;
import org.kmipkit.ttlv.TtlvItemType;

final class NativeBridgeCoverageTest {
    private static final long[] CODEC_LIMITS = {16_777_216, 64, 100_000};
    private static final long[] REGISTRY_LIMITS = {
            256, 16_384, 256, 4_096, 1_048_576, 4_096,
            1_048_576, 256, 16_384, 200_000, 1_048_576, 64
    };

    @Test
    void nativeStringBridgeRoundTripsUnicodeAndRejectsInvalidJavaText() {
        String text = "two-byte é, three-byte €, four-byte 🔐";
        long identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", text, "1");
        try {
            assertEquals(text, NativeExtensionRegistry.extensionIdentityName(identity));
        } finally {
            NativeExtensionRegistry.release(NativeExtensionRegistry.IDENTITY, identity);
        }

        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", null, "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\ud800", "1"));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "\udc00", "1"));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "x".repeat(4_097), "1"));
    }

    @Test
    void nativeArrayReadersRejectNullMalformedAndOverLimitInputs() {
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(null, CODEC_LIMITS));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueTextString(new byte[0], null));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {1, 2}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.ttlvValueBigInteger(new byte[0], new long[] {-1, 2, 3}));
        assertThrows(ResourceLimitException.class,
                () -> NativeExtensionRegistry.ttlvValueByteString(new byte[2], new long[] {1, 1, 1}));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(null, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.extensionSchemaStructure(new long[] {0}, new long[0], false));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.codecLimitsCreate(-1, 1, 1));
        assertThrows(InvalidInputException.class,
                () -> NativeExtensionRegistry.release(-1, 1));
    }

    @Test
    void nativeRegistryLookupsAndOptionalInformationUseTypedResults() {
        long identity = 0;
        long compatibility = 0;
        long rootRawTag = 0;
        long rootTag = 0;
        long childRawTag = 0;
        long childTag = 0;
        long rootPath = 0;
        long path = 0;
        long scalar = 0;
        long discriminator = 0;
        long scalarSchema = 0;
        long childRule = 0;
        long childSchema = 0;
        long rootRule = 0;
        long schema = 0;
        long definition = 0;
        long registry = 0;
        long definitionAt = 0;
        long definitionForIdentity = 0;
        long codecLimits = 0;
        try {
            identity = NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "registry", "1");
            compatibility = NativeExtensionRegistry.compatibilityCreate(
                    2, 1, 2, 1, "0.0.0", "99.0.0");
            rootRawTag = NativeExtensionRegistry.rawTagCreate(0x420001);
            rootTag = NativeExtensionRegistry.rawTagChecked(rootRawTag);
            childRawTag = NativeExtensionRegistry.rawTagCreate(0x420002);
            childTag = NativeExtensionRegistry.rawTagChecked(childRawTag);
            rootPath = NativeExtensionRegistry.ttlvPathCreate(rootTag);
            path = NativeExtensionRegistry.ttlvPathWithChildTag(rootPath, childTag);
            scalar = NativeExtensionRegistry.ttlvValueTextString(
                    "marker".getBytes(StandardCharsets.UTF_8), CODEC_LIMITS);
            discriminator = NativeExtensionRegistry.discriminatorCreate(path, scalar);
            scalarSchema = NativeExtensionRegistry.extensionSchemaScalar(TtlvItemType.TextString.code());
            childRule = NativeExtensionRegistry.extensionChildRuleRequired(childTag, scalarSchema);
            childSchema = NativeExtensionRegistry.extensionSchemaStructure(
                    new long[] {childRule}, new long[0], false);
            rootRule = NativeExtensionRegistry.extensionChildRuleRequired(rootTag, childSchema);
            schema = NativeExtensionRegistry.extensionSchemaStructure(new long[] {rootRule}, new long[0], false);
            definition = NativeExtensionRegistry.extensionDefinitionCreate(
                    identity, compatibility, discriminator, schema);
            registry = NativeExtensionRegistry.clientExtensionRegistryCreate(
                    new long[] {definition}, REGISTRY_LIMITS);

            definitionAt = NativeExtensionRegistry.clientExtensionRegistryDefinitionAt(registry, 0);
            definitionForIdentity = NativeExtensionRegistry.clientExtensionRegistryDefinitionForIdentity(
                    registry, identity);
            assertNotEquals(0, definitionAt);
            assertNotEquals(0, definitionForIdentity);
            assertEquals(0, NativeExtensionRegistry.extensionDefinitionInformation(definition));
            codecLimits = NativeExtensionRegistry.codecLimitsCreate(1_024, 10, 100);
            assertNotEquals(0, codecLimits);

            long duplicateDefinition = definition;
            assertThrows(DuplicateExtensionKeyException.class,
                    () -> NativeExtensionRegistry.clientExtensionRegistryCreate(
                            new long[] {duplicateDefinition, duplicateDefinition}, REGISTRY_LIMITS));
            assertThrows(ExtensionCompatibilityException.class,
                    () -> NativeExtensionRegistry.compatibilityCreate(
                            2, 2, 2, 1, "0.0.0", "99.0.0"));
            assertThrows(InvalidExtensionIdentityException.class,
                    () -> NativeExtensionRegistry.extensionIdentityCreate("coverage.vendor", "", "1"));
        } finally {
            release(26, codecLimits);
            release(NativeExtensionRegistry.DEFINITION, definitionForIdentity);
            release(NativeExtensionRegistry.DEFINITION, definitionAt);
            release(NativeExtensionRegistry.REGISTRY, registry);
            release(NativeExtensionRegistry.DEFINITION, definition);
            release(NativeExtensionRegistry.SCHEMA, schema);
            release(NativeExtensionRegistry.CHILD_RULE, rootRule);
            release(NativeExtensionRegistry.SCHEMA, childSchema);
            release(NativeExtensionRegistry.CHILD_RULE, childRule);
            release(NativeExtensionRegistry.SCHEMA, scalarSchema);
            release(NativeExtensionRegistry.DISCRIMINATOR, discriminator);
            release(NativeExtensionRegistry.TTLV_VALUE, scalar);
            release(NativeExtensionRegistry.PATH, path);
            release(NativeExtensionRegistry.PATH, rootPath);
            release(NativeExtensionRegistry.TAG, childTag);
            release(NativeExtensionRegistry.RAW_TAG, childRawTag);
            release(NativeExtensionRegistry.TAG, rootTag);
            release(NativeExtensionRegistry.RAW_TAG, rootRawTag);
            release(NativeExtensionRegistry.COMPATIBILITY, compatibility);
            release(NativeExtensionRegistry.IDENTITY, identity);
        }
    }

    private static void release(int handleKind, long handle) {
        if (handle != 0) {
            NativeExtensionRegistry.release(handleKind, handle);
        }
    }
}
