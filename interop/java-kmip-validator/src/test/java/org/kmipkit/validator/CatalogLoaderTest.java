package org.kmipkit.validator;

import org.junit.jupiter.api.Test;

import java.nio.file.Path;
import java.util.HashSet;
import java.util.Set;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.junit.jupiter.api.Assertions.assertTrue;

class CatalogLoaderTest {
    private static final Path CATALOG = Path.of("../shared/catalog/kmipkit-1.0-test-catalog.json");

    @Test
    void preservesVersionedCatalogAndCompleteNormativeCaseData() throws Exception {
        var catalog = CatalogLoader.load(CATALOG);

        assertEquals(Set.of("2.0", "2.1"), new HashSet<>(catalog.targetVersions()));
        assertEquals(catalog.totalCases(), catalog.cases().size());
        assertTrue(catalog.cases().size() > 218);
        assertTrue(catalog.normativeTargets().size() > 2_000);
        assertEquals(catalog.cases().size(), catalog.cases().stream().map(c -> c.caseId()).distinct().count());
        assertTrue(catalog.cases().stream().allMatch(c -> !c.normativeRefs().isEmpty()));
        assertTrue(catalog.cases().stream().allMatch(c -> !c.execution().protocolVersions().isEmpty()));
        assertTrue(catalog.cases().stream().allMatch(c ->
                catalog.targetVersions().containsAll(c.execution().protocolVersions())));
        assertTrue(catalog.cases().stream().allMatch(c -> !c.execution().steps().isEmpty()));
        assertTrue(catalog.cases().stream().allMatch(c -> !c.execution().assertions().isEmpty()));
        assertTrue(catalog.cases().stream().allMatch(c ->
                !"not_applicable".equals(c.execution().status())
                        || !c.execution().notApplicableReason().isBlank()));
    }

    @Test
    void rejectsMalformedCatalogInsteadOfTreatingItAsAnEmptyCatalog() throws Exception {
        Path malformed = Path.of("build/malformed-catalog.json");
        java.nio.file.Files.createDirectories(malformed.getParent());
        java.nio.file.Files.writeString(malformed, "{\"target_versions\":[\"2.1\"],\"test_cases\":[]}");

        assertThrows(IllegalArgumentException.class, () -> CatalogLoader.load(malformed));
    }
}
