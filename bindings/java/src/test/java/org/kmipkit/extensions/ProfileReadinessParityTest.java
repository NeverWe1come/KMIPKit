package org.kmipkit.extensions;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertNotNull;
import static org.junit.jupiter.api.Assertions.assertNull;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

import java.util.List;
import java.util.Optional;
import org.kmipkit.ResourceLimitException;
import org.junit.jupiter.api.Test;

final class ProfileReadinessParityTest {
    private static final String BASELINE = "KMIPKIT-PROFILE-BASELINE-CLIENT";

    @Test
    void baselineReadinessUsesCanonicalEvaluatorWithoutPublishingClaim() {
        ProfileCatalog.ClientProfile profile = ProfileCatalog.clientProfiles().stream()
                .filter(item -> item.profileId().equals(BASELINE)).findFirst().orElseThrow();
        List<ProfileEvidenceRecord> evidence = profile.requirementIds().stream()
                .map(ProfileEvidenceRecord::passed).toList();

        ProfileReadiness readiness = ProfileReadiness.evaluate(BASELINE, evidence).orElseThrow();

        assertEquals(BASELINE, readiness.profileId());
        assertTrue(readiness.targetSelected());
        assertFalse(readiness.runtimeValidationAvailable());
        assertEquals("evidence_incomplete", readiness.effectiveState());
        assertTrue(readiness.requirementRecordsComplete());
        assertFalse(readiness.publishedClaim());
        assertTrue(readiness.blockers().stream()
                .anyMatch(item -> item.kind().equals("verification_manifest_unavailable")));
        assertTrue(ProfileReadiness.evaluate("KMIPKIT-PROFILE-UNKNOWN", List.of()).isEmpty());
    }

    @Test
    void readinessBlockersExposeStableMessagesAndPresenceTaggedIdentifiers() {
        ProfileReadiness readiness = ProfileReadiness.evaluate(BASELINE, List.of(
                ProfileEvidenceRecord.failed("KMIPKIT-REQ-PROF-4.1.2-001"),
                ProfileEvidenceRecord.passed("KMIPKIT-REQ-PROF-4.1.2-001"),
                ProfileEvidenceRecord.failed("UNKNOWN-CALLER-MARKER"))).orElseThrow();

        assertTrue(readiness.blockers().stream()
                .anyMatch(item -> item.kind().equals("duplicate_requirement_record")));
        assertTrue(readiness.blockers().stream()
                .anyMatch(item -> item.kind().equals("unknown_requirement_record")));
        assertTrue(readiness.blockers().stream().allMatch(item -> !item.message().isBlank()));
        assertTrue(readiness.blockers().stream().noneMatch(item -> item.message().contains("UNKNOWN-CALLER-MARKER")));
        ProfileReadinessBlocker duplicate = readiness.blockers().stream()
                .filter(item -> item.kind().equals("duplicate_requirement_record"))
                .findFirst().orElseThrow();
        assertEquals("KMIPKIT-REQ-PROF-4.1.2-001", duplicate.requirementId());
        assertNull(duplicate.profileId());
        assertNull(duplicate.testId());
        assertNull(duplicate.sourceClauseId());

        ProfileReadinessBlocker unknown = readiness.blockers().stream()
                .filter(item -> item.kind().equals("unknown_requirement_record"))
                .findFirst().orElseThrow();
        assertNull(unknown.requirementId());
        assertNull(unknown.profileId());
        assertNull(unknown.testId());
        assertNull(unknown.sourceClauseId());
        assertTrue(readiness.blockers().stream()
                .filter(item -> item.kind().equals("missing_requirement_record"))
                .allMatch(item -> item.requirementId() != null && item.profileId() == null
                        && item.testId() == null && item.sourceClauseId() == null));
    }

    @Test
    void catalogSnapshotsExposeAllRowsAndNestedValuesAsImmutableOrderedLists() {
        List<ProfileCatalog.ClientProfile> profiles = ProfileCatalog.clientProfiles();
        List<ProfileCatalog.ClientRequirement> requirements = ProfileCatalog.clientRequirements();
        List<ProfileCatalog.ProfileTestCase> testCases = ProfileCatalog.profileTestCases();

        assertEquals(18, profiles.size());
        assertEquals(483, requirements.size());
        assertEquals(93, testCases.size());
        assertEquals(profiles.stream().map(ProfileCatalog.ClientProfile::profileId).sorted().toList(),
                profiles.stream().map(ProfileCatalog.ClientProfile::profileId).toList());
        assertEquals(requirements.stream().map(ProfileCatalog.ClientRequirement::requirementId).sorted().toList(),
                requirements.stream().map(ProfileCatalog.ClientRequirement::requirementId).toList());
        assertEquals(testCases.stream().map(ProfileCatalog.ProfileTestCase::testId).sorted().toList(),
                testCases.stream().map(ProfileCatalog.ProfileTestCase::testId).toList());
        assertEquals(5, java.util.Arrays.stream(ProfileCatalog.VerificationCategory.values()).count());
        assertThrows(UnsupportedOperationException.class, () -> profiles.clear());

        ProfileCatalog.ClientProfile baseline = profiles.stream()
                .filter(item -> item.profileId().equals(BASELINE)).findFirst().orElseThrow();
        assertEquals("evidence_incomplete", baseline.claimState());
        assertTrue(baseline.selectedTarget());
        assertNotNull(baseline.name());
        assertNotNull(baseline.applicability());
        assertNotNull(baseline.directSourceClauseIds());
        assertNotNull(baseline.sourceClauseIds());
        assertNotNull(baseline.requirementIds());
        assertNotNull(baseline.dependencyProfileIds());
        assertNotNull(baseline.transportRequirements());
        assertNotNull(baseline.encodingRequirements());
        assertNotNull(baseline.unresolvedScopeObligations());
        assertNotNull(baseline.testCases());
        assertTrue(profiles.stream().allMatch(item -> item.directSourceClauseIds() != null
                && item.sourceClauseIds() != null && item.requirementIds() != null
                && item.dependencyProfileIds() != null && item.transportRequirements() != null
                && item.encodingRequirements() != null && item.unresolvedScopeObligations() != null
                && item.testCases() != null));
        assertTrue(requirements.stream().allMatch(item -> item.requirementId() != null
                && item.sourceKeyword() != null && item.sourceForce() != null && item.sourceGapId() != null
                && item.sourceLocator() != null && item.verificationCategory() != null
                && item.scopeState() != null && item.sourceClauseIds() != null && item.profileIds() != null
                && item.officialTestCaseIds() != null));
        assertTrue(requirements.stream().anyMatch(item -> item.sourceKeyword().isPresent()));
        assertTrue(requirements.stream().anyMatch(item -> item.sourceForce().isPresent()));
        assertTrue(requirements.stream().anyMatch(item -> item.sourceGapId().isPresent()));
        assertTrue(requirements.stream().anyMatch(item -> item.sourceLocator().isPresent()));
        assertEquals(List.of("KMIPKIT-REQ-SPEC-4.57-005-002"), requirements.stream()
                .filter(ProfileCatalog.ClientRequirement::hasAcceptedDeviation)
                .map(ProfileCatalog.ClientRequirement::requirementId).toList());
        assertThrows(UnsupportedOperationException.class, () -> baseline.requirementIds().clear());
        assertEquals(java.util.Set.of(
                ProfileCatalog.VerificationCategory.REQUIRED_BEHAVIOR,
                ProfileCatalog.VerificationCategory.PROHIBITED_BEHAVIOR,
                ProfileCatalog.VerificationCategory.RECOMMENDED_BEHAVIOR_OR_DEVIATION,
                ProfileCatalog.VerificationCategory.DISCOURAGED_ABSENCE_OR_DEVIATION,
                ProfileCatalog.VerificationCategory.OPTIONAL_CAPABILITY),
                java.util.Set.of(ProfileCatalog.VerificationCategory.values()));
        assertTrue(testCases.stream().allMatch(item -> item.testId() != null
                && item.officialCaseId() != null && item.mandatoryStatus() != null
                && item.fixtureAvailability() != null));
        assertNull(requirements.get(0).sourceGapId().orElse(null));
    }

    @Test
    void readinessRejectsProfileIdOverFourKiBBeforeCallingJni() {
        String oversizedProfileId = "x".repeat(4 * 1024 + 1);

        assertThrows(ResourceLimitException.class,
                () -> ProfileReadiness.evaluate(oversizedProfileId, List.of()));
    }

    @Test
    void readinessRejectsAggregateEvidenceBeforeReadingLaterRecords() {
        String largeRequirementId = "x".repeat(8 * 1024 * 1024);
        List<ProfileEvidenceRecord> evidence = new java.util.AbstractList<>() {
            @Override
            public ProfileEvidenceRecord get(int index) {
                if (index == 2) {
                    throw new AssertionError("the facade must reject before reading this record");
                }
                return ProfileEvidenceRecord.passed(largeRequirementId);
            }

            @Override
            public int size() {
                return 3;
            }
        };

        assertThrows(ResourceLimitException.class,
                () -> ProfileReadiness.evaluate(BASELINE, evidence));
    }

    @Test
    void readinessAllowsEvidenceIdentifierAboveProfileIdLimitWithinAggregateLimit() {
        String requirementId = "x".repeat(8 * 1024);

        assertTrue(ProfileReadiness.evaluate(
                BASELINE, List.of(ProfileEvidenceRecord.passed(requirementId))).isPresent());
    }

    @Test
    void readinessAcceptsStandardUtf8UnknownRequirementAndKeepsItRedacted() {
        String requirementId = String.valueOf((char) 0) + "UNKNOWN-λ-🔐";
        ProfileReadiness readiness = ProfileReadiness.evaluate(
                BASELINE, List.of(ProfileEvidenceRecord.passed(requirementId))).orElseThrow();
        ProfileReadinessBlocker unknown = readiness.blockers().stream()
                .filter(item -> item.kind().equals("unknown_requirement_record"))
                .findFirst().orElseThrow();

        assertNull(unknown.requirementId());
        assertFalse(unknown.message().contains(requirementId));
    }
}
