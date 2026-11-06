package org.kmipkit.extensions;

import java.util.Arrays;
import java.util.List;
import java.util.OptionalLong;

import org.jspecify.annotations.Nullable;
import org.kmipkit.ClientOperationException;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.internal.NativeHandle;
import org.kmipkit.ttlv.CodecLimits;

/** Production KMIP client constructed from validated transport and extension configurations. */
public final class Client implements AutoCloseable {
    private final NativeHandle handle;

    private Client(long nativeHandle) {
        handle = NativeHandle.owned(nativeHandle, NativeExtensionRegistry.CLIENT);
    }

    /** Creates a client; construction validates configuration and performs no network exchange. */
    public static Client create(ClientConfiguration extensions, TransportConfiguration transport) {
        if (extensions == null || transport == null) {
            throw new InvalidInputException("client configuration is invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long nativeClient = NativeHandle.withNativeHandles(() -> {
            long extensionConfiguration = extensions.nativeHandleOwner().get();
            long transportConfiguration = transport.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientCreate(extensionConfiguration, transportConfiguration);
        });
        return new Client(nativeClient);
    }

    /** Executes one caller-ordered typed batch using the canonical TTLV codec limits. */
    public ClientBatchResponse execute(ClientBatch batch, CodecLimits limits)
            throws ClientOperationException {
        if (batch == null || limits == null) {
            throw new InvalidInputException("client batch inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientExecute(client, consumedBatch,
                    NativeExtensionRegistry.codecLimitValues(limits));
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    /** Executes one caller-ordered typed batch with exchange-local timeout overrides. */
    public ClientBatchResponse executeWithOptions(
            ClientBatch batch, CodecLimits limits, RequestOptions options)
            throws ClientOperationException {
        if (batch == null || limits == null || options == null) {
            throw new InvalidInputException("client batch inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long[] timeoutValues = timeoutValues(options);
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientExecuteWithOptions(
                    client, consumedBatch, NativeExtensionRegistry.codecLimitValues(limits), timeoutValues);
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    /** Executes exactly one exchange with an explicit profile selection. */
    public ClientBatchResponse executeWithProfiles(
            ClientBatch batch, CodecLimits limits, ProfileValidationSelection selection)
            throws ClientOperationException {
        return executeWithProfilesInternal(batch, limits, selection, null, null);
    }

    /** Executes exactly one profile-validated exchange with timeout overrides. */
    public ClientBatchResponse executeWithProfilesAndOptions(
            ClientBatch batch, CodecLimits limits, ProfileValidationSelection selection,
            RequestOptions options) throws ClientOperationException {
        return executeWithProfilesInternal(batch, limits, selection, null, options);
    }

    /** Executes exactly one profile-validated exchange with caller-owned object state. */
    public ClientBatchResponse executeWithProfilesAndContext(
            ClientBatch batch, CodecLimits limits, ProfileValidationSelection selection,
            ProfileValidationContext context) throws ClientOperationException {
        return executeWithProfilesInternal(batch, limits, selection, context, null);
    }

    /** Executes exactly one profile-validated exchange with context and timeout overrides. */
    public ClientBatchResponse executeWithProfilesAndContextAndOptions(
            ClientBatch batch, CodecLimits limits, ProfileValidationSelection selection,
            ProfileValidationContext context, RequestOptions options)
            throws ClientOperationException {
        return executeWithProfilesInternal(batch, limits, selection, context, options);
    }

    private ClientBatchResponse executeWithProfilesInternal(
            ClientBatch batch, CodecLimits limits, ProfileValidationSelection selection,
            @Nullable ProfileValidationContext context, @Nullable RequestOptions options)
            throws ClientOperationException {
        if (batch == null || limits == null || selection == null) {
            throw new InvalidInputException("profile validation inputs are invalid");
        }
        List<ProfileValidationContext.Entry> states = context == null
                ? List.of() : context.objectStates();
        long[] indexes = context == null ? null : new long[states.size()];
        String[] identifiers = context == null ? null : new String[states.size()];
        int[] knownFields = context == null ? null : new int[states.size()];
        int[] booleanValues = context == null ? null : new int[states.size()];
        long[] totals = context == null ? null : new long[states.size()];
        long[] counts = context == null ? null : new long[states.size()];
        for (int index = 0; index < states.size(); index++) {
            ProfileValidationContext.Entry entry = states.get(index);
            ProfileObjectState state = entry.state();
            indexes[index] = entry.batchItemIndex();
            identifiers[index] = state.uniqueIdentifier();
            Boolean[] facts = {
                    state.originalCreationDatePresent(), state.usageLimitsPresent(),
                    state.protectionUseStarted(), state.getUsageAllocationPerformed(),
                    state.getUsageAllocationChangedCount()};
            for (int bit = 0; bit < facts.length; bit++) {
                if (facts[bit] != null) {
                    knownFields[index] |= 1 << bit;
                    if (facts[bit]) booleanValues[index] |= 1 << bit;
                }
            }
            if (state.usageLimitsTotal() != null) {
                knownFields[index] |= 32;
                totals[index] = state.usageLimitsTotal();
            }
            if (state.usageLimitsCount() != null) {
                knownFields[index] |= 64;
                counts[index] = state.usageLimitsCount();
            }
        }
        RequestOptions exchangeOptions = options == null ? RequestOptions.defaults() : options;
        if (options != null && (options.connectTimeout() == null || options.writeTimeout() == null
                || options.readTimeout() == null || options.totalTimeout() == null)) {
            throw new InvalidInputException("profile validation options are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        String[] profileIds = selection.profileIds().toArray(String[]::new);
        long[] timeoutValues = timeoutValues(exchangeOptions);
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long consumedBatch = batch.nativeHandleOwner().transfer();
            return NativeExtensionRegistry.clientExecuteWithProfiles(
                    client, consumedBatch, NativeExtensionRegistry.codecLimitValues(limits),
                    profileIds, indexes, identifiers, knownFields, booleanValues, totals, counts,
                    timeoutValues);
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    /** Sends exactly one Poll with the caller-supplied opaque correlation bytes. */
    public AsyncOperationOutcome executePoll(byte[] correlationValue, CodecLimits limits)
            throws ClientOperationException {
        return executeCorrelationOperation(correlationValue, limits, true, null);
    }

    /** Sends exactly one Poll using the timeout overrides for this exchange. */
    public AsyncOperationOutcome executePollWithOptions(
            byte[] correlationValue, CodecLimits limits, RequestOptions options)
            throws ClientOperationException {
        if (options == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        return executeCorrelationOperation(correlationValue, limits, true, options);
    }

    /** Sends exactly one Cancel with the caller-supplied opaque correlation bytes. */
    public AsyncOperationOutcome executeCancel(byte[] correlationValue, CodecLimits limits)
            throws ClientOperationException {
        return executeCorrelationOperation(correlationValue, limits, false, null);
    }

    /** Sends exactly one Cancel using the timeout overrides for this exchange. */
    public AsyncOperationOutcome executeCancelWithOptions(
            byte[] correlationValue, CodecLimits limits, RequestOptions options)
            throws ClientOperationException {
        if (options == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        return executeCorrelationOperation(correlationValue, limits, false, options);
    }

    private AsyncOperationOutcome executeCorrelationOperation(
            byte[] correlationValue, CodecLimits limits, boolean poll, RequestOptions options)
            throws ClientOperationException {
        if (correlationValue == null || limits == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        byte[] staged = correlationValue.clone();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long[] codecLimits = NativeExtensionRegistry.codecLimitValues(limits);
            long outcome = handle.withValue(client -> {
                if (options != null) {
                    long[] requestTimeouts = timeoutValues(options);
                    return poll
                            ? NativeExtensionRegistry.clientExecutePollWithOptions(
                                    client, staged, codecLimits, requestTimeouts)
                            : NativeExtensionRegistry.clientExecuteCancelWithOptions(
                                    client, staged, codecLimits, requestTimeouts);
                }
                return poll
                        ? NativeExtensionRegistry.clientExecutePoll(client, staged, codecLimits)
                        : NativeExtensionRegistry.clientExecuteCancel(client, staged, codecLimits);
            });
            return new AsyncOperationOutcome(outcome);
        } finally {
            Arrays.fill(staged, (byte) 0);
        }
    }

    /** Sends exactly one Process request with an optional caller-selected asynchronous indicator. */
    public AsyncOperationOutcome executeProcess(
            byte[] correlationValue, CodecLimits limits, OptionalLong asynchronousIndicator)
            throws ClientOperationException {
        return executeProcessInternal(correlationValue, limits, asynchronousIndicator, null);
    }

    /** Sends one Process with timeout overrides and the caller-selected async indicator. */
    public AsyncOperationOutcome executeProcessWithOptions(
            byte[] correlationValue, CodecLimits limits, OptionalLong asynchronousIndicator,
            RequestOptions options) throws ClientOperationException {
        if (options == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        return executeProcessInternal(correlationValue, limits, asynchronousIndicator, options);
    }

    private AsyncOperationOutcome executeProcessInternal(
            byte[] correlationValue, CodecLimits limits, OptionalLong asynchronousIndicator,
            RequestOptions options) throws ClientOperationException {
        if (correlationValue == null || limits == null || asynchronousIndicator == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        long indicator = asynchronousIndicator.orElse(0);
        validateOptionalIndicator(asynchronousIndicator, indicator);
        byte[] staged = correlationValue.clone();
        try {
            NativeExtensionRegistry.ensureLoaded();
            long[] codecLimits = NativeExtensionRegistry.codecLimitValues(limits);
            long outcome = handle.withValue(client -> options == null
                    ? NativeExtensionRegistry.clientExecuteProcess(
                            client, staged, asynchronousIndicator.isPresent(), indicator, codecLimits)
                    : NativeExtensionRegistry.clientExecuteProcessWithOptions(
                            client, staged, asynchronousIndicator.isPresent(), indicator,
                            codecLimits, timeoutValues(options)));
            return new AsyncOperationOutcome(outcome);
        } finally {
            Arrays.fill(staged, (byte) 0);
        }
    }

    /** Sends one Query Asynchronous Requests with optional filters and a generic response payload. */
    public AsyncOperationOutcome executeQueryAsyncRequests(
            CodecLimits limits, @Nullable List<byte[]> correlationValues,
            @Nullable List<Long> operations, OptionalLong asynchronousIndicator)
            throws ClientOperationException {
        return executeQueryAsyncRequestsInternal(
                limits, correlationValues, operations, asynchronousIndicator, null);
    }

    /** Sends one Query Asynchronous Requests with timeout overrides and generic response TTLV. */
    public AsyncOperationOutcome executeQueryAsyncRequestsWithOptions(
            CodecLimits limits, @Nullable List<byte[]> correlationValues,
            @Nullable List<Long> operations, OptionalLong asynchronousIndicator,
            RequestOptions options) throws ClientOperationException {
        if (options == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        return executeQueryAsyncRequestsInternal(
                limits, correlationValues, operations, asynchronousIndicator, options);
    }

    private AsyncOperationOutcome executeQueryAsyncRequestsInternal(
            CodecLimits limits, @Nullable List<byte[]> correlationValues,
            @Nullable List<Long> operations, OptionalLong asynchronousIndicator,
            RequestOptions options) throws ClientOperationException {
        if (limits == null || asynchronousIndicator == null) {
            throw new InvalidInputException("asynchronous client inputs are invalid");
        }
        long indicator = asynchronousIndicator.orElse(0);
        validateOptionalIndicator(asynchronousIndicator, indicator);
        byte[][] staged = null;
        long[] operationValues = null;
        try {
            if (correlationValues != null) {
                if (correlationValues.size() > 100_000) {
                    throw new InvalidInputException("asynchronous filter count exceeds the configured limit");
                }
                staged = new byte[correlationValues.size()][];
                long total = 0;
                for (int index = 0; index < staged.length; index++) {
                    byte[] value = correlationValues.get(index);
                    if (value == null) {
                        throw new InvalidInputException("asynchronous correlation filter is invalid");
                    }
                    total += value.length;
                    if (total > 16 * 1024 * 1024L) {
                        throw new InvalidInputException("asynchronous filters exceed the configured limit");
                    }
                    staged[index] = value.clone();
                }
            }
            if (operations != null) {
                if (operations.size() > 100_000) {
                    throw new InvalidInputException("asynchronous filter count exceeds the configured limit");
                }
                operationValues = new long[operations.size()];
                for (int index = 0; index < operationValues.length; index++) {
                    Long value = operations.get(index);
                    if (value == null || value < 0 || value > 0xffff_ffffL) {
                        throw new InvalidInputException("KMIP operation filter is outside uint32");
                    }
                    operationValues[index] = value;
                }
            }
            NativeExtensionRegistry.ensureLoaded();
            byte[][] finalCorrelations = staged;
            long[] finalOperations = operationValues;
            long[] codecLimits = NativeExtensionRegistry.codecLimitValues(limits);
            long outcome = handle.withValue(client -> options == null
                    ? NativeExtensionRegistry.clientExecuteQueryAsyncRequests(
                            client, finalCorrelations, finalOperations,
                            asynchronousIndicator.isPresent(), indicator, codecLimits)
                    : NativeExtensionRegistry.clientExecuteQueryAsyncRequestsWithOptions(
                            client, finalCorrelations, finalOperations,
                            asynchronousIndicator.isPresent(), indicator,
                            codecLimits, timeoutValues(options)));
            return new AsyncOperationOutcome(outcome);
        } finally {
            if (staged != null) {
                for (byte[] value : staged) {
                    if (value != null) Arrays.fill(value, (byte) 0);
                }
            }
            if (operationValues != null) Arrays.fill(operationValues, 0);
        }
    }

    private static void validateOptionalIndicator(OptionalLong asynchronousIndicator, long indicator) {
        if (asynchronousIndicator.isPresent() && (indicator < 0 || indicator > 0xffff_ffffL)) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
    }

    /** Executes one caller-owned Encrypt, MAC, or Sign item under local Usage Allocation accounting. */
    public ClientBatchResponse executeGuardedProtectionItem(UsageAllocationContext context,
            ClientBatchItem item, long cost, CodecLimits limits) throws ClientOperationException {
        return executeGuardedProtectionItemInternal(context, item, cost, null, limits);
    }

    /** Executes one guarded protection item with the exact caller-selected KMIP Asynchronous Indicator. */
    public ClientBatchResponse executeGuardedProtectionItemWithAsynchronousIndicator(
            UsageAllocationContext context, ClientBatchItem item, long cost,
            long asynchronousIndicator, CodecLimits limits) throws ClientOperationException {
        if (asynchronousIndicator < 0 || asynchronousIndicator > 0xffff_ffffL) {
            throw new InvalidInputException("asynchronous indicator is outside uint32");
        }
        return executeGuardedProtectionItemInternal(
                context, item, cost, asynchronousIndicator, limits);
    }

    private ClientBatchResponse executeGuardedProtectionItemInternal(UsageAllocationContext context,
            ClientBatchItem item, long cost, Long asynchronousIndicator, CodecLimits limits)
            throws ClientOperationException {
        if (context == null || item == null || limits == null || cost <= 0) {
            throw new InvalidInputException("guarded protection inputs are invalid");
        }
        NativeExtensionRegistry.ensureLoaded();
        long response = NativeHandle.withNativeHandles(() -> {
            long client = handle.get();
            long allocation = context.nativeHandleOwner().get();
            long consumedItem = item.nativeHandleOwner().transfer();
            long[] codecLimits = NativeExtensionRegistry.codecLimitValues(limits);
            return asynchronousIndicator == null
                    ? NativeExtensionRegistry.clientExecuteGuardedProtectionItem(
                            client, allocation, consumedItem, cost, codecLimits)
                    : NativeExtensionRegistry.clientExecuteGuardedProtectionItemWithAsynchronousIndicator(
                            client, allocation, consumedItem, cost, asynchronousIndicator, codecLimits);
        });
        return new ClientBatchResponse(response, limits.maxElements(), limits.maxMessageBytes());
    }

    private static long[] timeoutValues(RequestOptions options) {
        TransportConfiguration.Timeout[] values = {
            options.connectTimeout(), options.writeTimeout(), options.readTimeout(), options.totalTimeout()
        };
        long[] encoded = new long[values.length * 3];
        for (int index = 0; index < values.length; index++) {
            TransportConfiguration.Timeout value = values[index];
            encoded[index * 3] = value.kind();
            encoded[index * 3 + 1] = value.seconds();
            encoded[index * 3 + 2] = value.nanoseconds();
        }
        return encoded;
    }

    @Override
    public void close() {
        handle.close();
    }
}
