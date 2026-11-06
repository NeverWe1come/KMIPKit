package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;

import org.jspecify.annotations.Nullable;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Owned, redacted validation results produced by KMIPKit's shared Rust validator. */
public final class ValidationReport implements AutoCloseable {
    private final NativeHandle handle;

    private ValidationReport(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.VALIDATION_REPORT);
    }

    static ValidationReport fromNativeHandle(long nativeHandle) {
        return new ValidationReport(nativeHandle);
    }

    /** Validates a generic Structure; a null schema never guesses an operation identity. */
    public static ValidationReport validateStructure(
            TtlvStructure structure, @Nullable String schema) {
        if (structure == null) {
            throw new InvalidInputException("a TTLV Structure is required");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeHandle = NativeHandle.withNativeHandles(() ->
                NativeExtensionRegistry.validationStructure(structure.handle(), schema));
        return new ValidationReport(nativeHandle);
    }

    /** Validates a typed request using the same Rust checks that run before client execution. */
    public static ValidationReport validateRequest(ClientBatchItem item) {
        if (item == null) {
            throw new InvalidInputException("a client batch item is required");
        }
        return item.validationReport();
    }

    /** Returns whether no error-severity finding exists. */
    public boolean isValid() {
        return handle.withValue(NativeExtensionRegistry::validationReportIsValid);
    }

    /** Returns a stable ordered snapshot of safe issue metadata. */
    public List<ValidationIssue> issues() {
        return handle.withValue(nativeReport -> {
            long count = NativeExtensionRegistry.validationReportIssueCount(nativeReport);
            if (count > Integer.MAX_VALUE) {
                throw new InvalidInputException("validation report exceeds the binding limit");
            }
            List<ValidationIssue> issues = new ArrayList<>((int) count);
            for (long index = 0; index < count; index++) {
                issues.add(new ValidationIssue(
                        NativeExtensionRegistry.validationReportIssueAt(nativeReport, index)));
            }
            return List.copyOf(issues);
        });
    }

    @Override
    public void close() {
        handle.close();
    }
}
