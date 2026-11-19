package org.kmipkit.protocol;

import java.util.Optional;
import org.kmipkit.NativeExtensionRegistry;

/** OASIS KMIP version gating for operations, tags, and attribute mutations. */
public final class VersionGate {
    private VersionGate() {}

    public record VersionSpan(
            int introducedMajor,
            int introducedMinor,
            Optional<ProtocolVersion> deprecated,
            Optional<ProtocolVersion> removed) {}

    public static record ProtocolVersion(int major, int minor) {}

    public static boolean validateOperation(int major, int minor, long operation) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.versionGateValidateOperation(major, minor, operation);
    }

    public static boolean validateTag(int major, int minor, long tag) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.versionGateValidateTag(major, minor, tag);
    }

    public static boolean validateAttributeMutation(int major, int minor, long attributeTag) {
        NativeExtensionRegistry.ensureLoaded();
        return NativeExtensionRegistry.versionGateValidateAttributeMutation(major, minor, attributeTag);
    }

    public static Optional<VersionSpan> operationSpan(long operation) {
        NativeExtensionRegistry.ensureLoaded();
        int[] span = NativeExtensionRegistry.versionGateOperationSpan(operation);
        return parseSpan(span);
    }

    public static Optional<VersionSpan> tagSpan(long tag) {
        NativeExtensionRegistry.ensureLoaded();
        int[] span = NativeExtensionRegistry.versionGateTagSpan(tag);
        return parseSpan(span);
    }

    private static Optional<VersionSpan> parseSpan(int[] span) {
        if (span == null || span.length != 6) {
            return Optional.empty();
        }
        Optional<ProtocolVersion> deprecated = (span[2] == 0 && span[3] == 0)
                ? Optional.empty()
                : Optional.of(new ProtocolVersion(span[2], span[3]));
        Optional<ProtocolVersion> removed = (span[4] == 0 && span[5] == 0)
                ? Optional.empty()
                : Optional.of(new ProtocolVersion(span[4], span[5]));
        return Optional.of(new VersionSpan(span[0], span[1], deprecated, removed));
    }
}
