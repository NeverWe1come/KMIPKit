# Traceability Matrix: KMIPKIT-0073

| Requirement ID | Design Element | Verification |
| --- | --- | --- |
| KMIPKIT-0073-FR-001 | `AwsKmsProvider` struct | `tests/aws_integration_tests.rs` |
| KMIPKIT-0073-FR-002 | Native KMS methods | `tests/aws_integration_tests.rs` |
| KMIPKIT-0073-FR-003 | Secrets Manager auto-envelope | `tests/aws_integration_tests.rs` |
| KMIPKIT-0073-FR-004 | Endpoint override in `AwsConfig` | `src/config.rs` & `tests/aws_integration_tests.rs` |
| KMIPKIT-0073-FR-005 | `.github/workflows/proxy-aws.yml` | GitHub Actions CI |
