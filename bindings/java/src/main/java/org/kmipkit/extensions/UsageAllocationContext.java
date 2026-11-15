package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;

/**
 * Single-owner local accounting for one successful Get Usage Allocation response.
 *
 * <p>The counts describe only guarded calls routed through this context. They are not evidence of
 * remote consumption and do not account for requests made by other contexts or clients.
 */
public final class UsageAllocationContext implements AutoCloseable {
    private final NativeHandle handle;

    UsageAllocationContext(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.USAGE_ALLOCATION_CONTEXT);
    }

    NativeHandle nativeHandleOwner() {
        return handle;
    }

    /** Returns the caller-supplied raw Usage Limits Unit enumeration value. */
    public long usageLimitsUnit() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::usageAllocationContextUsageLimitsUnit);
    }

    /** Returns the exact positive Usage Limits Count supplied for the allocation request. */
    public long allocatedUnits() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::usageAllocationContextAllocatedUnits);
    }

    /** Returns units not currently reserved by guarded calls through this context. */
    public long availableUnits() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::usageAllocationContextAvailableUnits);
    }

    /** Returns units currently reserved by guarded calls through this context. */
    public long reservedUnits() {
        NativeExtensionRegistry.ensureLoaded();
        return handle.withValue(NativeExtensionRegistry::usageAllocationContextReservedUnits);
    }

    /** Returns the server-resolved identifier bound to this local accounting context. */
    public UniqueIdentifier uniqueIdentifier() {
        NativeExtensionRegistry.ensureLoaded();
        Object[] values = handle.withValue(
                NativeExtensionRegistry::usageAllocationContextUniqueIdentifier);
        if (values == null || values.length != 2 || !(values[0] instanceof long[] fields)
                || fields.length != 4 || fields[0] != 1) {
            throw new InvalidInputException("native usage allocation identifier is invalid");
        }
        byte[] text = null;
        if (values[1] != null) {
            if (!(values[1] instanceof byte[] bytes)) {
                throw new InvalidInputException("native usage allocation identifier is invalid");
            }
            text = bytes;
        }
        return UniqueIdentifier.fromNative(fields[1], text, fields[2], fields[3]);
    }

    @Override
    public void close() {
        handle.close();
    }

    @Override
    public String toString() {
        return "UsageAllocationContext[accounting=local, identifier=redacted]";
    }
}
