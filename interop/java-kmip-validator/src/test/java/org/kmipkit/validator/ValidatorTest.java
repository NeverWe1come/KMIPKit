package org.kmipkit.validator;

import org.kmipkit.validator.models.Models.CaseResult;
import org.kmipkit.validator.models.Models.CaseStatus;
import org.kmipkit.validator.models.Models.ServerCapabilities;
import org.kmipkit.validator.models.Models.TestCase;

import java.io.IOException;
import java.nio.file.Path;
import java.nio.file.Paths;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;

public class ValidatorTest {
    public static void main(String[] args) {
        System.out.println("Running Java Validator Tests...");
        try {
            testAttemptContextUniqueness();
            testCapabilityMatcher();
            testCosmianProfileParity();
            testEquivalenceVerifier();
            testRecipeRunnerGuaranteedTeardown();
            System.out.println("All 5 test suites passed successfully!");
        } catch (Throwable t) {
            System.err.println("Test failure: " + t.getMessage());
            t.printStackTrace();
            System.exit(1);
        }
    }

    private static void assertTrue(boolean condition, String message) {
        if (!condition) throw new AssertionError(message);
    }

    private static void assertEquals(Object expected, Object actual, String message) {
        if (!expected.equals(actual)) {
            throw new AssertionError(message + " (expected: " + expected + ", got: " + actual + ")");
        }
    }

    public static void testAttemptContextUniqueness() {
        AttemptContext ctx1 = new AttemptContext("run-1", "java", "ttlv", "l2");
        AttemptContext ctx2 = new AttemptContext("run-1", "java", "ttlv", "l2");

        String name1 = ctx1.generateUniqueName("key");
        String name2 = ctx2.generateUniqueName("key");

        assertTrue(!name1.equals(name2), "Generated names should be unique");
        assertTrue(name1.startsWith("key-java-ttlv-l2-"), "Prefix format should match");
        System.out.println("  [PASS] testAttemptContextUniqueness");
    }

    public static void testCapabilityMatcher() {
        ServerCapabilities caps = new ServerCapabilities(
                "test-server",
                Set.of("Create", "Get", "Destroy"),
                Set.of("SymmetricKey"),
                Set.of("CryptographicAlgorithm")
        );

        CapabilityMatcher matcher = new CapabilityMatcher(caps);

        TestCase supportedCase = new TestCase("TC-1", "core", "Create", "SymmetricKey", "");
        assertTrue(matcher.isSupported(supportedCase), "Supported case should match");

        TestCase unsupportedOp = new TestCase("TC-2", "core", "Sign", "SymmetricKey", "");
        assertTrue(!matcher.isSupported(unsupportedOp), "Unsupported operation should fail match");

        TestCase unsupportedObj = new TestCase("TC-3", "core", "Create", "Certificate", "");
        assertTrue(!matcher.isSupported(unsupportedObj), "Unsupported object should fail match");

        System.out.println("  [PASS] testCapabilityMatcher");
    }

    public static void testCosmianProfileParity() throws IOException {
        Path catalogPath = Paths.get("../shared/catalog/kmipkit-1.0-test-catalog.json");
        Path profilePath = Paths.get("../shared/profiles/cosmian-kms.json");

        ServerCapabilities caps = JsonLoader.loadCapabilities(profilePath);
        List<TestCase> catalog = JsonLoader.loadCatalog(catalogPath);
        CapabilityMatcher matcher = new CapabilityMatcher(caps);

        int supported = 0;
        int unsupported = 0;

        for (TestCase tc : catalog) {
            if (matcher.isSupported(tc)) {
                supported++;
            } else {
                unsupported++;
            }
        }

        assertEquals(200, catalog.size(), "Catalog should contain 200 cases");
        assertEquals(117, supported, "Cosmian profile should support 117 cases");
        assertEquals(83, unsupported, "Cosmian profile should leave 83 unsupported");
        System.out.println("  [PASS] testCosmianProfileParity (117 supported, 83 unsupported)");
    }

    public static void testEquivalenceVerifier() {
        var r1 = new EquivalenceVerifier.NormalizedResponse("Success", "", "SymmetricKey", "deadbeef");
        var r2 = new EquivalenceVerifier.NormalizedResponse("Success", "", "SymmetricKey", "deadbeef");
        var rMismatch = new EquivalenceVerifier.NormalizedResponse("OperationFailed", "GeneralError", "SymmetricKey", "deadbeef");

        Map<String, EquivalenceVerifier.NormalizedResponse> mapOk = new LinkedHashMap<>();
        mapOk.put("HighLevel", r1);
        mapOk.put("TypedProtocol", r2);

        var resOk = EquivalenceVerifier.verifyEquivalence(mapOk);
        assertTrue(resOk.isEquivalent(), "Should be equivalent");
        assertTrue(resOk.discrepancies().isEmpty(), "No discrepancies");

        Map<String, EquivalenceVerifier.NormalizedResponse> mapBad = new LinkedHashMap<>();
        mapBad.put("HighLevel", r1);
        mapBad.put("LowLevel", rMismatch);

        var resBad = EquivalenceVerifier.verifyEquivalence(mapBad);
        assertTrue(!resBad.isEquivalent(), "Should not be equivalent");
        assertTrue(resBad.discrepancies().size() == 2, "2 discrepancies expected");
        System.out.println("  [PASS] testEquivalenceVerifier");
    }

    public static void testRecipeRunnerGuaranteedTeardown() {
        AtomicBoolean teardownRun = new AtomicBoolean(false);
        boolean caught = false;
        try {
            RecipeRunner.executeWithGuaranteedTeardown(
                    () -> true,
                    () -> { throw new RuntimeException("Target simulation failure"); },
                    () -> {
                        teardownRun.set(true);
                        return true;
                    }
            );
        } catch (RuntimeException e) {
            caught = true;
        }

        assertTrue(caught, "Target exception must propagate");
        assertTrue(teardownRun.get(), "Teardown must execute even when target fails");
        System.out.println("  [PASS] testRecipeRunnerGuaranteedTeardown");
    }
}
