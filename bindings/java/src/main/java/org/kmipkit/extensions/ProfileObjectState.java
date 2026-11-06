package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/** Caller-attested state for one exact object in one request batch item. */
public record ProfileObjectState(
        String uniqueIdentifier,
        Boolean originalCreationDatePresent,
        Boolean usageLimitsPresent,
        Boolean protectionUseStarted,
        Boolean getUsageAllocationPerformed,
        Boolean getUsageAllocationChangedCount,
        Long usageLimitsTotal,
        Long usageLimitsCount) {
    public ProfileObjectState {
        if (uniqueIdentifier == null) {
            throw new InvalidInputException("profile object state is invalid");
        }
    }

    public ProfileObjectState(String uniqueIdentifier) {
        this(uniqueIdentifier, null, null, null, null, null, null, null);
    }

    public ProfileObjectState withOriginalCreationDatePresent(Boolean value) {
        return new ProfileObjectState(uniqueIdentifier, value, usageLimitsPresent,
                protectionUseStarted, getUsageAllocationPerformed,
                getUsageAllocationChangedCount, usageLimitsTotal, usageLimitsCount);
    }

    public ProfileObjectState withUsageLimitsPresent(Boolean value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent, value,
                protectionUseStarted, getUsageAllocationPerformed,
                getUsageAllocationChangedCount, usageLimitsTotal, usageLimitsCount);
    }

    public ProfileObjectState withProtectionUseStarted(Boolean value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent,
                usageLimitsPresent, value, getUsageAllocationPerformed,
                getUsageAllocationChangedCount, usageLimitsTotal, usageLimitsCount);
    }

    public ProfileObjectState withGetUsageAllocationPerformed(Boolean value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent,
                usageLimitsPresent, protectionUseStarted, value,
                getUsageAllocationChangedCount, usageLimitsTotal, usageLimitsCount);
    }

    public ProfileObjectState withGetUsageAllocationChangedCount(Boolean value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent,
                usageLimitsPresent, protectionUseStarted, getUsageAllocationPerformed,
                value, usageLimitsTotal, usageLimitsCount);
    }

    public ProfileObjectState withUsageLimitsTotal(Long value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent,
                usageLimitsPresent, protectionUseStarted, getUsageAllocationPerformed,
                getUsageAllocationChangedCount, value, usageLimitsCount);
    }

    public ProfileObjectState withUsageLimitsCount(Long value) {
        return new ProfileObjectState(uniqueIdentifier, originalCreationDatePresent,
                usageLimitsPresent, protectionUseStarted, getUsageAllocationPerformed,
                getUsageAllocationChangedCount, usageLimitsTotal, value);
    }

    @Override
    public String toString() {
        return "ProfileObjectState[uniqueIdentifier=[REDACTED], state=[REDACTED]]";
    }
}
