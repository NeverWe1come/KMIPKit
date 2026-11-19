package org.kmipkit.extensions;

import org.kmipkit.InvalidInputException;

/**
 * Caller-local pass/fail evidence for one stable requirement identifier.
 *
 * @param requirementId stable catalog requirement identifier
 * @param passed whether the caller reports that the requirement passed
 */
public record ProfileEvidenceRecord(String requirementId, boolean passed) {
    public ProfileEvidenceRecord {
        if (requirementId == null) {
            throw new InvalidInputException("profile evidence record is invalid");
        }
    }

    /** Creates a caller evidence record marked as passing. */
    public static ProfileEvidenceRecord passed(String requirementId) {
        return new ProfileEvidenceRecord(requirementId, true);
    }

    /** Creates a caller evidence record marked as failing. */
    public static ProfileEvidenceRecord failed(String requirementId) {
        return new ProfileEvidenceRecord(requirementId, false);
    }
}
