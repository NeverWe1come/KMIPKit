package org.kmipkit.credentials;

import java.util.concurrent.atomic.AtomicInteger;
import java.util.function.Function;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.ttlv.TtlvStructureView;

/** A validated Device Credential Value preserving its exact original TTLV structure. */
public final class DeviceCredential implements AutoCloseable {
    private final NativeHandle handle;
    private final AtomicInteger activeViews = new AtomicInteger();

    private DeviceCredential(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.DEVICE_CREDENTIAL);
    }

    /** Consume and validate a Device Credential Value Structure. */
    public static DeviceCredential fromTtlv(TtlvStructure structure) {
        if (structure == null) {
            throw new InvalidInputException("device credential structure is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        return NativeHandle.withNativeHandles(() -> {
            NativeHandle owner = structure.nativeHandleOwner();
            long[] outCredential = new long[1];
            int status = NativeExtensionRegistry.deviceCredentialFromTtlv(
                    owner.get(), outCredential);
            if (status != CredentialInterop.SHARED_OWNER_STATUS) {
                owner.transfer();
            }
            CredentialInterop.throwIfFailed(status, "device credential");
            return new DeviceCredential(outCredential[0]);
        });
    }

    /** Lend a read-only view of the original TTLV and close it when the callback returns. */
    public <T> T withTtlv(Function<TtlvStructureView, T> callback) {
        if (callback == null) {
            throw new InvalidInputException("device credential view callback is invalid");
        }
        return handle.withValue(value -> {
            TtlvStructureView view = TtlvStructureView.fromNative(
                    NativeExtensionRegistry.deviceCredentialTtlvView(value), this);
            activeViews.incrementAndGet();
            try {
                try (view) {
                    return callback.apply(view);
                }
            } finally {
                activeViews.decrementAndGet();
            }
        });
    }

    /** Transfer the original ordered TTLV Structure without rebuilding it. */
    public TtlvStructure intoTtlv() {
        if (activeViews.get() != 0) {
            throw new InvalidInputException("device credential is in use by a scoped view");
        }
        return TtlvStructure.fromNative(
                NativeExtensionRegistry.deviceCredentialIntoTtlv(handle.transfer()), this);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "DeviceCredential[redacted]";
    }
}
