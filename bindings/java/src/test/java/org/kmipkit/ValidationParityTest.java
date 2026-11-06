package org.kmipkit;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

import org.junit.jupiter.api.Test;
import org.kmipkit.ttlv.TtlvStructure;
import org.kmipkit.validation.ValidationIssue;
import org.kmipkit.validation.ValidationReport;

final class ValidationParityTest {
    @Test
    void genericValidationDoesNotGuessAnOperationWithoutAnExplicitSchema() {
        try (TtlvStructure structure = TtlvStructure.create();
                ValidationReport report = ValidationReport.validateStructure(structure, null)) {
            assertTrue(report.isValid());
            assertEquals(0, report.issues().size());
        }
    }

    @Test
    void explicitCreateSchemaReportsOnlySafeStableMetadata() {
        try (TtlvStructure structure = TtlvStructure.create();
                ValidationReport report = ValidationReport.validateStructure(
                        structure, "RequestPayload/Create")) {
            assertFalse(report.isValid());
            assertEquals(2, report.issues().size());
            ValidationIssue issue = report.issues().get(0);
            assertEquals("KMIPKIT-0051-R019", issue.code());
            assertEquals("Request Payload.Object Type", issue.path());
            assertEquals("ERROR", issue.severity());
            assertEquals("KMIP v2.1", issue.sourceIdentifier());
            assertEquals("§6.1.8, Table 186", issue.sourceReference());
            assertFalse(issue.toString().contains("SECRET"));
        }
    }
}
