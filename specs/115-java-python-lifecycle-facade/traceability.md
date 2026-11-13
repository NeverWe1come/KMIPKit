# KMIPKIT-0115 Traceability

| Requirement | Implementation | Verification | Normative / project source |
|---|---|---|---|
| KMIPKIT-0115-FR-001 | Java `FluentRequest`/builders; Python `Request`/builders; `fluent_java_python_lifecycle_facade` declarations in `specification/api/public-api.json` | Java/Python shape tests build five matching typed batch items | KMIP 2.1 §§6.1.1, 6.1.4, 6.1.28, 6.1.42, 6.1.44; KMIPKIT-0093-FR-004 |
| KMIPKIT-0115-FR-002 | Existing client execution path and typed batch response | Controlled mTLS tests decode requests and return typed Success/Pending/Failure results; one exchange asserted | KMIPKIT-0093-FR-004, FR-007, FR-011 |
| KMIPKIT-0115-FR-003 | Java/Python `LocateBuilder` | Tests assert required Attributes and exact optional filter presence and values | KMIP 2.1 §6.1.28; KMIPKIT-0101-FR-003 |
| KMIPKIT-0115-FR-004 | Java/Python `RevokeBuilder` | Tests assert reason, optional message/date, absent message redaction, and no-send invalid values | KMIP 2.1 §6.1.44; KMIPKIT-0101-FR-002 |
| KMIPKIT-0115-FR-005 | `KmipClient.{activateById,archiveById,locate,recoverById,revokeById}` and Python `Client.{activate_by_id,archive_by_id,locate,recover_by_id,revoke_by_id}` | Compiled Java and Python tests invoke every direct method and options path | KMIPKIT-0093-FR-004, FR-007 |
| KMIPKIT-0115-FR-006 | Builder options and asynchronous indicator fields | Tests assert options/header separation and no automatic follow-up | KMIPKIT-0093-FR-005, FR-007, FR-011 |
| KMIPKIT-0115-FR-007 | Existing native-handle wrappers and documented language limitations | Repeated build/close tests, failure cleanup checks, and bilingual ownership notes | KMIPKIT-0007 security invariants; KMIPKIT-0093-FR-011 |
| KMIPKIT-0115-FR-008 | Capability `fluent_java_python_lifecycle_facade`, generated Java/Python export tables, and cross-language declaration mappings | `tools/api_manifest/generate.py --check` and `audit_binding_exports.py --check` both pass with no missing, stale, or mismatched exports | KMIPKIT-0093-FR-013 |
| KMIPKIT-0115-FR-009 | Java/Python controlled mutual-TLS integration tests | Fixture decodes each operation; invalid local inputs produce zero accepted connections | KMIP 2.1 §§6.1.1, 6.1.4, 6.1.28, 6.1.42, 6.1.44 |
