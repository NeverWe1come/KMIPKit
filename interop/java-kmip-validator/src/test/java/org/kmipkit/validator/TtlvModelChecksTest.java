package org.kmipkit.validator;

import org.junit.jupiter.api.Test;

import java.nio.file.Path;

import static org.junit.jupiter.api.Assertions.assertArrayEquals;
import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertTrue;

class TtlvModelChecksTest {
    @Test
    void preservesUnknownEnumerationValuesAndBitmaskBits() {
        assertEquals(0xf0000001L, TtlvModelChecks.constructAndReadEnumeration(0xf0000001L));
        assertEquals(0x80000001, TtlvModelChecks.constructAndReadIntegerMask(0x80000001));
        assertEquals(0xf0000001L, TtlvModelChecks.decodeEnumeration(0xf0000001L));
        assertEquals(0x80000001, TtlvModelChecks.decodeIntegerMask(0x80000001));
    }

    @Test
    void preservesGenericStructureItemOrder() {
        assertArrayEquals(new int[] {0x420001, 0x420002}, TtlvModelChecks.constructAndReadTagOrder());
    }

    @Test
    void executesEveryApplicableLocalNormativeCatalogCase() throws Exception {
        CatalogLoader.Catalog catalog = CatalogLoader.load(
                Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json"));
        for (CatalogLoader.CatalogCase testCase : catalog.cases()) {
            if (!"local_model".equals(testCase.execution().mode())
                    || !testCase.execution().protocolVersions().contains("2.1")) {
                continue;
            }
            TtlvModelChecks.ModelCheck result = TtlvModelChecks.verifyLocalCase(
                    testCase, catalog.normativeTargets());
            assertTrue(result.passed(), () -> testCase.caseId() + ": " + result.diagnostic());
            if ("enumeration_value".equals(testCase.requires().path("target_kind").asText())
                    && "assigned".equals(testCase.requires().path("allocation").asText())) {
                assertTrue(result.diagnostic().contains("assigned KMIP enumeration value"),
                        () -> testCase.caseId() + " did not exercise the named KMIP enumeration API");
            }
            if ("bitmask_value".equals(testCase.requires().path("target_kind").asText())
                    && "assigned".equals(testCase.requires().path("allocation").asText())) {
                assertTrue(result.diagnostic().contains("assigned KMIP bitmask flag"),
                        () -> testCase.caseId() + " did not exercise the KMIP bitmask API");
            }
            if (testCase.requires().has("exhaustive_bitmask_combinations")) {
                assertTrue(result.diagnostic().contains("all bitmask combinations"),
                        () -> testCase.caseId() + " did not exercise all bitmask combinations");
            }
        }
    }
}
