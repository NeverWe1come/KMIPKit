package org.kmipkit.extensions;

/** Redacted readiness blocker metadata with stable optional identifiers. */
public record ProfileReadinessBlocker(
        String kind,
        String message,
        String requirementId,
        String profileId,
        String testId,
        String sourceClauseId) {}
