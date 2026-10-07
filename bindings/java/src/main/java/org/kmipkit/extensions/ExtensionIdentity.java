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
        if (nativeHandle == 0) {
            throw new InvalidInputException("native identity metadata is invalid");
        }
        try {
            String vendorIdentifier = NativeExtensionRegistry.extensionIdentityVendorIdentifier(nativeHandle);
            String name = NativeExtensionRegistry.extensionIdentityName(nativeHandle);
            String version = NativeExtensionRegistry.extensionIdentityVersion(nativeHandle);
            if (vendorIdentifier == null || name == null || version == null) {
                throw new InvalidInputException("native identity metadata is invalid");
            }
            return new ExtensionIdentity(vendorIdentifier, name, version, nativeHandle);
        } catch (RuntimeException | Error error) {
            NativeExtensionRegistry.release(NativeExtensionRegistry.IDENTITY, nativeHandle);
            throw error;
        }
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

    NativeHandle nativeHandleOwner() {
        return handle;
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
