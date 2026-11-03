package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/** Caller-local pass/fail evidence for one stable requirement identifier. */
public record ProfileEvidenceRecord(String requirementId, boolean passed) {
    public ProfileEvidenceRecord {
        if (requirementId == null || requirementId.isBlank()) {
            throw new InvalidInputException("profile evidence record is invalid");
        }
    }

    public static ProfileEvidenceRecord passed(String requirementId) {
        return new ProfileEvidenceRecord(requirementId, true);
    }

    public static ProfileEvidenceRecord failed(String requirementId) {
        return new ProfileEvidenceRecord(requirementId, false);
    }
}
