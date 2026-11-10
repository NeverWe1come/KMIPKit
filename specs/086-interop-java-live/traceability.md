# KMIPKIT-0086 Traceability

Each case-level conformance check retains the OASIS document and section
references in `interop/shared/catalog/kmipkit-1.0-test-catalog.json`.

| Requirement | Implementation | Verification |
|---|---|---|
| FR-001 | `Main.java`, `ConnectionConfig.java`, `CatalogLoader.java` | `MainContractTest.offlinePlanPreflightAndRunClassifyLocalCasesWithoutServerClaims`; `CatalogLoaderTest` |
| FR-002 | `tools/install_artifact.py`, `ArtifactProvenance.java` | `ArtifactProvenanceTest.verifiesPinnedCommitAndJarDigest`; `ArtifactProvenanceTest.executesPublicApiFromTheInstalledJar` |
| FR-003 | `CatalogLoader.java`, `JavaRouteInventory.java`, `Main.java`, `JavaKmipDriver.java` | `CatalogLoaderTest.preservesVersionedCatalogAndCompleteNormativeCaseData`; `JavaRouteInventoryTest.classifiesEveryCatalogOperationAndExplainsPendingRoutes`; `MainContractTest.planDoesNotCountObjectLifecycleCasesWithoutCompleteRecipes`; every applicable case is represented in the `MainContractTest` report |
| FR-004 | `JavaKmipDriver.java`, `TypedRequestFactory.java` | Controlled mTLS cases cover typed Discover Versions, Query, and Log requests and responses; live Log remains pending because the shared catalog does not yet require mutation opt-in; `Get Constraints`, `Get Attribute List`, and other incomplete recipes remain pending |
| FR-005 | `TtlvModelChecks.java` | `TtlvModelChecksTest.preservesUnknownEnumerationValuesAndBitmaskBits`; `TtlvModelChecksTest.preservesGenericStructureItemOrder` |
| FR-006 | `ConnectionConfig.java`, `Main.java`, `JavaKmipDriver.java` | `ConnectionConfigTest.rejectsUnconfirmedEnvironmentAndMalformedFingerprint`; `ConnectionConfigTest.mutationsRequireConfigurationAndCommandLineOptIn` |
| FR-007 | `RunReport.java`, `MatrixReporter.java` | `ReportWriterTest.emitsTheSharedSchemaAndDoesNotAcceptRemotePayloadText`; workflow validates JSON against `validation-results.schema.json` |
| FR-008 | `.github/workflows/interop-java.yml`, `ControlledTlsPeerTest.java` | Workflow installs the exact pinned artifact and exercises mTLS with local controlled Discover Versions, Query, and Log peers. |
