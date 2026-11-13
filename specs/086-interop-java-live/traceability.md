# KMIPKIT-0086 Traceability

Each case-level conformance check retains the OASIS document and section
references in `interop/shared/catalog/kmipkit-1.0-test-catalog.json`.

| Requirement | Implementation | Verification |
|---|---|---|
| FR-001 | `Main.java`, `ConnectionConfig.java`, `CatalogLoader.java` | `MainContractTest.offlinePlanPreflightAndRunClassifyLocalCasesWithoutServerClaims`; `CatalogLoaderTest` |
| FR-002 | `tools/install_artifact.py`, `library-pin.json`, `ArtifactProvenance.java` | `ArtifactProvenanceTest.libraryPinMatchesTheExactSharedReleaseArtifactCommit`; `ArtifactProvenanceTest.verifiesPinnedCommitAndJarDigest`; `ArtifactProvenanceTest.executesPublicApiFromTheInstalledJar` |
| FR-003 | `CatalogLoader.java`, `JavaRouteInventory.java`, `Main.java`, `JavaKmipDriver.java` | `CatalogLoaderTest.preservesVersionedCatalogAndCompleteNormativeCaseData`; `JavaRouteInventoryTest.classifiesEveryCatalogOperationAndExplainsPendingRoutes`; `MainContractTest.unsupportedNormativeTtlvTargetIsReportedAsCoveragePendingNotLibraryFailure`; `MainContractTest.planDoesNotCountCasesWithoutCompleteRecipesOrMutationSafetyAsExecutable`; every applicable case is represented in the report |
| FR-004 | `JavaKmipDriver.java`, `TypedRequestFactory.java` | Controlled mTLS cases cover typed Discover Versions, Query, Log, guarded Create, and KMIP 2.0 Query requests and responses; Create verifies a unique-named AES-256 Symmetric Key and cleans it up with Destroy in a second exchange; KMIP 2.0 Discover Versions, live Log, `Get Constraints`, `Get Attribute List`, and other incomplete recipes remain pending |
| FR-005 | `TtlvModelChecks.java` | `TtlvModelChecksTest.preservesUnknownEnumerationValuesAndBitmaskBits`; `TtlvModelChecksTest.preservesGenericStructureItemOrder` |
| FR-006 | `ConnectionConfig.java`, `Main.java`, `JavaKmipDriver.java` | `ConnectionConfigTest.rejectsUnconfirmedEnvironmentAndMalformedFingerprint`; `ConnectionConfigTest.mutationsRequireConfigurationAndCommandLineOptIn` |
| FR-007 | `RunReport.java`, `MatrixReporter.java` | `ReportWriterTest.emitsTheSharedSchemaAndDoesNotAcceptRemotePayloadText`; workflow validates JSON against `validation-results.schema.json` |
| FR-008 | `.github/workflows/interop-java.yml`, `ControlledTlsPeerTest.java` | Workflow installs the exact shared release-pinned artifact and exercises mTLS with local controlled KMIP 2.0/2.1 Query and Discover Versions, Log, and Create/Destroy exchanges. |
