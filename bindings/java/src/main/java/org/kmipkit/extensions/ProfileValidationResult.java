package org.kmipkit.extensions;

import java.lang.ref.Cleaner;
import java.lang.ref.Reference;
import java.util.ArrayList;
import java.util.List;
import java.util.function.LongFunction;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.TtlvStructure;

/** Immutable, ordered identifiers and explicit validation operations for a resolved profile set. */
public final class ProfileValidationResult implements AutoCloseable {
    private static final Cleaner CLEANER = Cleaner.create();
    private static final int MAX_PROFILE_IDS = 4_096;
    private static final int MAX_REQUIREMENT_IDS = 100_000;

    private final State state;
    private final Cleaner.Cleanable cleanable;
    private final List<String> profileIds;
    private final List<String> requirementIds;

    ProfileValidationResult(long nativeHandle) {
        this(nativeHandle, NativeBridge.INSTANCE);
    }

    ProfileValidationResult(long nativeHandle, Bridge bridge) {
        if (nativeHandle == 0 || bridge == null) {
            throw new InvalidInputException("profile validation result is invalid");
        }
        state = new State(nativeHandle, bridge);
        cleanable = CLEANER.register(this, state);
        try {
            profileIds = snapshotIds(state, bridge::profileIdCount, bridge::profileIdAt, MAX_PROFILE_IDS);
            requirementIds = snapshotIds(
                    state, bridge::requirementIdCount, bridge::requirementIdAt, MAX_REQUIREMENT_IDS);
        } catch (RuntimeException | Error failure) {
            close();
            throw failure;
        }
    }

    /** Returns an immutable snapshot of selected profile identifiers in caller order. */
    public List<String> profileIds() {
        return profileIds;
    }

    /** Returns an immutable snapshot of resolved requirement identifiers in evaluator order. */
    public List<String> requirementIds() {
        return requirementIds;
    }

    /** Validates this profile set against the selected transport. */
    public void validateTransport(ProfileTransport transport) {
        if (transport == null) throw new InvalidInputException("profile transport is invalid");
        int rawValue = transport.abiValue();
        if (rawValue != 1 && rawValue != 2) {
            throw new InvalidInputException("profile transport is invalid");
        }
        state.withValue(handle -> {
            state.bridge.validateTransport(handle, rawValue);
            return null;
        });
    }

    /** Validates one request without caller-owned object state. */
    public void validateRequest(TtlvStructure request) {
        if (request == null) throw new InvalidInputException("profile request is invalid");
        withTtlvHandles(request, null, (result, requestHandle, ignored) -> {
            state.bridge.validateRequest(result, requestHandle);
            return null;
        });
    }

    /** Validates one request using caller-owned state for its exact batch items. */
    public void validateRequestWithContext(TtlvStructure request, ProfileValidationContext context) {
        if (request == null || context == null) {
            throw new InvalidInputException("profile request context is invalid");
        }
        withTtlvHandles(request, null, (result, requestHandle, ignored) -> {
            state.bridge.validateRequestWithContext(result, requestHandle, context);
            return null;
        });
    }

    /** Validates one response against its corresponding request. */
    public void validateResponse(TtlvStructure request, TtlvStructure response) {
        if (request == null || response == null) {
            throw new InvalidInputException("profile response inputs are invalid");
        }
        withTtlvHandles(request, response, (result, requestHandle, responseHandle) -> {
            state.bridge.validateResponse(result, requestHandle, responseHandle);
            return null;
        });
    }

    /** Validates one response using caller-owned state for its exact batch items. */
    public void validateResponseWithContext(
            TtlvStructure request, TtlvStructure response, ProfileValidationContext context) {
        if (request == null || response == null || context == null) {
            throw new InvalidInputException("profile response context is invalid");
        }
        withTtlvHandles(request, response, (result, requestHandle, responseHandle) -> {
            state.bridge.validateResponseWithContext(result, requestHandle, responseHandle, context);
            return null;
        });
    }

    private <T> T withTtlvHandles(
            TtlvStructure request, TtlvStructure response, TtlvOperation<T> operation) {
        NativeHandle[] handles = response == null
                ? new NativeHandle[] {request.nativeHandleOwner()}
                : new NativeHandle[] {request.nativeHandleOwner(), response.nativeHandleOwner()};
        return state.withValue(result -> NativeHandle.withValues(handles, values ->
                operation.apply(result, values[0], response == null ? 0 : values[1])));
    }

    private static List<String> snapshotIds(
            State state, LongFunction<Long> countReader, IdReader idReader, int maximumCount) {
        return state.withValue(handle -> {
            long count = countReader.apply(handle);
            if (count < 0) throw new InvalidInputException("profile validation result is invalid");
            if (count > maximumCount) {
                throw new ResourceLimitException("profile validation result exceeds the configured limit");
            }
            List<String> values = new ArrayList<>(Math.toIntExact(count));
            for (long index = 0; index < count; index++) {
                String value = idReader.read(handle, index);
                if (value == null) throw new InvalidInputException("profile validation result is invalid");
                values.add(value);
            }
            return List.copyOf(values);
        });
    }

    @Override
    public void close() {
        try {
            state.release();
        } finally {
            cleanable.clean();
        }
    }

    @Override
    public String toString() {
        return "ProfileValidationResult[profileCount=" + profileIds.size()
                + ", requirementCount=" + requirementIds.size() + "]";
    }

    private interface IdReader {
        String read(long handle, long index);
    }

    @FunctionalInterface
    private interface TtlvOperation<T> {
        T apply(long result, long request, long response);
    }

    interface Bridge {
        long profileIdCount(long handle);
        String profileIdAt(long handle, long index);
        long requirementIdCount(long handle);
        String requirementIdAt(long handle, long index);
        void validateTransport(long handle, int transport);
        void validateRequest(long handle, long request);
        void validateRequestWithContext(long handle, long request, ProfileValidationContext context);
        void validateResponse(long handle, long request, long response);
        void validateResponseWithContext(
                long handle, long request, long response, ProfileValidationContext context);
        void release(long handle);
    }

    private static final class NativeBridge implements Bridge {
        private static final NativeBridge INSTANCE = new NativeBridge();

        @Override public long profileIdCount(long handle) {
            return NativeExtensionRegistry.profileValidationResultProfileIdCount(handle);
        }
        @Override public String profileIdAt(long handle, long index) {
            return NativeExtensionRegistry.profileValidationResultProfileIdAt(handle, index);
        }
        @Override public long requirementIdCount(long handle) {
            return NativeExtensionRegistry.profileValidationResultRequirementIdCount(handle);
        }
        @Override public String requirementIdAt(long handle, long index) {
            return NativeExtensionRegistry.profileValidationResultRequirementIdAt(handle, index);
        }
        @Override public void validateTransport(long handle, int transport) {
            NativeExtensionRegistry.profileValidationResultValidateTransport(handle, transport);
        }
        @Override public void validateRequest(long handle, long request) {
            NativeExtensionRegistry.profileValidationResultValidateRequest(handle, request);
        }
        @Override public void validateRequestWithContext(
                long handle, long request, ProfileValidationContext context) {
            ProfileContextArrays arrays = ProfileContextArrays.from(context);
            NativeExtensionRegistry.profileValidationResultValidateRequestWithContext(
                    handle, request, arrays.indexes, arrays.identifiers, arrays.knownFields,
                    arrays.booleanValues, arrays.usageLimitsTotals, arrays.usageLimitsCounts);
        }
        @Override public void validateResponse(long handle, long request, long response) {
            NativeExtensionRegistry.profileValidationResultValidateResponse(handle, request, response);
        }
        @Override public void validateResponseWithContext(
                long handle, long request, long response, ProfileValidationContext context) {
            ProfileContextArrays arrays = ProfileContextArrays.from(context);
            NativeExtensionRegistry.profileValidationResultValidateResponseWithContext(
                    handle, request, response, arrays.indexes, arrays.identifiers, arrays.knownFields,
                    arrays.booleanValues, arrays.usageLimitsTotals, arrays.usageLimitsCounts);
        }
        @Override public void release(long handle) {
            NativeExtensionRegistry.profileValidationResultRelease(handle);
        }
    }

    private static final class ProfileContextArrays {
        private final long[] indexes;
        private final String[] identifiers;
        private final int[] knownFields;
        private final int[] booleanValues;
        private final long[] usageLimitsTotals;
        private final long[] usageLimitsCounts;

        private ProfileContextArrays(ProfileValidationContext context) {
            List<ProfileValidationContext.Entry> entries = context.objectStates();
            int count = entries.size();
            indexes = new long[count];
            identifiers = new String[count];
            knownFields = new int[count];
            booleanValues = new int[count];
            usageLimitsTotals = new long[count];
            usageLimitsCounts = new long[count];
            for (int index = 0; index < count; index++) {
                ProfileValidationContext.Entry entry = entries.get(index);
                ProfileObjectState value = entry.state();
                indexes[index] = entry.batchItemIndex();
                identifiers[index] = value.uniqueIdentifier();
                Boolean[] facts = {
                        value.originalCreationDatePresent(), value.usageLimitsPresent(),
                        value.protectionUseStarted(), value.getUsageAllocationPerformed(),
                        value.getUsageAllocationChangedCount()};
                for (int bit = 0; bit < facts.length; bit++) {
                    if (facts[bit] != null) {
                        knownFields[index] |= 1 << bit;
                        if (facts[bit]) booleanValues[index] |= 1 << bit;
                    }
                }
                if (value.usageLimitsTotal() != null) {
                    knownFields[index] |= 32;
                    usageLimitsTotals[index] = value.usageLimitsTotal();
                }
                if (value.usageLimitsCount() != null) {
                    knownFields[index] |= 64;
                    usageLimitsCounts[index] = value.usageLimitsCount();
                }
            }
        }

        private static ProfileContextArrays from(ProfileValidationContext context) {
            if (context == null) throw new InvalidInputException("profile context is invalid");
            return new ProfileContextArrays(context);
        }
    }

    private static final class State implements Runnable {
        private long handle;
        private final Bridge bridge;

        private State(long handle, Bridge bridge) {
            this.handle = handle;
            this.bridge = bridge;
        }

        private synchronized <T> T withValue(LongFunction<T> operation) {
            if (handle == 0) throw new InvalidInputException("native handle is closed");
            T result = operation.apply(handle);
            Reference.reachabilityFence(this);
            return result;
        }

        private void release() {
            long released;
            synchronized (this) {
                released = handle;
                handle = 0;
            }
            if (released != 0) bridge.release(released);
        }

        @Override
        public void run() {
            try {
                release();
            } catch (LinkageError | RuntimeException ignored) {
                // Cleaner threads cannot report native unload or shutdown failures to callers.
            }
        }
    }
}
