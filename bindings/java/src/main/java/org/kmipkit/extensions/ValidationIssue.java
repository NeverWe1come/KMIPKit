package org.kmipkit.extensions;

import java.util.Objects;

/** Safe, value-free metadata for one locally detected validation issue. */
public final class ValidationIssue {
    private final String code;
    private final String path;
    private final String severity;
    private final String sourceIdentifier;
    private final String sourceReference;

    ValidationIssue(String[] fields) {
        if (fields == null || fields.length != 5) {
            throw new IllegalArgumentException("validation issue metadata is invalid");
        }
        code = Objects.requireNonNull(fields[0]);
        path = Objects.requireNonNull(fields[1]);
        severity = Objects.requireNonNull(fields[2]);
        sourceIdentifier = Objects.requireNonNull(fields[3]);
        sourceReference = Objects.requireNonNull(fields[4]);
    }

    public String code() { return code; }
    public String path() { return path; }
    public String severity() { return severity; }
    public String sourceIdentifier() { return sourceIdentifier; }
    public String sourceReference() { return sourceReference; }

    @Override
    public String toString() {
        return "ValidationIssue[code=" + code + ", path=" + path + ", severity=" + severity
                + ", sourceIdentifier=" + sourceIdentifier + ", sourceReference="
                + sourceReference + "]";
    }
}
