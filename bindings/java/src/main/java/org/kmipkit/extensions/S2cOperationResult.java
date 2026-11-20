package org.kmipkit.extensions;

import java.util.Objects;
import java.util.Optional;

/**
 * Lossless operation result returned by {@link ServerRequestHandler} callbacks.
 * Enforces KMIP Success/Failure reason invariants and carries optional status, reason, and message.
 */
public final class S2cOperationResult {
    private static final long MAX_UINT32 = 0xFFFF_FFFFL;
    private static final S2cOperationResult SUCCESS_SINGLETON = new S2cOperationResult(0L, null, null);

    private final long status;
    private final Long reason;
    private final String message;

    private S2cOperationResult(long status, Long reason, String message) {
        this.status = status;
        this.reason = reason;
        this.message = message;
    }

    /** Returns the canonical KMIP Success result (status 0, no reason, no message). */
    public static S2cOperationResult success() {
        return SUCCESS_SINGLETON;
    }

    /** Returns a failure result (status 1) with the required reason and optional message. */
    public static S2cOperationResult failure(long reason, String message) {
        return of(1L, reason, message);
    }

    /** Returns a failure result (status 1) with the required reason and no message. */
    public static S2cOperationResult failure(long reason) {
        return of(1L, reason, null);
    }

    /**
     * Creates an operation result and enforces KMIP Success/Failure reason invariants.
     *
     * @param status raw KMIP result status (0 = Success, 1 = Operation Failed, 2 = Operation Pending, etc.)
     * @param reason optional raw KMIP result reason (forbidden if status is 0, required if status is 1)
     * @param message optional untrusted message text
     * @return validated S2cOperationResult
     * @throws IllegalArgumentException if values are out of 32-bit range or violate reason invariants
     */
    public static S2cOperationResult of(long status, Long reason, String message) {
        if (status < 0 || status > MAX_UINT32) {
            throw new IllegalArgumentException("Result status out of uint32 range: " + status);
        }
        if (reason != null && (reason < 0 || reason > MAX_UINT32)) {
            throw new IllegalArgumentException("Result reason out of uint32 range: " + reason);
        }
        if (status == 0L && reason != null) {
            throw new IllegalArgumentException("Success status (0) forbids a result reason");
        }
        if (status == 1L && reason == null) {
            throw new IllegalArgumentException("Failure status (1) requires a result reason");
        }
        if (status == 0L && message == null) {
            return SUCCESS_SINGLETON;
        }
        return new S2cOperationResult(status, reason, message);
    }

    /** Returns the exact raw KMIP result status. */
    public long status() {
        return status;
    }

    /** Returns true if the status represents Success (0). */
    public boolean isSuccess() {
        return status == 0L;
    }

    /** Returns the optional raw KMIP result reason. */
    public Optional<Long> reason() {
        return Optional.ofNullable(reason);
    }

    /** Returns the optional message text. */
    public Optional<String> message() {
        return Optional.ofNullable(message);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        S2cOperationResult that = (S2cOperationResult) o;
        return status == that.status
                && Objects.equals(reason, that.reason)
                && Objects.equals(message, that.message);
    }

    @Override
    public int hashCode() {
        return Objects.hash(status, reason, message);
    }

    @Override
    public String toString() {
        return "S2cOperationResult{status=" + status + ", reason=" + reason + ", message=" + message + "}";
    }
}
