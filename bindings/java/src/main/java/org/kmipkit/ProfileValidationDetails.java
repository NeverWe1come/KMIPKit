package org.kmipkit;

import java.util.Optional;
import java.util.OptionalLong;

/** Safe, source-backed IDs describing a fail-closed profile validation error. */
public final class ProfileValidationDetails {
    private final String errorCode;
    private final String profileId;
    private final String requirementId;
    private final String sourceClauseId;
    private final String sourceGapId;
    private final String sourceGapLocator;
    private final boolean hasBatchItemIndex;
    private final long batchItemIndex;

    ProfileValidationDetails(
            String errorCode,
            String profileId,
            String requirementId,
            String sourceClauseId,
            String sourceGapId,
            String sourceGapLocator,
            boolean hasBatchItemIndex,
            long batchItemIndex) {
        if (errorCode == null || errorCode.isEmpty() || batchItemIndex < 0) {
            throw new IllegalArgumentException("profile validation details are invalid");
        }
        this.errorCode = errorCode;
        this.profileId = profileId;
        this.requirementId = requirementId;
        this.sourceClauseId = sourceClauseId;
        this.sourceGapId = sourceGapId;
        this.sourceGapLocator = sourceGapLocator;
        this.hasBatchItemIndex = hasBatchItemIndex;
        this.batchItemIndex = batchItemIndex;
    }

    /** Returns the stable machine-readable validation code. */
    public String errorCode() {
        return errorCode;
    }

    /** Returns the catalogued profile identifier, when the selection resolved. */
    public Optional<String> profileId() {
        return Optional.ofNullable(profileId);
    }

    /** Returns the source-backed requirement identifier, when present. */
    public Optional<String> requirementId() {
        return Optional.ofNullable(requirementId);
    }

    /** Returns the exact source clause identifier, when present. */
    public Optional<String> sourceClauseId() {
        return Optional.ofNullable(sourceClauseId);
    }

    /** Returns the exact source-gap identifier, when present. */
    public Optional<String> sourceGapId() {
        return Optional.ofNullable(sourceGapId);
    }

    /** Returns the catalogued locator for a source-gap identifier, when present. */
    public Optional<String> sourceGapLocator() {
        return Optional.ofNullable(sourceGapLocator);
    }

    /** Returns the zero-based request batch item index, when present. */
    public OptionalLong batchItemIndex() {
        return hasBatchItemIndex ? OptionalLong.of(batchItemIndex) : OptionalLong.empty();
    }
}
