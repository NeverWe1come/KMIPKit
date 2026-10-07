package org.kmipkit.extensions;

import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvItemType;
import org.kmipkit.ttlv.TtlvStructure;

/** Local KMIP §7.13 Extension Information metadata. */
public final class ExtensionInformation {
    private final NativeHandle handle;

    private ExtensionInformation(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.INFORMATION);
    }

    public static ExtensionInformation create(String extensionName) {
        requireText(extensionName);
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationCreate(extensionName));
    }

    public static ExtensionInformation with_tag(ExtensionInformation information, int tag) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithTag(
                information.handle.transfer(), tag));
    }

    public static ExtensionInformation with_type(ExtensionInformation information, TtlvItemType type) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithType(
                information.handle.transfer(), type.code()));
    }

    public static ExtensionInformation with_enumeration(ExtensionInformation information, int enumeration) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithEnumeration(
                information.handle.transfer(), enumeration));
    }

    public static ExtensionInformation with_attribute(ExtensionInformation information, boolean attribute) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithAttribute(
                information.handle.transfer(), attribute));
    }

    public static ExtensionInformation with_parent_structure_tag(
            ExtensionInformation information, int parentStructureTag) {
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithParentStructureTag(
                information.handle.transfer(), parentStructureTag));
    }

    public static ExtensionInformation with_description(ExtensionInformation information, String description) {
        requireText(description);
        NativeExtensionRegistry.ensureLoaded();
        return new ExtensionInformation(NativeExtensionRegistry.extensionInformationWithDescription(
                information.handle.transfer(), description));
    }

    public static TtlvStructure toTtlv(ExtensionInformation information) {
        NativeExtensionRegistry.ensureLoaded();
        return information.handle.withValue(value -> TtlvStructure.fromNative(
                NativeExtensionRegistry.extensionInformationToTtlv(value), information));
    }

    long handle() {
        return handle.get();
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    private static void requireText(String value) {
        if (value == null || value.isEmpty()) {
            throw new org.kmipkit.InvalidInputException("extension metadata text is invalid");
        }
        ExtensionText.requireWithinLimit(value, "extension metadata text");
    }
}
