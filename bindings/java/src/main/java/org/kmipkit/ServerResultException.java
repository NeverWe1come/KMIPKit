package org.kmipkit;

import java.util.OptionalLong;

/** A sanitized server-result error (failure or Pending) from typed client execution. */
public final class ServerResultException extends ClientOperationException {
    private static final long serialVersionUID = 1L;

    private final long resultStatus;
    private final long resultReasonValue;
    private final boolean hasResultReason;
    private final boolean pending;

    ServerResultException(int statusCode, int categoryCode, int causeCode, int deliveryStateCode) {
        super(statusCode, categoryCode, causeCode, deliveryStateCode);
        this.resultStatus = -1L;
        this.resultReasonValue = 0L;
        this.hasResultReason = false;
        this.pending = false;
    }

    private ServerResultException(long resultStatus, OptionalLong resultReason) {
        super(5, 4, 0, 4);
        this.resultStatus = resultStatus;
        this.hasResultReason = resultReason != null && resultReason.isPresent();
        this.resultReasonValue = this.hasResultReason ? resultReason.getAsLong() : 0L;
        this.pending = resultStatus == 2L;
    }

    /** Creates a sanitized {@link ServerResultException} from a non-zero server Result Status. */
    public static ServerResultException fromServerResult(long resultStatus, OptionalLong resultReason) {
        return new ServerResultException(resultStatus, resultReason);
    }

    /** Returns the raw KMIP Result Status (or {@code -1} when constructed without a metadata snapshot). */
    public long resultStatus() {
        return resultStatus;
    }

    /** Returns the optional raw KMIP Result Reason when present. */
    public OptionalLong resultReason() {
        return hasResultReason ? OptionalLong.of(resultReasonValue) : OptionalLong.empty();
    }

    /** Returns {@code true} when the server returned KMIP {@code Operation Pending} ({@code 2}). */
    public boolean isPending() {
        return pending;
    }

    /** Returns {@code true} when the server rejected the operation with a non-pending error status. */
    public boolean isServerRejected() {
        return !pending;
    }
}
