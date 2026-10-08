package org.kmipkit.extensions;

import org.kmipkit.ResourceLimitException;

/** Immutable per-registry bounds shared by the extension adapters. */
public final class ExtensionRegistryLimits {
    private static final long[] HARD_MAXIMUMS = {
            1_024, 100_000, 4_096, 4_096, 16_777_216, 4_096,
            16_777_216, 4_096, 100_000, 200_000, 4_194_304, 64
    };
    private static final long[] DEFAULTS = {
            256, 16_384, 256, 4_096, 1_048_576, 4_096,
            1_048_576, 256, 16_384, 200_000, 1_048_576, 64
    };

    private final long[] values;

    private ExtensionRegistryLimits(long[] values) {
        this.values = values.clone();
    }

    public static ExtensionRegistryLimits defaults() {
        return new ExtensionRegistryLimits(DEFAULTS);
    }

    public static ExtensionRegistryLimits withValues(long maxDefinitions, long maxSchemaNodes,
            long maxChildRulesPerStructure, long maxTextBytesPerField, long maxRegistryTextBytes,
            long maxDiscriminatorScalarBytes, long maxTotalDiscriminatorScalarBytes,
            long maxConstraintMembersPerRule, long maxTotalConstraintMembers,
            long maxPayloadIndexRecords, long maxLookupComparisons, long maxDepth) {
        long[] requested = {
                maxDefinitions, maxSchemaNodes, maxChildRulesPerStructure, maxTextBytesPerField,
                maxRegistryTextBytes, maxDiscriminatorScalarBytes, maxTotalDiscriminatorScalarBytes,
                maxConstraintMembersPerRule, maxTotalConstraintMembers, maxPayloadIndexRecords,
                maxLookupComparisons, maxDepth
        };
        for (int index = 0; index < requested.length; index++) {
            if (requested[index] > HARD_MAXIMUMS[index]) {
                throw new ResourceLimitException("extension registry limit exceeds its hard maximum");
            }
        }
        return new ExtensionRegistryLimits(requested);
    }

    public long maxDefinitions() { return values[0]; }
    public long maxSchemaNodes() { return values[1]; }
    public long maxChildRulesPerStructure() { return values[2]; }
    public long maxTextBytesPerField() { return values[3]; }
    public long maxRegistryTextBytes() { return values[4]; }
    public long maxDiscriminatorScalarBytes() { return values[5]; }
    public long maxTotalDiscriminatorScalarBytes() { return values[6]; }
    public long maxConstraintMembersPerRule() { return values[7]; }
    public long maxTotalConstraintMembers() { return values[8]; }
    public long maxPayloadIndexRecords() { return values[9]; }
    public long maxLookupComparisons() { return values[10]; }
    public long maxDepth() { return values[11]; }

    long[] nativeValues() {
        return values.clone();
    }
}
