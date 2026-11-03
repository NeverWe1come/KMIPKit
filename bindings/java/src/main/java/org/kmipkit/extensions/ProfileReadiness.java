package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;
import org.kmipkit.ResourceLimitException;

/** Immutable, caller-local readiness snapshot from the canonical Rust evaluator. */
public final class ProfileReadiness {
    private static final int MAX_PROFILE_ID_BYTES = 4 * 1024;
    private static final long MAX_EVIDENCE_BYTES = 16L * 1024 * 1024;
    private static final int MAX_EVIDENCE_RECORDS = 100_000;

    private final String profileId;
    private final boolean targetSelected;
    private final boolean runtimeValidationAvailable;
    private final String effectiveState;
    private final boolean requirementRecordsComplete;
    private final boolean publishedClaim;
    private final List<ProfileReadinessBlocker> blockers;

    private ProfileReadiness(long handle) {
        try {
            String[] view = NativeExtensionRegistry.profileReadinessView(handle);
            profileId = view[0];
            effectiveState = view[1];
            targetSelected = Boolean.parseBoolean(view[2]);
            runtimeValidationAvailable = Boolean.parseBoolean(view[3]);
            requirementRecordsComplete = Boolean.parseBoolean(view[4]);
            publishedClaim = Boolean.parseBoolean(view[5]);
            int blockerCount = Integer.parseInt(view[6]);
            List<ProfileReadinessBlocker> values = new ArrayList<>(blockerCount);
            for (int index = 0; index < blockerCount; index++) {
                String[] item = NativeExtensionRegistry.profileReadinessBlockerAt(handle, index);
                values.add(new ProfileReadinessBlocker(
                        item[0], item[1], item[2], item[3], item[4], item[5]));
            }
            blockers = List.copyOf(values);
        } finally {
            NativeExtensionRegistry.profileReadinessRelease(handle);
        }
    }

    /** Evaluates local evidence. An unknown profile has no readiness value. */
    public static Optional<ProfileReadiness> evaluate(
            String profileId, List<ProfileEvidenceRecord> evidence) {
        if (profileId == null || evidence == null) {
            throw new InvalidInputException("profile readiness input is invalid");
        }
        int recordCount = evidence.size();
        if (recordCount > MAX_EVIDENCE_RECORDS) {
            throw new ResourceLimitException("profile evidence exceeds the configured limit");
        }
        long profileBytes = utf8LengthUpTo(profileId, MAX_PROFILE_ID_BYTES);
        if (profileBytes > MAX_PROFILE_ID_BYTES) {
            throw new ResourceLimitException("profile evidence exceeds the configured limit");
        }

        List<String> identifiers = new ArrayList<>(recordCount);
        boolean[] passed = new boolean[recordCount];
        long totalBytes = profileBytes;
        for (int index = 0; index < recordCount; index++) {
            ProfileEvidenceRecord record = evidence.get(index);
            if (record == null) throw new InvalidInputException("profile evidence input is invalid");
            String requirementId = record.requirementId();
            long remaining = MAX_EVIDENCE_BYTES - totalBytes;
            long requirementBytes = utf8LengthUpTo(requirementId, remaining);
            if (requirementBytes > remaining) {
                throw new ResourceLimitException("profile evidence exceeds the configured limit");
            }
            totalBytes += requirementBytes;
            identifiers.add(requirementId);
            passed[index] = record.passed();
        }
        long handle = NativeExtensionRegistry.profileReadinessCreate(
                profileId, identifiers.toArray(String[]::new), passed);
        return handle == 0 ? Optional.empty() : Optional.of(new ProfileReadiness(handle));
    }

    public String profileId() { return profileId; }
    public boolean targetSelected() { return targetSelected; }
    public boolean runtimeValidationAvailable() { return runtimeValidationAvailable; }
    public String effectiveState() { return effectiveState; }
    public boolean requirementRecordsComplete() { return requirementRecordsComplete; }
    public boolean publishedClaim() { return publishedClaim; }
    public List<ProfileReadinessBlocker> blockers() { return blockers; }

    private static long utf8LengthUpTo(String value, long maximum) {
        if (value.length() > maximum) return maximum + 1;
        long total = 0;
        for (int index = 0; index < value.length(); index++) {
            char character = value.charAt(index);
            int width;
            if (character <= 0x7f) {
                width = 1;
            } else if (character <= 0x7ff) {
                width = 2;
            } else if (Character.isHighSurrogate(character)
                    && index + 1 < value.length()
                    && Character.isLowSurrogate(value.charAt(index + 1))) {
                width = 4;
                index++;
            } else {
                width = 3;
            }
            total += width;
            if (total > maximum) return maximum + 1;
        }
        return total;
    }
}
