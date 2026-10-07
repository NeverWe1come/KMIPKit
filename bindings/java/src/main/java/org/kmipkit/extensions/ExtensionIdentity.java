package org.kmipkit.extensions;

import java.nio.charset.StandardCharsets;
import java.util.Objects;
import java.util.regex.Pattern;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;

/** Immutable local identity for one vendor extension definition. */
public final class ExtensionIdentity {
    private static final Pattern VENDOR_PATTERN = Pattern.compile("[A-Za-z0-9_.]+");
    private final String vendorIdentifier;
    private final String name;
    private final String version;
    private final NativeHandle handle;

    private ExtensionIdentity(String vendorIdentifier, String name, String version, long nativeHandle) {
        this.vendorIdentifier = vendorIdentifier;
        this.name = name;
        this.version = version;
        this.handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.IDENTITY);
    }

    static ExtensionIdentity fromNative(long nativeHandle) {
        NativeExtensionRegistry.ensureLoaded();
        String[] fields = NativeExtensionRegistry.extensionIdentityFields(nativeHandle);
        if (fields == null || fields.length != 3) {
            throw new InvalidInputException("native identity metadata is invalid");
        }
        return new ExtensionIdentity(fields[0], fields[1], fields[2], nativeHandle);
    }

    public static ExtensionIdentity create(String vendorIdentifier, String name, String version) {
        requireVendor(vendorIdentifier);
        requireText(name, "extension name");
        requireText(version, "extension version");
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeExtensionRegistry.extensionIdentityCreate(vendorIdentifier, name, version);
        return new ExtensionIdentity(vendorIdentifier, name, version, nativeHandle);
    }

    private static void requireVendor(String value) {
        if (value == null || !VENDOR_PATTERN.matcher(value).matches()) {
            throw new InvalidExtensionIdentityException("vendor identification is invalid");
        }
        requireTextLength(value, "vendor identification");
    }

    private static void requireText(String value, String description) {
        if (value == null || value.isEmpty()) {
            throw new InvalidExtensionIdentityException(description + " is invalid");
        }
        requireTextLength(value, description);
    }

    private static void requireTextLength(String value, String description) {
        if (value.getBytes(StandardCharsets.UTF_8).length > 4_096) {
            throw new ResourceLimitException(description + " exceeds its configured limit");
        }
    }

    long handle() {
        return handle.get();
    }

    public String vendorIdentifier() {
        return vendorIdentifier;
    }

    public String name() {
        return name;
    }

    public String version() {
        return version;
    }

    @Override
    public boolean equals(Object other) {
        if (this == other) {
            return true;
        }
        if (!(other instanceof ExtensionIdentity identity)) {
            return false;
        }
        return vendorIdentifier.equals(identity.vendorIdentifier)
                && name.equals(identity.name)
                && version.equals(identity.version);
    }

    @Override
    public int hashCode() {
        return Objects.hash(vendorIdentifier, name, version);
    }

    @Override
    public String toString() {
        return "ExtensionIdentity[" + vendorIdentifier + ", " + name + ", " + version + "]";
    }
}
