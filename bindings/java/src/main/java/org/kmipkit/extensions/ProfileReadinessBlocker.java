package org.kmipkit.extensions;

/**
 * Redacted readiness blocker metadata with stable optional identifiers.
 *
 * @param kind open string identifying the blocker category
 * @param message fixed diagnostic text without caller payloads
 * @param requirementId optional catalog requirement identifier
 * @param profileId optional catalog profile identifier
 * @param testId optional catalog test identifier
 * @param sourceClauseId optional source-clause identifier
 */
public record ProfileReadinessBlocker(
        String kind,
        String message,
        String requirementId,
        String profileId,
        String testId,
        String sourceClauseId) {}
