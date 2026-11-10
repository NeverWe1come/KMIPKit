# KMIPKIT-0086 Traceability

Each case-level conformance check retains the OASIS document and section
references in `interop/shared/catalog/kmipkit-1.0-test-catalog.json`.

| Requirement | Implementation | Verification |
|---|---|---|
| FR-001 | `Main.java`, `ConnectionConfig.java`, `Preflight.java` | `JavaContractTest.planAndPreflightAreOffline`; `ConnectionConfigTest.rejectsInvalidContracts` |
| FR-002 | `tools/install_artifact.py`, `ArtifactProvenance.java` | `ArtifactProvenanceTest.rejectsCommitAndDigestMismatch`; installed-consumer CI |
| FR-003 | `CatalogLoader.java`, `JavaKmipDriver.java`, `Models.java` | `CatalogCoverageTest.everyApplicableCaseHasAnOutcomeOrReason` |
| FR-004 | `JavaKmipDriver.java`, `CaseRecipes.java` | `ControlledTlsPeerTest.typedOperationsUseTheInstalledPackage` |
| FR-005 | `TtlvModelChecks.java`, `EquivalenceVerifier.java` | `TtlvModelTest.preservesRawValuesAndOrder`; `EquivalenceVerifierTest.normalizesEquivalentResponses` |
| FR-006 | `ConnectionConfig.java`, `JavaKmipDriver.java` | `SafetyGuardTest.requiresDedicatedEnvironmentAndMutationOptIn`; `ControlledTlsPeerTest.doesNotLeakCredentials` |
| FR-007 | `MatrixReporter.java`, `Models.java` | `ReportSchemaTest.jsonMatchesSharedSchemaAndIsSanitized` |
| FR-008 | `.github/workflows/interop-java.yml`, `tests/controlled_kms_test.py` | Workflow builds the installed artifact and runs the TLS peer fixture against the external Java consumer. |
