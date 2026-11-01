package org.kmipkit.credentials;

import java.util.ArrayList;
import java.util.List;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** Ordered KMIP Authentication preserving its original TTLV structure. */
public final class Authentication implements AutoCloseable {
    private static final int MAX_CREDENTIALS = 100_000;
    private final NativeHandle handle;

    private Authentication(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.AUTHENTICATION);
    }

    /** Consumes an ordered list of Credential handles, rejecting an empty Authentication natively. */
    public static Authentication create(List<Credential> credentials) {
        if (credentials == null) {
            throw new InvalidInputException("authentication credentials are invalid");
        }
        if (credentials.size() > MAX_CREDENTIALS) {
            throw new ResourceLimitException("configured resource limit was exceeded");
        }
        NativeExtensionRegistry.ensureLoaded();
        Credential[] values = credentials.toArray(Credential[]::new);
        NativeHandle[] owners = new NativeHandle[values.length];
        for (int index = 0; index < values.length; index++) {
            if (values[index] == null) {
                throw new InvalidInputException("authentication credentials are invalid");
            }
            owners[index] = values[index].nativeHandleOwner();
        }
        return NativeHandle.withValues(owners, nativeValues -> {
            long[] output = new long[1];
            int status = NativeExtensionRegistry.authenticationCreate(nativeValues, output);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                NativeHandle.transferAll(owners);
            }
            CredentialInterop.throwIfFailed(status, "authentication");
            return new Authentication(output[0]);
        });
    }

    /** Consumes and validates a TTLV Structure containing Authentication. */
    public static Authentication fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("authentication structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] output = new long[1];
            int status = NativeExtensionRegistry.authenticationFromTtlv(owner.get(), output);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "authentication");
            return new Authentication(output[0]);
        });
    }

    /** Return owner-backed Credential views in their original wire order. */
    public List<CredentialView> credentials() {
        return handle.withValue(value -> {
            long count = NativeExtensionRegistry.authenticationCredentialCount(value);
            if (count < 0 || count > Integer.MAX_VALUE) {
                throw new InvalidInputException("authentication credential count is invalid");
            }
            List<CredentialView> views = new ArrayList<>((int) count);
            try {
                for (long index = 0; index < count; index++) {
                    long nativeView = NativeExtensionRegistry.authenticationCredentialAt(value, index);
                    views.add(new CredentialView(nativeView, this));
                }
                return List.copyOf(views);
            } catch (RuntimeException exception) {
                views.forEach(CredentialView::close);
                throw exception;
            }
        });
    }

    /** Lends a read-only TTLV view and closes it when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("authentication view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.authenticationTtlvView(value), this);
            try (view) {
                return callback.apply(view);
            }
        });
    }

    /** Transfers the original ordered TTLV Structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        return handle.withValue(value -> {
            long[] output = new long[1];
            int status = NativeExtensionRegistry.authenticationIntoTtlv(value, output);
            // A retained native child view returns shared_owner before removing the owner.
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                handle.transfer();
            }
            CredentialInterop.throwIfFailed(status, "authentication");
            return TtlvStructure.fromNative(output[0], this);
        });
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "Authentication[redacted]";
    }
}
