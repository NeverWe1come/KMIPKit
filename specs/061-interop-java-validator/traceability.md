# Traceability Matrix: Standalone Java KMIP 2.1 Validator

| Requirement | Description | Implementation Artifact | Verification |
|-------------|-------------|-------------------------|--------------|
| REQ-VAL-JAVA-001 | Standalone Java Gradle/javac project isolated from root | `interop/java-kmip-validator/build.gradle.kts` | Standalone compile |
| REQ-VAL-JAVA-002 | JSON loader for capabilities and catalog | `org/kmipkit/validator/JsonLoader.java` | `ValidatorTest.testCosmianProfileParity` |
| REQ-VAL-JAVA-003 | Capability matcher with operation, object, attribute filtering | `org/kmipkit/validator/CapabilityMatcher.java` | `ValidatorTest.testCapabilityMatcher` |
| REQ-VAL-JAVA-004 | Attempt context with unique naming for isolation | `org/kmipkit/validator/AttemptContext.java` | `ValidatorTest.testAttemptContextUniqueness` |
| REQ-VAL-JAVA-005 | ANSI, HTML, Markdown, JSON matrix reporting | `org/kmipkit/validator/MatrixReporter.java` | File generation verification |
| REQ-VAL-JAVA-006 | CLI interface with `plan` and `run` commands | `org/kmipkit/validator/Main.java` | CLI integration test |
| REQ-VAL-JAVA-007 | Lightweight GitHub Actions workflow | `.github/workflows/interop-java.yml` | Action syntax & path filter |
