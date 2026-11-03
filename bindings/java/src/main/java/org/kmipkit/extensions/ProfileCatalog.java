package org.kmipkit.extensions;

import java.util.ArrayList;
import java.util.List;
import java.util.Optional;
import org.kmipkit.InvalidInputException;
import org.kmipkit.NativeExtensionRegistry;

/** Owned immutable snapshots of generated profile metadata. */
public final class ProfileCatalog {
    private static final int PROFILES = NativeExtensionRegistry.PROFILE_CATALOG_CLIENT_PROFILES;
    private static final int REQUIREMENTS = NativeExtensionRegistry.PROFILE_CATALOG_CLIENT_REQUIREMENTS;
    private static final int TEST_CASES = NativeExtensionRegistry.PROFILE_CATALOG_PROFILE_TEST_CASES;
    private static final int DIRECT_CLAUSES = 1;
    private static final int SOURCE_CLAUSES = 2;
    private static final int REQUIREMENT_IDS = 3;
    private static final int SCOPE_BLOCKERS = 4;
    private static final int DEPENDENCIES = 5;
    private static final int TRANSPORTS = 6;
    private static final int ENCODINGS = 7;
    private static final int PROFILE_TESTS = 8;
    private static final int PROFILE_IDS = 9;
    private static final int OFFICIAL_TESTS = 10;

    private ProfileCatalog() {}

    public static List<ClientProfile> clientProfiles() {
        List<ClientProfile> rows = new ArrayList<>();
        long count = NativeExtensionRegistry.profileCatalogTableCount(PROFILES);
        for (long index = 0; index < count; index++) {
            String[] row = NativeExtensionRegistry.profileCatalogRow(PROFILES, index);
            List<ProfileScopeBlocker> blockers = new ArrayList<>();
            for (long nested = 0; nested < relationCount(PROFILES, index, SCOPE_BLOCKERS); nested++) {
                String[] item = relationRow(PROFILES, index, SCOPE_BLOCKERS, nested);
                blockers.add(new ProfileScopeBlocker(item[0], item[1], item[2]));
            }
            List<ProfileTestCase> testCases = new ArrayList<>();
            for (long nested = 0; nested < relationCount(PROFILES, index, PROFILE_TESTS); nested++) {
                testCases.add(testCase(relationRow(PROFILES, index, PROFILE_TESTS, nested)));
            }
            rows.add(new ClientProfile(row[0], row[1], row[2], row[3], Boolean.parseBoolean(row[4]),
                    strings(PROFILES, index, DIRECT_CLAUSES), strings(PROFILES, index, SOURCE_CLAUSES),
                    strings(PROFILES, index, REQUIREMENT_IDS), blockers,
                    strings(PROFILES, index, DEPENDENCIES), strings(PROFILES, index, TRANSPORTS),
                    strings(PROFILES, index, ENCODINGS), testCases));
        }
        return List.copyOf(rows);
    }

    public static List<ClientRequirement> clientRequirements() {
        List<ClientRequirement> rows = new ArrayList<>();
        long count = NativeExtensionRegistry.profileCatalogTableCount(REQUIREMENTS);
        for (long index = 0; index < count; index++) {
            String[] row = NativeExtensionRegistry.profileCatalogRow(REQUIREMENTS, index);
            rows.add(new ClientRequirement(row[0], optional(row[1]), optional(row[2]), optional(row[3]),
                    optional(row[4]), VerificationCategory.fromValue(row[5]), row[6],
                    strings(REQUIREMENTS, index, SOURCE_CLAUSES),
                    strings(REQUIREMENTS, index, PROFILE_IDS),
                    strings(REQUIREMENTS, index, OFFICIAL_TESTS), Boolean.parseBoolean(row[7])));
        }
        return List.copyOf(rows);
    }

    public static List<ProfileTestCase> profileTestCases() {
        List<ProfileTestCase> rows = new ArrayList<>();
        long count = NativeExtensionRegistry.profileCatalogTableCount(TEST_CASES);
        for (long index = 0; index < count; index++) {
            rows.add(testCase(NativeExtensionRegistry.profileCatalogRow(TEST_CASES, index)));
        }
        return List.copyOf(rows);
    }

    private static long relationCount(int table, long row, int relation) {
        return NativeExtensionRegistry.profileCatalogRelationCount(table, row, relation);
    }

    private static String[] relationRow(int table, long row, int relation, long index) {
        return NativeExtensionRegistry.profileCatalogRelationRow(table, row, relation, index);
    }

    private static List<String> strings(int table, long row, int relation) {
        List<String> values = new ArrayList<>();
        for (long index = 0; index < relationCount(table, row, relation); index++) {
            String[] item = relationRow(table, row, relation, index);
            values.add(item[0]);
        }
        return List.copyOf(values);
    }

    private static Optional<String> optional(String value) {
        return Optional.ofNullable(value);
    }

    private static ProfileTestCase testCase(String[] row) {
        return new ProfileTestCase(row[0], row[1], row[2], row[3]);
    }

    private static String required(String value) {
        if (value == null) throw new InvalidInputException("profile catalog row is invalid");
        return value;
    }

    public enum VerificationCategory {
        REQUIRED_BEHAVIOR("required_behavior"),
        PROHIBITED_BEHAVIOR("prohibited_behavior"),
        RECOMMENDED_BEHAVIOR_OR_DEVIATION("recommended_behavior_or_deviation"),
        DISCOURAGED_ABSENCE_OR_DEVIATION("discouraged_absence_or_deviation"),
        OPTIONAL_CAPABILITY("optional_capability");

        private final String value;
        VerificationCategory(String value) { this.value = value; }
        public String value() { return value; }
        static VerificationCategory fromValue(String value) {
            for (VerificationCategory category : values()) if (category.value.equals(value)) return category;
            throw new InvalidInputException("profile catalog category is invalid");
        }
    }

    public record ProfileScopeBlocker(String requirementId, String sourceClauseId, String scopeState) {}
    public record ProfileTestCase(String testId, String officialCaseId, String mandatoryStatus,
            String fixtureAvailability) {}
    public record ClientProfile(String profileId, String name, String applicability, String claimState,
            boolean selectedTarget, List<String> directSourceClauseIds, List<String> sourceClauseIds,
            List<String> requirementIds, List<ProfileScopeBlocker> unresolvedScopeObligations,
            List<String> dependencyProfileIds, List<String> transportRequirements,
            List<String> encodingRequirements, List<ProfileTestCase> testCases) {
        public ClientProfile {
            required(profileId); required(name); required(applicability); required(claimState);
            directSourceClauseIds = List.copyOf(directSourceClauseIds);
            sourceClauseIds = List.copyOf(sourceClauseIds);
            requirementIds = List.copyOf(requirementIds);
            unresolvedScopeObligations = List.copyOf(unresolvedScopeObligations);
            dependencyProfileIds = List.copyOf(dependencyProfileIds);
            transportRequirements = List.copyOf(transportRequirements);
            encodingRequirements = List.copyOf(encodingRequirements);
            testCases = List.copyOf(testCases);
        }
    }
    public record ClientRequirement(String requirementId, Optional<String> sourceKeyword,
            Optional<String> sourceForce, Optional<String> sourceGapId, Optional<String> sourceLocator,
            VerificationCategory verificationCategory, String scopeState, List<String> sourceClauseIds,
            List<String> profileIds, List<String> officialTestCaseIds, boolean hasAcceptedDeviation) {
        public ClientRequirement {
            required(requirementId); required(scopeState);
            sourceClauseIds = List.copyOf(sourceClauseIds); profileIds = List.copyOf(profileIds);
            officialTestCaseIds = List.copyOf(officialTestCaseIds);
        }
    }
}
