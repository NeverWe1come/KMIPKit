# KMIPKIT-0086 Traceability

Each case-level conformance check retains the OASIS document and section
references in `interop/shared/catalog/kmipkit-1.0-test-catalog.json`.

| Requirement | Implementation | Verification |
|---|---|---|
| FR-001 | `Main.java`, `ConnectionConfig.java`, `CatalogLoader.java` | `MainContractTest.offlinePlanPreflightAndRunClassifyLocalCasesWithoutServerClaims`; `CatalogLoaderTest` |
| FR-002 | `tools/install_artifact.py`, `ArtifactProvenance.java` | `ArtifactProvenanceTest.verifiesPinnedCommitAndJarDigest`; `ArtifactProvenanceTest.executesPublicApiFromTheInstalledJar` |
| FR-003 | `CatalogLoader.java`, `JavaRouteInventory.java`, `Main.java`, `JavaKmipDriver.java` | `CatalogLoaderTest.preservesVersionedCatalogAndCompleteNormativeCaseData`; `JavaRouteInventoryTest.classifiesEveryCatalogOperationAndExplainsPendingRoutes`; every applicable case is represented in the `MainContractTest` report |
| FR-004 | `JavaKmipDriver.java` | `ControlledTlsPeerTest.typedDiscoverVersionsUsesInstalledPackageAndVerifiedMutualTls`; additional catalog operations remain explicit coverage gaps |
| FR-005 | `TtlvModelChecks.java` | `TtlvModelChecksTest.preservesUnknownEnumerationValuesAndBitmaskBits`; `TtlvModelChecksTest.preservesGenericStructureItemOrder` |
| FR-006 | `ConnectionConfig.java`, `Main.java`, `JavaKmipDriver.java` | `ConnectionConfigTest.rejectsUnconfirmedEnvironmentAndMalformedFingerprint`; `ConnectionConfigTest.mutationsRequireConfigurationAndCommandLineOptIn` |
| FR-007 | `RunReport.java`, `MatrixReporter.java` | `ReportWriterTest.emitsTheSharedSchemaAndDoesNotAcceptRemotePayloadText`; workflow validates JSON against `validation-results.schema.json` |
| FR-008 | `.github/workflows/interop-java.yml`, `ControlledTlsPeerTest.java` | Workflow installs the exact pinned artifact and exercises mTLS with a local controlled peer. |
