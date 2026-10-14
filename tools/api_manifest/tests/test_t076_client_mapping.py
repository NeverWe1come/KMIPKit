"""Regressions for the reviewed T076 shared-client public API mapping."""

from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

from tools.api_manifest import audit_binding_exports, generate


ROOT = Path(__file__).resolve().parents[3]
MANIFEST = ROOT / "specification" / "api" / "public-api.json"
T076_SYMBOLS = {
    "kmipkit_client_batch_add_error_continuation_value",
    "kmipkit_client_batch_add_item",
    "kmipkit_client_batch_create",
    "kmipkit_client_batch_item_with_unique_batch_item_id",
    "kmipkit_client_batch_item_response_metadata_v1",
    "kmipkit_client_batch_item_response_pending_correlation_value",
    "kmipkit_client_batch_item_response_release",
    "kmipkit_client_batch_item_response_t",
    "kmipkit_client_batch_item_response_unique_batch_item_id",
    "kmipkit_client_batch_release",
    "kmipkit_client_batch_response_item_at",
    "kmipkit_client_batch_response_item_count",
    "kmipkit_client_batch_response_release",
    "kmipkit_client_batch_response_t",
    "kmipkit_client_batch_set_asynchronous_indicator",
    "kmipkit_client_batch_set_client_correlation_value",
    "kmipkit_client_batch_set_order_option",
    "kmipkit_client_batch_set_request_time_stamp",
    "kmipkit_client_batch_t",
    "kmipkit_client_create",
    "kmipkit_client_execute",
    "kmipkit_client_release",
    "kmipkit_client_t",
    "kmipkit_last_error_get",
    "kmipkit_transport_config_create_v1",
    "kmipkit_transport_config_release",
    "kmipkit_transport_config_t",
    "kmipkit_client_execute_with_error_ttlv_v1",
}
T076_BLOCKED = {
    "kmipkit_client_execute_with_error_ttlv_v1",
}
T076_EXCLUDED = {
    "kmipkit_client_batch_item_response_metadata_v1",
    "kmipkit_client_batch_item_response_release",
    "kmipkit_transport_config_create_v1",
    "kmipkit_transport_config_release",
}
T076_READY = T076_SYMBOLS - T076_BLOCKED - T076_EXCLUDED

C_SEMANTIC_FIELDS = {
    "kmipkit_client_batch_item_metadata_v1_t.has_result_reason",
    "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
    "kmipkit_client_batch_item_metadata_v1_t.result_reason",
    "kmipkit_client_batch_item_metadata_v1_t.result_status",
    "kmipkit_timeout_limit_v1_t.kind",
    "kmipkit_timeout_limit_v1_t.seconds",
    "kmipkit_timeout_limit_v1_t.nanoseconds",
    "kmipkit_transport_config_v1_t.endpoint_kind",
    "kmipkit_transport_config_v1_t.trust_kind",
    "kmipkit_transport_config_v1_t.endpoint",
    "kmipkit_transport_config_v1_t.endpoint_port",
    "kmipkit_transport_config_v1_t.client_certificates",
    "kmipkit_transport_config_v1_t.client_private_key",
    "kmipkit_transport_config_v1_t.trust_certificates",
    "kmipkit_transport_config_v1_t.revocation_lists",
    "kmipkit_transport_config_v1_t.tls_server_name",
    "kmipkit_transport_config_v1_t.target_uri",
    "kmipkit_transport_config_v1_t.connect_timeout",
    "kmipkit_transport_config_v1_t.write_timeout",
    "kmipkit_transport_config_v1_t.read_timeout",
    "kmipkit_transport_config_v1_t.total_timeout",
    "kmipkit_transport_config_v1_t.max_request_bytes",
    "kmipkit_request_options_v1_t.connect_timeout",
    "kmipkit_request_options_v1_t.write_timeout",
    "kmipkit_request_options_v1_t.read_timeout",
    "kmipkit_request_options_v1_t.total_timeout",
}
C_REPRESENTATION_FIELD_EXCLUSIONS = {
    "kmipkit_transport_config_v1_t.client_certificate_count",
    "kmipkit_transport_config_v1_t.flags",
    "kmipkit_transport_config_v1_t.revocation_list_count",
    "kmipkit_transport_config_v1_t.trust_certificate_count",
    "kmipkit_transport_input_v1_t.data",
    "kmipkit_transport_input_v1_t.encoding",
    "kmipkit_transport_input_v1_t.source_kind",
}
C_ABI_FIELD_EXCLUSIONS = {
    "kmipkit_byte_span_v1_t.data",
    "kmipkit_byte_span_v1_t.length",
    "kmipkit_client_batch_item_metadata_v1_t.struct_size",
    "kmipkit_client_batch_item_metadata_v1_t.abi_version",
    "kmipkit_timeout_limit_v1_t.reserved",
    "kmipkit_timeout_limit_v1_t.reserved2",
    "kmipkit_transport_config_v1_t.struct_size",
    "kmipkit_transport_config_v1_t.abi_version",
    "kmipkit_transport_config_v1_t.reserved",
    "kmipkit_transport_config_v1_t.reserved2",
    "kmipkit_request_options_v1_t.struct_size",
    "kmipkit_request_options_v1_t.abi_version",
}
C_LAYOUT_TYPES = {
    "kmipkit_byte_span_v1_t",
    "kmipkit_client_batch_item_metadata_v1_t",
    "kmipkit_timeout_limit_v1_t",
    "kmipkit_transport_config_v1_t",
    "kmipkit_transport_input_v1_t",
}
C_FIELD_TARGETS = {
    "kmipkit_client_batch_item_metadata_v1_t.has_result_reason": (
        ("rust-kmipkit-protocol", "kmipkit_protocol::KmipOperationResult::reason"),
        "org.kmipkit.extensions.ClientBatchItemResult.resultReason",
        "kmipkit.client.BatchItemResult.result_reason",
    ),
    "kmipkit_client_batch_item_metadata_v1_t.outcome_kind": (
        ("rust-kmipkit-client", "kmipkit_client::ClientBatchItemResponse::outcome"),
        "org.kmipkit.extensions.ClientBatchItemResult.outcomeKind",
        "kmipkit.client.BatchItemResult.outcome",
    ),
    "kmipkit_client_batch_item_metadata_v1_t.result_reason": (
        ("rust-kmipkit-protocol", "kmipkit_protocol::KmipOperationResult::reason"),
        "org.kmipkit.extensions.ClientBatchItemResult.resultReason",
        "kmipkit.client.BatchItemResult.result_reason",
    ),
    "kmipkit_client_batch_item_metadata_v1_t.result_status": (
        ("rust-kmipkit-protocol", "kmipkit_protocol::KmipOperationResult::status"),
        "org.kmipkit.extensions.ClientBatchItemResult.resultStatus",
        "kmipkit.client.BatchItemResult.result_status",
    ),
    "kmipkit_timeout_limit_v1_t.kind": (
        ("rust-kmipkit-transport", "kmipkit_transport::TimeoutPolicy"),
        "org.kmipkit.extensions.TransportConfiguration.Timeout.kind",
        "kmipkit.client.TimeoutLimit.kind",
    ),
    "kmipkit_timeout_limit_v1_t.nanoseconds": (
        ("rust-kmipkit-transport", "kmipkit_transport::TimeoutLimit::Bounded"),
        "org.kmipkit.extensions.TransportConfiguration.Timeout.nanoseconds",
        "kmipkit.client.TimeoutLimit.nanoseconds",
    ),
    "kmipkit_timeout_limit_v1_t.seconds": (
        ("rust-kmipkit-transport", "kmipkit_transport::TimeoutLimit::Bounded"),
        "org.kmipkit.extensions.TransportConfiguration.Timeout.seconds",
        "kmipkit.client.TimeoutLimit.seconds",
    ),
    "kmipkit_transport_config_v1_t.client_certificates": (
        ("rust-kmipkit-transport", "kmipkit_transport::CertificateInput"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.TransportConfig.client_certificates",
    ),
    "kmipkit_transport_config_v1_t.client_private_key": (
        ("rust-kmipkit-transport", "kmipkit_transport::PrivateKeyInput"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.TransportConfig.client_private_key",
    ),
    "kmipkit_transport_config_v1_t.connect_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::timeouts"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.timeouts",
        "kmipkit.client.TransportConfig.connect_timeout",
    ),
    "kmipkit_transport_config_v1_t.endpoint": (
        ("rust-kmipkit-transport", "kmipkit_transport::Endpoint"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.Endpoint",
    ),
    "kmipkit_transport_config_v1_t.endpoint_kind": (
        ("rust-kmipkit-transport", "kmipkit_transport::Endpoint"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.Endpoint",
    ),
    "kmipkit_transport_config_v1_t.endpoint_port": (
        ("rust-kmipkit-transport", "kmipkit_transport::Endpoint::raw_tls"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.rawTls",
        "kmipkit.client.Endpoint.raw_tls",
    ),
    "kmipkit_transport_config_v1_t.max_request_bytes": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::max_request_bytes"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.maxRequestBytes",
        "kmipkit.client.TransportConfig.max_request_bytes",
    ),
    "kmipkit_request_options_v1_t.connect_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::RequestOptions::with_connect"),
        "org.kmipkit.extensions.RequestOptions.connectTimeout",
        "kmipkit.client.RequestOptions.connect_timeout",
    ),
    "kmipkit_request_options_v1_t.write_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::RequestOptions::with_write"),
        "org.kmipkit.extensions.RequestOptions.writeTimeout",
        "kmipkit.client.RequestOptions.write_timeout",
    ),
    "kmipkit_request_options_v1_t.read_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::RequestOptions::with_read"),
        "org.kmipkit.extensions.RequestOptions.readTimeout",
        "kmipkit.client.RequestOptions.read_timeout",
    ),
    "kmipkit_request_options_v1_t.total_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::RequestOptions::with_total"),
        "org.kmipkit.extensions.RequestOptions.totalTimeout",
        "kmipkit.client.RequestOptions.total_timeout",
    ),
    "kmipkit_transport_config_v1_t.read_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::timeouts"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.timeouts",
        "kmipkit.client.TransportConfig.read_timeout",
    ),
    "kmipkit_transport_config_v1_t.revocation_lists": (
        ("rust-kmipkit-transport", "kmipkit_transport::RevocationListInput"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.TransportConfig.revocation_lists",
    ),
    "kmipkit_transport_config_v1_t.target_uri": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::target_uri"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.targetUri",
        "kmipkit.client.Endpoint.target_uri",
    ),
    "kmipkit_transport_config_v1_t.tls_server_name": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::tls_server_name"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.tlsServerName",
        "kmipkit.client.Endpoint.tls_server_name",
    ),
    "kmipkit_transport_config_v1_t.total_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::timeouts"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.timeouts",
        "kmipkit.client.TransportConfig.total_timeout",
    ),
    "kmipkit_transport_config_v1_t.trust_certificates": (
        ("rust-kmipkit-transport", "kmipkit_transport::TrustSource::certificate_authorities"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.TrustSource.certificate_authorities",
    ),
    "kmipkit_transport_config_v1_t.trust_kind": (
        ("rust-kmipkit-transport", "kmipkit_transport::TrustSource"),
        "org.kmipkit.extensions.TransportConfiguration.Builder",
        "kmipkit.client.TrustSource.kind",
    ),
    "kmipkit_transport_config_v1_t.write_timeout": (
        ("rust-kmipkit-transport", "kmipkit_transport::TransportConfigBuilder::timeouts"),
        "org.kmipkit.extensions.TransportConfiguration.Builder.timeouts",
        "kmipkit.client.TransportConfig.write_timeout",
    ),
}


class T076ClientMappingTests(unittest.TestCase):
    def setUp(self) -> None:
        self.manifest = json.loads(MANIFEST.read_text(encoding="utf-8"))
        self.declarations = [
            row
            for row in self.manifest["publicDeclarations"]
            if row.get("declarationId", "").startswith("T076:")
        ]

    def test_all_28_exports_are_mapped_or_remain_exactly_blocked(self) -> None:
        c_rows = [
            row
            for row in self.declarations
            if row["packageId"] == "c-kmipkit" and row["exportName"] in T076_SYMBOLS
        ]
        exclusions = [
            row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit" and row["exportName"] in T076_SYMBOLS
        ]
        carrier = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["exportName"] == "kmipkit_client_batch_item_metadata_v1_t"
        )
        self.assertEqual(
            T076_SYMBOLS,
            {row["exportName"] for row in c_rows} | {row["exportName"] for row in exclusions},
        )
        self.assertEqual(28, len(c_rows) + len(exclusions))

        ready = {
            row["exportName"]
            for row in c_rows
            if row["declarationState"] in {"declared", "verified"}
        }
        blocked = {
            row["exportName"]
            for row in c_rows
            if row["declarationState"] == "blocked"
        }
        self.assertEqual(T076_READY, ready)
        self.assertEqual(T076_BLOCKED, blocked)
        self.assertEqual(T076_EXCLUDED, {row["exportName"] for row in exclusions})
        blocked_ttlv = next(
            row
            for row in c_rows
            if row["exportName"] == "kmipkit_client_execute_with_error_ttlv_v1"
        )
        self.assertIn("no approved Java/Python mapping contract", blocked_ttlv["blockedReason"])

        for row in c_rows:
            if row["declarationState"] == "blocked":
                self.assertTrue(row["blockedReason"])
                self.assertNotIn("approvalSource", row)
                self.assertNotIn("languageMappings", row)
                self.assertNotIn("signature", row)
            else:
                self.assertIn("approvalSource", row)
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(row["languageMappings"]),
                )

        for exclusion in exclusions:
            self.assertEqual("c-kmipkit", exclusion["packageId"])
            self.assertTrue(exclusion["reason"])
            self.assertIn("approvalSource", exclusion)
            if exclusion["exportName"] in {
                "kmipkit_client_batch_item_response_metadata_v1",
                "kmipkit_transport_config_create_v1",
            }:
                self.assertEqual(exclusion["exportName"], exclusion["cFunction"]["symbol"])

    def test_transport_input_encoding_and_source_are_exact_abi_discriminants(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit"
        }
        for path in (
            "kmipkit_transport_input_v1_t.encoding",
            "kmipkit_transport_input_v1_t.source_kind",
        ):
            with self.subTest(discriminant=path):
                self.assertFalse(path in declarations, f"discriminant {path} must be excluded")
                exclusion = exclusions.get(path)
                self.assertIsNotNone(exclusion, f"unclassified ABI discriminant: {path}")
                self.assertIn("discriminant", exclusion["reason"].lower())
                self.assertIn("KMIPKIT-0043-FR-007", exclusion["evidence"])

        parent_fields = {
            "kmipkit_transport_config_v1_t.client_certificates": (
                "kmipkit_transport::CertificateInput",
                "KMIPKIT-0013-FR-004",
            ),
            "kmipkit_transport_config_v1_t.client_private_key": (
                "kmipkit_transport::PrivateKeyInput",
                "KMIPKIT-0013-FR-004",
            ),
            "kmipkit_transport_config_v1_t.trust_certificates": (
                "kmipkit_transport::TrustSource::certificate_authorities",
                "KMIPKIT-0013-FR-003",
            ),
            "kmipkit_transport_config_v1_t.revocation_lists": (
                "kmipkit_transport::RevocationListInput",
                "KMIPKIT-0013-FR-003",
            ),
        }
        for path, (rust_target, requirement) in parent_fields.items():
            with self.subTest(parent=path):
                declaration = declarations[path]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertIn(requirement, declaration["requirementIds"])
                self.assertEqual(
                    rust_target,
                    declaration["languageMappings"]["rust"]["name"],
                )

    def test_c_transport_and_result_fields_are_semantically_mapped_or_exactly_excluded(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit"
        }

        self.assertEqual(26, len(C_SEMANTIC_FIELDS))
        self.assertEqual(12, len(C_ABI_FIELD_EXCLUSIONS))
        self.assertEqual(C_SEMANTIC_FIELDS, set(C_FIELD_TARGETS))
        self.assertTrue(
            C_REPRESENTATION_FIELD_EXCLUSIONS.isdisjoint(declarations),
            "representation-only C fields must not also be semantic declarations",
        )
        self.assertTrue(
            C_REPRESENTATION_FIELD_EXCLUSIONS.issubset(exclusions),
            f"unclassified C representation fields: {sorted(C_REPRESENTATION_FIELD_EXCLUSIONS - exclusions.keys())}",
        )
        self.assertTrue(
            C_SEMANTIC_FIELDS.issubset(declarations),
            f"unmapped C semantic fields: {sorted(C_SEMANTIC_FIELDS - declarations.keys())}",
        )
        self.assertTrue(
            C_ABI_FIELD_EXCLUSIONS.issubset(exclusions),
            f"unclassified C ABI fields: {sorted(C_ABI_FIELD_EXCLUSIONS - exclusions.keys())}",
        )
        self.assertEqual(C_LAYOUT_TYPES, C_LAYOUT_TYPES.intersection(exclusions))
        self.assertTrue(
            C_REPRESENTATION_FIELD_EXCLUSIONS.issubset(exclusions),
            f"unclassified C representation fields: {sorted(C_REPRESENTATION_FIELD_EXCLUSIONS - exclusions.keys())}",
        )
        for name in C_LAYOUT_TYPES:
            self.assertEqual(name, exclusions[name]["cType"]["name"])

        for path in C_SEMANTIC_FIELDS:
            with self.subTest(field=path):
                declaration = declarations[path]
                self.assertEqual("field", declaration["kind"])
                self.assertEqual("declared", declaration["declarationState"])
                self.assertIn(
                    declaration["capabilityId"],
                    {"client_execution", "production_transport"},
                )
                self.assertTrue(declaration["requirementIds"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(declaration["languageMappings"]),
                )
                self.assertEqual(path, declaration["languageMappings"]["c"]["name"])
                rust_target, java_target, python_target = C_FIELD_TARGETS[path]
                self.assertEqual(
                    rust_target,
                    (
                        declaration["languageMappings"]["rust"]["packageId"],
                        declaration["languageMappings"]["rust"]["name"],
                    ),
                )
                self.assertEqual(java_target, declaration["languageMappings"]["java"]["name"])
                self.assertEqual(python_target, declaration["languageMappings"]["python"]["name"])
                if path in {
                    "kmipkit_transport_config_v1_t.endpoint",
                    "kmipkit_transport_config_v1_t.endpoint_kind",
                    "kmipkit_transport_config_v1_t.endpoint_port",
                }:
                    self.assertIn("KMIPKIT-0013-FR-007", declaration["requirementIds"])

        for path in C_ABI_FIELD_EXCLUSIONS:
            with self.subTest(exclusion=path):
                exclusion = exclusions[path]
                self.assertTrue(exclusion["reason"])
                self.assertIn("abi", exclusion["reason"].lower())
                self.assertIn("KMIPKIT-0043-FR-007", exclusion["evidence"])

        flags = exclusions["kmipkit_transport_config_v1_t.flags"]
        self.assertIn("tls_server_name", flags["reason"])
        self.assertIn("target_uri", flags["reason"])
        self.assertIn("KMIPKIT-0013-FR-", flags["evidence"])
        for path in (
            "kmipkit_transport_config_v1_t.client_certificate_count",
            "kmipkit_transport_config_v1_t.trust_certificate_count",
            "kmipkit_transport_config_v1_t.revocation_list_count",
        ):
            self.assertIn("length", exclusions[path]["reason"].lower())
            self.assertIn("KMIPKIT-0043-FR-007", exclusions[path]["evidence"])
        input_data = exclusions["kmipkit_transport_input_v1_t.data"]
        self.assertIn("byte span", input_data["reason"].lower())
        self.assertIn("KMIPKIT-0043-FR-007", input_data["evidence"])

        last_error = declarations["kmipkit_last_error_get"]
        self.assertEqual("c-symbol", last_error["kind"])


    def test_transport_input_mappings_cover_specified_encodings_and_sources(self) -> None:
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
        }
        mappings = {
            path: C_FIELD_TARGETS[path]
            for path in (
                "kmipkit_transport_config_v1_t.client_certificates",
                "kmipkit_transport_config_v1_t.client_private_key",
                "kmipkit_transport_config_v1_t.trust_certificates",
                "kmipkit_transport_config_v1_t.revocation_lists",
            )
        }
        rust = (ROOT / "crates/kmipkit-transport/src/config.rs").read_text(encoding="utf-8")
        java = (ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/TransportConfiguration.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings/python/src/kmipkit/client.py").read_text(encoding="utf-8")
        rust_symbols = (
            "impl CertificateInput",
            "pub fn from_pem(", "pub fn from_der(",
            "pub fn from_pem_file(", "pub fn from_der_file(",
            "impl PrivateKeyInput",
            "pub fn from_pem(", "pub fn from_der(",
            "pub fn from_pem_file(", "pub fn from_der_file(",
            "impl RevocationListInput",
            "pub fn from_pem(", "pub fn from_der(",
            "pub struct TransportConfigBuilder",
        )
        for symbol in rust_symbols:
            self.assertTrue(symbol in rust, f"missing Rust input mode declaration: {symbol}")
        for symbol in (
            "clientIdentityPem", "clientIdentityDer", "clientIdentityFiles",
            "trustCertificatesPem", "trustCertificatesDer", "trustCertificateFiles",
            "revocationListsPem", "revocationListsDer",
        ):
            self.assertIn(symbol, java)
        self.assertIn("def memory(", python)
        self.assertIn("def file(", python)
        self.assertTrue(
            "CredentialEncoding" in python,
            "Python exposes the shared PEM/DER encoding selector",
        )
        self.assertIn("if any(isinstance(item.source, Path) for item in self.revocation_lists)", python)
        self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mappings["kmipkit_transport_config_v1_t.client_certificates"][1])
        self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mappings["kmipkit_transport_config_v1_t.client_private_key"][1])
        self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mappings["kmipkit_transport_config_v1_t.trust_certificates"][1])
        self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mappings["kmipkit_transport_config_v1_t.revocation_lists"][1])
        for path, (rust_target, requirement) in {
            "kmipkit_transport_config_v1_t.client_certificates": (
                "kmipkit_transport::CertificateInput",
                "KMIPKIT-0013-FR-004",
            ),
            "kmipkit_transport_config_v1_t.client_private_key": (
                "kmipkit_transport::PrivateKeyInput",
                "KMIPKIT-0013-FR-004",
            ),
            "kmipkit_transport_config_v1_t.trust_certificates": (
                "kmipkit_transport::TrustSource::certificate_authorities",
                "KMIPKIT-0013-FR-003",
            ),
            "kmipkit_transport_config_v1_t.revocation_lists": (
                "kmipkit_transport::RevocationListInput",
                "KMIPKIT-0013-FR-003",
            ),
        }.items():
            declaration = declarations[path]
            self.assertIn(requirement, declaration["requirementIds"])
            self.assertEqual(rust_target, declaration["languageMappings"]["rust"]["name"])
            self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mappings[path][1])

        self.assertNotIn("kmipkit_transport_input_v1_t.encoding", mappings)
        self.assertNotIn("kmipkit_transport_input_v1_t.source_kind", mappings)

    def test_transport_mode_exports_map_to_the_parent_c_configuration(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        mode_exports = {
            ("rust-kmipkit-transport", "kmipkit_transport::CertificateInput::from_der"),
            ("rust-kmipkit-transport", "kmipkit_transport::CertificateInput::from_der_file"),
            ("rust-kmipkit-transport", "kmipkit_transport::CertificateInput::from_pem_file"),
            ("rust-kmipkit-transport", "kmipkit_transport::PrivateKeyInput::from_der_file"),
            ("rust-kmipkit-transport", "kmipkit_transport::PrivateKeyInput::from_pem_file"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityDer"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityFiles"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificateFiles"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificatesDer"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Encoding.DER"),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Encoding.PEM"),
            ("python-kmipkit", "kmipkit.client.CredentialEncoding.DER"),
            ("python-kmipkit", "kmipkit.client.CredentialEncoding.PEM"),
            ("python-kmipkit", "kmipkit.client.TransportInput.__init__"),
            ("python-kmipkit", "kmipkit.client.TransportInput.encoding"),
            ("python-kmipkit", "kmipkit.client.TransportInput.file"),
            ("python-kmipkit", "kmipkit.client.TransportInput.source"),
        }
        for key in mode_exports:
            with self.subTest(export=key[1]):
                declaration = declarations[key]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual(
                    "kmipkit_transport_config_v1_t",
                    declaration["languageMappings"]["c"]["name"],
                    "per-input encoding/source discriminants are represented by the parent config",
                )

        production = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "production_transport"
        )
        self.assertIn("kmipkit_transport_config_v1_t", production["mappings"]["c"])
        self.assertNotIn("kmipkit_transport_input_v1_t.encoding", production["mappings"]["c"])
        self.assertNotIn("kmipkit_transport_input_v1_t.source_kind", production["mappings"]["c"])

    def test_transport_field_mappings_cover_policy_modes_and_exact_c_carriers(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }

        # A caller-selected CA collection is a TrustSource, not a naked
        # CertificateInput. The target must name the public constructor that
        # carries that configuration meaning.
        trust_field = declarations[("c-kmipkit", "kmipkit_transport_config_v1_t.trust_certificates")]
        self.assertEqual(
            "kmipkit_transport::TrustSource::certificate_authorities",
            trust_field["languageMappings"]["rust"]["name"],
        )

        # The C timeout discriminant includes DEFAULT, while the adjacent
        # seconds/nanoseconds fields describe only BOUNDED. Keep that policy
        # distinction traceable to each facade's default selector.
        timeout_kind = declarations[("c-kmipkit", "kmipkit_timeout_limit_v1_t.kind")]
        self.assertEqual("kmipkit_transport::TimeoutPolicy", timeout_kind["languageMappings"]["rust"]["name"])
        default_selector = declarations[("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Timeout.defaultValue")]
        self.assertEqual("kmipkit_transport::TimeoutPolicy", default_selector["languageMappings"]["rust"]["name"])
        self.assertEqual("kmipkit.client.TimeoutLimit.default", default_selector["languageMappings"]["python"]["name"])

        # Every supported material mode is represented by existing public
        # constructors/selectors. CRLs intentionally have memory PEM/DER only.
        rust_exports = {
            name
            for package_id, name in declarations
            if package_id == "rust-kmipkit-transport"
        }
        for kind in ("CertificateInput", "PrivateKeyInput"):
            for mode in ("from_pem", "from_der", "from_pem_file", "from_der_file"):
                self.assertIn(f"kmipkit_transport::{kind}::{mode}", rust_exports)
        for mode in ("from_pem", "from_der"):
            self.assertIn(f"kmipkit_transport::RevocationListInput::{mode}", rust_exports)
        self.assertNotIn("kmipkit_transport::RevocationListInput::from_pem_file", rust_exports)
        self.assertNotIn("kmipkit_transport::RevocationListInput::from_der_file", rust_exports)

        java = (ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/TransportConfiguration.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings/python/src/kmipkit/client.py").read_text(encoding="utf-8")
        rust_ffi = (ROOT / "crates/kmipkit-ffi/src/extension_registry/client_abi.rs").read_text(encoding="utf-8")
        rust_transport = (ROOT / "crates/kmipkit-transport/src/config.rs").read_text(encoding="utf-8")
        rust_tests = (ROOT / "crates/kmipkit-ffi/tests/support/client_abi_tests.rs").read_text(encoding="utf-8")
        python_tests = (ROOT / "bindings/python/tests/test_client.py").read_text(encoding="utf-8")
        for symbol in (
            "clientIdentityPem", "clientIdentityDer", "clientIdentityFiles",
            "trustCertificatesPem", "trustCertificatesDer", "trustCertificateFiles",
            "revocationListsPem", "revocationListsDer",
        ):
            self.assertIn(symbol, java)
        for symbol in (
            "class CredentialEncoding(IntEnum):",
            "def memory(", "def file(", "def certificate_authorities(",
        ):
            self.assertIn(symbol, python)
        for export in (
            "kmipkit.client.CredentialEncoding.PEM",
            "kmipkit.client.CredentialEncoding.DER",
        ):
            self.assertIn(("python-kmipkit", export), declarations)
        for symbol in (
            "TRANSPORT_SOURCE_MEMORY => copy_transport_span",
            "TRANSPORT_SOURCE_FILE =>",
            "fn client_certificate_chain(",
            "fn private_key_input(",
            "fn revocation_input(",
            "fn transport_input_encoding(",
            "fn transport_input_path(",
            "let defaults = TimeoutPolicy::default();",
        ):
            self.assertIn(symbol, rust_ffi)
        self.assertIn("file_based_revocation_lists_are_rejected_before_reading", rust_tests)
        self.assertIn("test_transport_config_rejects_file_backed_revocation_lists_without_echoing_path", python_tests)
        for symbol in ("pub fn from_pem_file(", "pub fn from_der_file(", "pub fn from_pem(", "pub fn from_der("):
            self.assertIn(symbol, rust_transport)

        # FR-021 inventories the parent data carrier and its nested span
        # framing separately. `data` is not an independent capability: its
        # bytes are interpreted through source_kind, encoding, and the parent
        # configuration field. The generated C layouts still retain both
        # exact record shapes.
        c_exclusions = {
            name: row
            for (package_id, name), row in exclusions.items()
            if package_id == "c-kmipkit"
        }
        for name in (
            "kmipkit_transport_config_v1_t.client_certificate_count",
            "kmipkit_transport_config_v1_t.trust_certificate_count",
            "kmipkit_transport_config_v1_t.revocation_list_count",
        ):
            self.assertIn(name, c_exclusions)
            self.assertIn("length", c_exclusions[name]["reason"].lower())
        self.assertNotIn(("c-kmipkit", "kmipkit_transport_input_v1_t.data"), declarations)
        self.assertIn(("c-kmipkit", "kmipkit_transport_input_v1_t.data"), exclusions)
        self.assertIn(("c-kmipkit", "kmipkit_byte_span_v1_t.data"), exclusions)
        self.assertIn(("c-kmipkit", "kmipkit_byte_span_v1_t.length"), exclusions)
        input_layout = c_exclusions["kmipkit_transport_input_v1_t"]["cType"]["fields"]
        self.assertEqual(
            [("source_kind", "uint32_t"), ("encoding", "uint32_t"), ("data", "kmipkit_byte_span_v1_t")],
            [(field["name"], field["type"]) for field in input_layout],
        )
        span_layout = c_exclusions["kmipkit_byte_span_v1_t"]["cType"]["fields"]
        self.assertEqual(
            [("data", "const uint8_t *"), ("length", "uint64_t")],
            [(field["name"], field["type"]) for field in span_layout],
        )

        # endpoint_port is only the RAW_TLS host's port. HTTPS carries its
        # authority (including any explicit port) in the absolute endpoint
        # URI; target_uri is the independent origin-form request path/query.
        endpoint_port = declarations[("c-kmipkit", "kmipkit_transport_config_v1_t.endpoint_port")]
        self.assertEqual("kmipkit_transport::Endpoint::raw_tls", endpoint_port["languageMappings"]["rust"]["name"])
        self.assertIn("TRANSPORT_ENDPOINT_HTTPS if input.endpoint_port == 0", rust_ffi)
        self.assertIn("endpointPort = 0;", java)
        self.assertIn("if port != 0 or self.tls_server_name is not None", python)
        self.assertIn("test_transport_record_supports_https_platform_trust_and_empty_optional_arrays", python_tests)
        self.assertIn("b\"https://server.kmipkit.test:8443\"", rust_tests)
        self.assertIn("assert_eq!(parsed.target_uri(), Some(\"/kms/kmip\"));", rust_tests)

    def test_endpoint_kind_mapping_names_the_constructor_selection(self) -> None:
        mapping = C_FIELD_TARGETS["kmipkit_transport_config_v1_t.endpoint_kind"]
        self.assertEqual("kmipkit_transport::Endpoint", mapping[0][1])
        self.assertEqual("org.kmipkit.extensions.TransportConfiguration.Builder", mapping[1])
        self.assertEqual("kmipkit.client.Endpoint", mapping[2])
        rust = (ROOT / "crates/kmipkit-transport/src/config.rs").read_text(encoding="utf-8")
        java = (ROOT / "bindings/java/src/main/java/org/kmipkit/extensions/TransportConfiguration.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings/python/src/kmipkit/client.py").read_text(encoding="utf-8")
        evidence = (ROOT / "specs/043-public-api-bindings/implementation-evidence.md").read_text(encoding="utf-8")
        for symbol in ("pub fn raw_tls", "pub fn https"):
            self.assertIn(symbol, rust)
        for symbol in ("public Builder rawTls(", "public Builder https("):
            self.assertIn(symbol, java)
        for symbol in ("def raw_tls(", "def https("):
            self.assertIn(symbol, python)
        for term in ("endpoint_kind", "raw_tls", "https"):
            self.assertTrue(term in evidence, f"implementation evidence omits endpoint mapping term: {term}")

    def test_c_field_targets_are_exact_ready_public_exports(self) -> None:
        declarations_by_path = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        for field_path, targets in C_FIELD_TARGETS.items():
            rust_target, java_target, python_target = targets
            for language, package_and_path in (
                ("rust", rust_target),
                ("java", ("java-kmipkit", java_target)),
                ("python", ("python-kmipkit", python_target)),
            ):
                package, path = package_and_path
                with self.subTest(field=field_path, language=language, target=path):
                    self.assertNotIn("{", path)
                    self.assertNotIn("}", path)
                    self.assertNotIn("(", path)
                    target = declarations_by_path.get((package, path))
                    self.assertIsNotNone(target, f"missing exact {language} export target: {path}")
                    self.assertIn(target["declarationState"], {"declared", "verified"})

    def test_client_batch_item_release_is_declared_for_c_error_cleanup(self) -> None:
        release = next(
            row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
            and row["exportName"] == "kmipkit_client_batch_item_release"
        )
        self.assertEqual("c-symbol", release["kind"])
        self.assertEqual("declared", release["declarationState"])
        self.assertEqual("native-handle-release", release["mappingSemantics"])
        self.assertEqual(
            {
                "rust": {
                    "packageId": "rust-kmipkit-client",
                    "name": "kmipkit_client::ClientBatchItem",
                },
                "c": {
                    "packageId": "c-kmipkit",
                    "name": "kmipkit_client_batch_item_release",
                },
                "java": {
                    "packageId": "java-kmipkit",
                    "name": "org.kmipkit.extensions.ClientBatchItem.close",
                },
                "python": {
                    "packageId": "python-kmipkit",
                    "name": "kmipkit.extensions.ClientBatchItem.close",
                },
            },
            release["languageMappings"],
        )
        self.assertIn("KMIPKIT-0043-FR-021", release["requirementIds"])
        header = (ROOT / "bindings/c/include/kmipkit.h").read_text(encoding="utf-8")
        self.assertIn(
            "void kmipkit_client_batch_item_release(kmipkit_client_batch_item_t *handle);",
            header,
        )
    def test_ready_function_mappings_resolve_to_real_four_language_apis(self) -> None:
        expected = {
            "client_batch_add_error_continuation_value": (
                "kmipkit_client::ClientBatch::with_batch_error_continuation_option",
                "kmipkit_client_batch_add_error_continuation_value",
                "org.kmipkit.extensions.ClientBatch.addErrorContinuationValue",
                "kmipkit.client.Batch.add_error_continuation_value",
            ),
            "client_batch_add_item": (
                "kmipkit_client::ClientBatch::push",
                "kmipkit_client_batch_add_item",
                "org.kmipkit.extensions.ClientBatch.add",
                "kmipkit.client.Batch.add_item",
            ),
            "client_batch_create": (
                "kmipkit_client::ClientBatch::empty",
                "kmipkit_client_batch_create",
                "org.kmipkit.extensions.ClientBatch.create",
                "kmipkit.client.Batch.create",
            ),
            "client_batch_item_with_unique_batch_item_id": (
                "kmipkit_client::ClientBatchItem::with_unique_batch_item_id",
                "kmipkit_client_batch_item_with_unique_batch_item_id",
                "org.kmipkit.extensions.ClientBatchItem.withUniqueBatchItemId",
                "kmipkit.extensions.with_unique_batch_item_id",
            ),
            "client_batch_item_response_pending_correlation_value": (
                "kmipkit_client::ClientBatchOutcome::asynchronous_correlation_value",
                "kmipkit_client_batch_item_response_pending_correlation_value",
                "org.kmipkit.extensions.ClientBatchItemResult.pendingCorrelationValue",
                "kmipkit.client.BatchItemResult.pending_correlation_value",
            ),
            "client_batch_item_response_unique_batch_item_id": (
                "kmipkit_client::ClientBatchItemResponse::unique_batch_item_id",
                "kmipkit_client_batch_item_response_unique_batch_item_id",
                "org.kmipkit.extensions.ClientBatchItemResult.uniqueBatchItemId",
                "kmipkit.client.BatchItemResult.unique_batch_item_id",
            ),
            "client_batch_response_item_at": (
                "kmipkit_client::ClientBatchResponse::get",
                "kmipkit_client_batch_response_item_at",
                "org.kmipkit.extensions.ClientBatchResponse.itemAt",
                "kmipkit.client.BatchResult.items",
            ),
            "client_batch_response_item_count": (
                "kmipkit_client::ClientBatchResponse::len",
                "kmipkit_client_batch_response_item_count",
                "org.kmipkit.extensions.ClientBatchResponse.itemCount",
                "kmipkit.client.BatchResult.items",
            ),
            "client_batch_set_asynchronous_indicator": (
                "kmipkit_client::ClientBatch::with_asynchronous_indicator",
                "kmipkit_client_batch_set_asynchronous_indicator",
                "org.kmipkit.extensions.ClientBatch.asynchronousIndicator",
                "kmipkit.client.Batch.set_asynchronous_indicator",
            ),
            "client_batch_set_client_correlation_value": (
                "kmipkit_client::ClientBatch::with_client_correlation_value",
                "kmipkit_client_batch_set_client_correlation_value",
                "org.kmipkit.extensions.ClientBatch.clientCorrelationValue",
                "kmipkit.client.Batch.set_client_correlation_value",
            ),
            "client_batch_set_order_option": (
                "kmipkit_client::ClientBatch::with_batch_order_option",
                "kmipkit_client_batch_set_order_option",
                "org.kmipkit.extensions.ClientBatch.ordered",
                "kmipkit.client.Batch.set_ordered",
            ),
            "client_batch_set_request_time_stamp": (
                "kmipkit_client::ClientBatch::with_request_time_stamp",
                "kmipkit_client_batch_set_request_time_stamp",
                "org.kmipkit.extensions.ClientBatch.requestTimestamp",
                "kmipkit.client.Batch.set_request_timestamp",
            ),
            "client_create": (
                "kmipkit_client::Client::new",
                "kmipkit_client_create",
                "org.kmipkit.extensions.Client.create",
                "kmipkit.client.Client.create",
            ),
            "client_execute": (
                "kmipkit_client::Client::execute",
                "kmipkit_client_execute",
                "org.kmipkit.extensions.Client.execute",
                "kmipkit.client.Client.execute",
            ),
        }
        functions = {record["id"]: record for record in self.manifest["functions"]}
        runtime_sources = {
            "rust": (
                ROOT / "crates" / "kmipkit-client" / "src" / "execute.rs",
                ROOT / "crates" / "kmipkit-transport" / "src" / "config.rs",
            ),
            "c": (
                ROOT / "crates" / "kmipkit-ffi" / "src" / "extension_registry" / "client_abi.rs",
                ROOT / "crates" / "kmipkit-ffi" / "src" / "extension_registry.rs",
            ),
            "java": tuple((ROOT / "bindings" / "java" / "src" / "main" / "java" / "org" / "kmipkit" / "extensions").glob("*.java")),
            "python": (ROOT / "bindings" / "python" / "src" / "kmipkit" / "client.py", ROOT / "bindings" / "python" / "src" / "kmipkit" / "extensions.py"),
        }

        for function_id, names in expected.items():
            self.assertIn(function_id, functions)
            function = functions[function_id]
            self.assertEqual(names[0], f"{function['rust']['module']}::{function['rust']['name']}")
            self.assertEqual(names[1], function["c"]["symbol"])
            self.assertEqual(names[2], f"{function['java']['package']}.{function['java']['class']}.{function['java']['method']}")
            self.assertEqual(names[3], f"{function['python']['module']}.{function['python']['function']}")
            for language, expected_name in zip(("rust", "c", "java", "python"), names):
                source_text = "\n".join(path.read_text(encoding="utf-8") for path in runtime_sources[language])
                member = expected_name.rsplit("::", 1)[-1].rsplit(".", 1)[-1]
                self.assertRegex(source_text, re.compile(rf"\b{re.escape(member)}\b"))

    def test_generated_public_header_and_c_consumer_use_the_same_t076_surface(self) -> None:
        header = generate.render_c_header(self.manifest)
        consumer = (ROOT / "bindings" / "c" / "tests" / "client_abi_consumer.c").read_text(
            encoding="utf-8"
        )
        self.assertIn('#include "kmipkit.h"', consumer)

        for export_name in T076_READY:
            declaration = next(row for row in self.declarations if row["exportName"] == export_name and row["packageId"] == "c-kmipkit")
            if declaration["kind"] == "type":
                self.assertIn(f"typedef struct {export_name} {export_name};", header)
            else:
                self.assertIn(f"{export_name}(", header)
            self.assertNotRegex(
                consumer,
                re.compile(rf"^[ \t]*(?:void|int32_t)[ \t]+{re.escape(export_name)}\s*\(", re.MULTILINE),
            )

        for export_name in T076_BLOCKED:
            self.assertNotIn(f"{export_name}(", header)

        for exclusion in self.manifest["approvedExclusions"]:
            export_name = exclusion["exportName"]
            if exclusion["packageId"] != "c-kmipkit" or export_name not in T076_EXCLUDED:
                continue
            self.assertIn(f"{export_name}(", header)

    def test_c_metadata_exclusion_matches_facade_result_values(self) -> None:
        exclusion = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["exportName"] == "kmipkit_client_batch_item_response_metadata_v1"
        )
        carrier = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["exportName"] == "kmipkit_client_batch_item_metadata_v1_t"
        )
        self.assertEqual(
            [
                {"name": "struct_size", "type": "uint32_t"},
                {"name": "abi_version", "type": "uint32_t"},
                {"name": "outcome_kind", "type": "uint32_t"},
                {"name": "result_status", "type": "uint32_t"},
                {"name": "has_result_reason", "type": "uint32_t"},
                {"name": "result_reason", "type": "uint32_t"},
            ],
            carrier["cType"]["fields"],
        )
        java = (ROOT / "bindings" / "java" / "src" / "main" / "java" / "org" / "kmipkit" / "extensions" / "ClientBatchItemResult.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings" / "python" / "src" / "kmipkit" / "client.py").read_text(encoding="utf-8")
        rust = (ROOT / "crates" / "kmipkit-client" / "src" / "execute.rs").read_text(encoding="utf-8")
        result_model = (ROOT / "crates" / "kmipkit-protocol" / "src" / "result.rs").read_text(encoding="utf-8")
        for member in ("outcomeKind", "resultStatus", "resultReason"):
            self.assertIn(member, java)
        for member in ("outcome", "result_status", "result_reason"):
            self.assertIn(member, python)
        self.assertRegex(rust, re.compile(r"pub\s+(?:const\s+)?fn\s+outcome\s*\("))
        self.assertRegex(rust, re.compile(r"pub\s+enum\s+ClientBatchOutcome\b"))
        self.assertRegex(rust, re.compile(r"pub\s+const\s+fn\s+result\s*\("))
        self.assertRegex(result_model, re.compile(r"pub\s+const\s+fn\s+status\s*\("))
        self.assertRegex(result_model, re.compile(r"pub\s+const\s+fn\s+reason\s*\("))
        self.assertIn("pub enum ClientBatchOutcome", rust)
        self.assertIn(
            "kmipkit_client_batch_item_response_metadata_v1(",
            generate.render_c_header(self.manifest),
        )

    def test_python_batch_item_result_constructor_is_an_exact_record_exclusion(self) -> None:
        constructor = "kmipkit.client.BatchItemResult.__init__"
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "python-kmipkit"
        }

        self.assertNotIn(constructor, declarations)
        exclusion = exclusions.get(constructor)
        self.assertIsNotNone(
            exclusion,
            "the Python result-record constructor needs an exact mapping or reviewed exclusion",
        )
        assert exclusion is not None
        self.assertIn("record constructor", exclusion["reason"].lower())
        self.assertIn("copied response metadata", exclusion["reason"].lower())
        self.assertEqual("specs/043-public-api-bindings/spec.md", exclusion["approvalSource"]["path"])
        self.assertIn("KMIPKIT-0043-FR-012", exclusion["evidence"])
        self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

        result_type = declarations.get("kmipkit.client.BatchItemResult")
        self.assertIsNotNone(result_type, "the result value type remains a mapped shared API")
        assert result_type is not None
        self.assertEqual("declared", result_type["declarationState"])
        for field in (
            "outcome",
            "result_status",
            "result_reason",
            "unique_batch_item_id",
            "pending_correlation_value",
        ):
            with self.subTest(field=field):
                row = declarations.get(f"kmipkit.client.BatchItemResult.{field}")
                self.assertIsNotNone(row, f"missing mapped result metadata field: {field}")
                assert row is not None
                self.assertEqual("declared", row["declarationState"])

    def test_python_native_handle_initializers_are_exact_internal_factory_exclusions(self) -> None:
        wrapper_types = {
            "kmipkit.client.Batch",
            "kmipkit.client.BatchResult",
            "kmipkit.client.Client",
            "kmipkit.ttlv.RawTag",
            "kmipkit.ttlv.Tag",
            "kmipkit.ttlv.CodecLimits",
            "kmipkit.ttlv.Value",
            "kmipkit.ttlv.Structure",
            "kmipkit.ttlv.TtlvItem",
            "kmipkit.ttlv.TtlvStructureView",
            "kmipkit.ttlv.TtlvItemView",
            "kmipkit.ttlv.TtlvValueView",
            "kmipkit.extensions.ExtensionIdentity",
            "kmipkit.extensions.Compatibility",
            "kmipkit.extensions.TtlvPath",
            "kmipkit.extensions.Discriminator",
            "kmipkit.extensions.ExtensionSchema",
            "kmipkit.extensions.ExtensionChildRule",
            "kmipkit.extensions.ExtensionOrderConstraint",
            "kmipkit.extensions.ExtensionInformation",
            "kmipkit.extensions.ExtensionDefinition",
            "kmipkit.extensions.ClientExtensionRegistry",
            "kmipkit.extensions.ClientConfiguration",
            "kmipkit.extensions.ValidatedExtensionValue",
            "kmipkit.extensions.RegisteredExtensionValue",
            "kmipkit.extensions.ClientRequestMessageExtension",
            "kmipkit.extensions.ClientBatchItem",
            "kmipkit.extensions.RegisterRequest",
            "kmipkit.extensions.ExtensionRecognition",
        }
        expected_initializers = {f"{wrapper}.__init__" for wrapper in wrapper_types}
        python_root = ROOT / "bindings" / "python" / "src" / "kmipkit"
        sources = {
            path.relative_to(python_root.parent).as_posix(): path.read_text(encoding="utf-8")
            for path in sorted(python_root.rglob("*.py"))
        }
        inventory = audit_binding_exports.inventory_python_sources(sources)
        observed = {row["exportName"] for row in inventory["declarations"]}
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "python-kmipkit"
        }

        observed_initializers = {
            path for path in observed if path in expected_initializers
        }
        self.assertEqual(expected_initializers, observed_initializers)
        matching_exclusions = {
            path for path in exclusions if path in expected_initializers
        }
        self.assertEqual(expected_initializers, matching_exclusions)
        for path in sorted(expected_initializers):
            with self.subTest(path=path):
                self.assertNotIn(path, declarations)
                exclusion = exclusions[path]
                self.assertIn("private identity token", exclusion["reason"].lower())
                self.assertIn("private factory", exclusion["reason"].lower())
                self.assertEqual(
                    "specs/043-public-api-bindings/spec.md",
                    exclusion["approvalSource"]["path"],
                )
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                self.assertIn("T024", exclusion["evidence"])

        blocked_wrappers = {
            "kmipkit.ttlv.TtlvItemView",
            "kmipkit.ttlv.TtlvValueView",
        }
        for wrapper in wrapper_types:
            with self.subTest(wrapper=wrapper):
                declaration = declarations[wrapper]
                self.assertEqual("type", declaration["kind"])
                expected_state = "blocked" if wrapper in blocked_wrappers else "declared"
                self.assertEqual(expected_state, declaration["declarationState"])
                if expected_state == "declared":
                    self.assertEqual(
                        {"rust", "c", "java", "python"},
                        set(declaration["languageMappings"]),
                    )

        factories = {
            "kmipkit.client.Batch.create",
            "kmipkit.client.Client.create",
            "kmipkit.extensions.RegisterRequest.create",
            "kmipkit.extensions.create_client_configuration",
            "kmipkit.extensions.create_client_extension_registry",
            "kmipkit.extensions.create_extension_definition",
            "kmipkit.extensions.create_extension_identity",
            "kmipkit.extensions.inspect_extension",
            "kmipkit.extensions.client_batch_item_discover_versions",
            "kmipkit.extensions.client_batch_item_register",
            "kmipkit.ttlv.codec_limits_create",
            "kmipkit.ttlv.codec_limits_defaults",
            "kmipkit.ttlv.raw_tag_from_raw",
            "kmipkit.ttlv.raw_tag_try_checked",
            "kmipkit.ttlv.ttlv_structure_create",
            "kmipkit.ttlv.ttlv_value_integer",
        }
        for path in factories:
            with self.subTest(factory=path):
                declaration = declarations[path]
                self.assertEqual("declared", declaration["declarationState"])
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(declaration["languageMappings"]),
                )
        for path in (
            "kmipkit.ttlv.ttlv_item_create",
            "kmipkit.ttlv.ttlv_structure_view",
        ):
            with self.subTest(blocked_factory=path):
                self.assertEqual("blocked", declarations[path]["declarationState"])

    def test_remaining_python_exports_are_mapped_or_exactly_excluded(self) -> None:
        python_root = ROOT / "bindings" / "python" / "src" / "kmipkit"
        sources = {
            path.relative_to(python_root.parent).as_posix(): path.read_text(encoding="utf-8")
            for path in sorted(python_root.rglob("*.py"))
        }
        inventory = audit_binding_exports.inventory_python_sources(sources)
        observed = {row["exportName"] for row in inventory["declarations"]}
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
        }
        exclusions = {
            row["exportName"]: row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "python-kmipkit"
        }

        unclassified = observed - declarations.keys() - exclusions.keys()
        self.assertEqual(set(), unclassified, "unclassified Python package exports")

        context_manager_exclusions = {
            f"kmipkit.client.{wrapper}.{member}"
            for wrapper in ("Batch", "BatchResult", "Client")
            for member in ("__enter__", "__exit__")
        }
        exception_constructor_exclusions = {
            f"kmipkit.errors.{name}.__init__"
            for name in (
                "ClientExecutionError",
                "CompatibilityMismatchError",
                "DuplicateKeyError",
                "InvalidExtensionSchemaError",
                "InvalidIdentityError",
                "InvalidInputError",
                "KmipKitError",
                "ResourceLimitError",
            )
        }
        exact_exclusions = (
            context_manager_exclusions
            | exception_constructor_exclusions
            | {"kmipkit.metadata.Version.__init__"}
        )
        for path in sorted(exact_exclusions):
            with self.subTest(excluded_export=path):
                self.assertIn(path, observed)
                self.assertNotIn(path, declarations)
                exclusion = exclusions[path]
                self.assertEqual(path, exclusion["exportName"])
                self.assertEqual(
                    "specs/043-public-api-bindings/spec.md",
                    exclusion["approvalSource"]["path"],
                )
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])
                if path in context_manager_exclusions:
                    self.assertIn("context-manager syntax", exclusion["reason"])
                    self.assertIn("close lifecycle", exclusion["reason"])
                elif path in exception_constructor_exclusions:
                    self.assertIn("exception constructor", exclusion["reason"])
                    self.assertIn("local", exclusion["reason"])
                else:
                    self.assertIn("runtime metadata", exclusion["reason"].lower())
                    self.assertIn("local runtime metadata value", exclusion["reason"].lower())

        trust_source = declarations["kmipkit.client.TrustSource"]
        self.assertEqual("type", trust_source["kind"])
        self.assertEqual("declared", trust_source["declarationState"])
        self.assertEqual(
            {
                "rust": ("rust-kmipkit-transport", "kmipkit_transport::TrustSource"),
                "c": ("c-kmipkit", "kmipkit_transport_config_t"),
                "java": (
                    "java-kmipkit",
                    "org.kmipkit.extensions.TransportConfiguration.Builder",
                ),
                "python": ("python-kmipkit", "kmipkit.client.TrustSource"),
            },
            {
                language: (target["packageId"], target["name"])
                for language, target in trust_source["languageMappings"].items()
            },
        )

        limits_constructor = declarations[
            "kmipkit.extensions.ExtensionRegistryLimits.__init__"
        ]
        self.assertEqual("python-export", limits_constructor["kind"])
        self.assertEqual("declared", limits_constructor["declarationState"])
        self.assertEqual(
            {
                "rust": ("rust-kmipkit-protocol", "kmipkit_protocol::extension::ExtensionRegistryLimits"),
                "c": ("c-kmipkit", "kmipkit_extension_registry_limits_v1_t"),
                "java": ("java-kmipkit", "org.kmipkit.extensions.ExtensionRegistryLimits"),
                "python": (
                    "python-kmipkit",
                    "kmipkit.extensions.ExtensionRegistryLimits.__init__",
                ),
            },
            {
                language: (target["packageId"], target["name"])
                for language, target in limits_constructor["languageMappings"].items()
            },
        )

        exception_category = declarations[
            "kmipkit.errors.ClientExecutionError.category"
        ]
        self.assertEqual("python-export", exception_category["kind"])
        self.assertEqual("declared", exception_category["declarationState"])
        self.assertEqual(
            {
                "rust": ("rust-kmipkit-client", "kmipkit_client::ClientError"),
                "c": ("c-kmipkit", "kmipkit_last_error_get"),
                "java": ("java-kmipkit", "org.kmipkit.ClientOperationException"),
                "python": (
                    "python-kmipkit",
                    "kmipkit.errors.ClientExecutionError",
                ),
            },
            {
                language: (target["packageId"], target["name"])
                for language, target in exception_category["languageMappings"].items()
            },
        )

    def test_transport_handle_adapter_is_backed_by_four_language_configuration_values(self) -> None:
        capability = next(
            row
            for row in self.manifest["capabilities"]
            if row["capabilityId"] == "production_transport"
        )
        for language in ("rust", "c", "java", "python"):
            self.assertTrue(capability["mappings"][language])
        exclusion = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["exportName"] == "kmipkit_transport_config_create_v1"
        )
        self.assertEqual(
            "kmipkit_transport_config_create_v1", exclusion["cFunction"]["symbol"]
        )
        java = (ROOT / "bindings" / "java" / "src" / "main" / "java" / "org" / "kmipkit" / "extensions" / "TransportConfiguration.java").read_text(encoding="utf-8")
        python = (ROOT / "bindings" / "python" / "src" / "kmipkit" / "client.py").read_text(encoding="utf-8")
        rust = (ROOT / "crates" / "kmipkit-transport" / "src" / "config.rs").read_text(encoding="utf-8")
        self.assertIn("public TransportConfiguration build()", java)
        self.assertIn("class TransportConfig:", python)
        self.assertIn("def create(", python)
        self.assertIn("kmipkit_transport_config_create_v1", python)
        self.assertIn("pub fn build(self)", rust)

    def test_transport_input_facade_exports_are_exactly_reconciled(self) -> None:
        expected_exports = {
            "java-kmipkit": {
                "org.kmipkit.extensions.TransportConfiguration.Builder.build",
                "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityDer",
                "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityFiles",
                "org.kmipkit.extensions.TransportConfiguration.Builder.https",
                "org.kmipkit.extensions.TransportConfiguration.Builder.revocationListsDer",
                "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificateFiles",
                "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificatesDer",
                "org.kmipkit.extensions.TransportConfiguration.Builder.trustPlatform",
                "org.kmipkit.extensions.TransportConfiguration.Encoding.DER",
                "org.kmipkit.extensions.TransportConfiguration.Encoding.PEM",
                "org.kmipkit.extensions.TransportConfiguration.Timeout",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.<init>",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.bounded",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.defaultValue",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.unbounded",
                "org.kmipkit.extensions.TransportConfiguration.builder",
            },
            "python-kmipkit": {
                "kmipkit.client.CredentialEncoding.DER",
                "kmipkit.client.CredentialEncoding.PEM",
                "kmipkit.client.Endpoint.__init__",
                "kmipkit.client.Endpoint.address",
                "kmipkit.client.Endpoint.https",
                "kmipkit.client.Endpoint.kind",
                "kmipkit.client.Endpoint.port",
                "kmipkit.client.EndpointKind",
                "kmipkit.client.EndpointKind.HTTPS",
                "kmipkit.client.EndpointKind.RAW_TLS",
                "kmipkit.client.TimeoutKind",
                "kmipkit.client.TimeoutKind.BOUNDED",
                "kmipkit.client.TimeoutKind.DEFAULT",
                "kmipkit.client.TimeoutKind.UNBOUNDED",
                "kmipkit.client.TimeoutLimit",
                "kmipkit.client.TimeoutLimit.__init__",
                "kmipkit.client.TimeoutLimit.bounded",
                "kmipkit.client.TimeoutLimit.default",
                "kmipkit.client.TimeoutLimit.unbounded",
                "kmipkit.client.TransportConfig.__init__",
                "kmipkit.client.TransportConfig.endpoint",
                "kmipkit.client.TransportConfig.trust",
                "kmipkit.client.TransportInput.__init__",
                "kmipkit.client.TransportInput.encoding",
                "kmipkit.client.TransportInput.file",
                "kmipkit.client.TransportInput.source",
                "kmipkit.client.TrustKind",
                "kmipkit.client.TrustKind.CERTIFICATES",
                "kmipkit.client.TrustKind.PLATFORM",
                "kmipkit.client.TrustSource.__init__",
                "kmipkit.client.TrustSource.certificates",
                "kmipkit.client.TrustSource.platform",
            },
        }
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"])
            for row in self.manifest["approvedExclusions"]
        }
        for package_id, exports in expected_exports.items():
            for export_name in sorted(exports):
                with self.subTest(package=package_id, export=export_name):
                    declaration = declarations.get((package_id, export_name))
                    self.assertNotIn(
                        (package_id, export_name),
                        exclusions,
                        "approved transport facade inputs must be mapped, not excluded",
                    )
                    self.assertIsNotNone(
                        declaration,
                        f"missing exact production transport mapping for {export_name}",
                    )
                    self.assertEqual("production_transport", declaration["capabilityId"])
                    self.assertEqual("declared", declaration["declarationState"])
                    self.assertEqual("four-language", declaration["parityScope"])
                    self.assertIn("KMIPKIT-0013-FR-001", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0013-FR-007", declaration["requirementIds"])
                    self.assertIn("KMIPKIT-0043-FR-021", declaration["requirementIds"])
                    for language in ("rust", "c", "java", "python"):
                        mapping = declaration["languageMappings"][language]
                        target = declarations.get((mapping["packageId"], mapping["name"]))
                        self.assertIsNotNone(
                            target,
                            f"{export_name} has unresolved {language} target {mapping['name']}",
                        )
                        self.assertIn(target["declarationState"], {"declared", "verified"})

        default_timeout = declarations.get(
            (
                "java-kmipkit",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.defaultValue",
            )
        )
        if default_timeout is not None:
            self.assertEqual(
                "kmipkit_transport::TimeoutPolicy",
                default_timeout["languageMappings"]["rust"]["name"],
                "DEFAULT selects the accepted policy; it is distinct from bounded and unbounded",
            )
            self.assertEqual(
                "kmipkit.client.TimeoutLimit.default",
                default_timeout["languageMappings"]["python"]["name"],
            )

    def test_transport_mapping_targets_preserve_endpoint_credential_and_timeout_modes(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        expected = {
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.https"): (
                "kmipkit_transport::Endpoint::https",
                "kmipkit_transport_config_v1_t.endpoint_kind",
                "org.kmipkit.extensions.TransportConfiguration.Builder.https",
                "kmipkit.client.Endpoint.https",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificatesDer"): (
                "kmipkit_transport::CertificateInput::from_der",
                "kmipkit_transport_input_v1_t.encoding",
                "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificatesDer",
                "kmipkit.client.CredentialEncoding.DER",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificateFiles"): (
                "kmipkit_transport::CertificateInput",
                "kmipkit_transport_input_v1_t.source_kind",
                "org.kmipkit.extensions.TransportConfiguration.Builder.trustCertificateFiles",
                "kmipkit.client.TransportInput.file",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityFiles"): (
                "kmipkit_transport::ClientIdentity",
                "kmipkit_transport_input_v1_t.source_kind",
                "org.kmipkit.extensions.TransportConfiguration.Builder.clientIdentityFiles",
                "kmipkit.client.TransportInput.file",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Builder.revocationListsDer"): (
                "kmipkit_transport::RevocationListInput::from_der",
                "kmipkit_transport_config_v1_t.revocation_lists",
                "org.kmipkit.extensions.TransportConfiguration.Builder.revocationListsDer",
                "kmipkit.client.TransportConfig.revocation_lists",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Encoding.DER"): (
                "kmipkit_transport::CertificateInput::from_der",
                "kmipkit_transport_input_v1_t.encoding",
                "org.kmipkit.extensions.TransportConfiguration.Encoding.DER",
                "kmipkit.client.CredentialEncoding.DER",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Encoding.PEM"): (
                "kmipkit_transport::CertificateInput::from_pem",
                "kmipkit_transport_input_v1_t.encoding",
                "org.kmipkit.extensions.TransportConfiguration.Encoding.PEM",
                "kmipkit.client.CredentialEncoding.PEM",
            ),
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.Timeout.defaultValue"): (
                "kmipkit_transport::TimeoutPolicy",
                "kmipkit_timeout_limit_v1_t.kind",
                "org.kmipkit.extensions.TransportConfiguration.Timeout.defaultValue",
                "kmipkit.client.TimeoutLimit.default",
            ),
            ("python-kmipkit", "kmipkit.client.Endpoint.kind"): (
                "kmipkit_transport::Endpoint",
                "kmipkit_transport_config_v1_t.endpoint_kind",
                "org.kmipkit.extensions.TransportConfiguration.Builder",
                "kmipkit.client.Endpoint.kind",
            ),
            ("python-kmipkit", "kmipkit.client.Endpoint.port"): (
                "kmipkit_transport::Endpoint::raw_tls",
                "kmipkit_transport_config_v1_t.endpoint_port",
                "org.kmipkit.extensions.TransportConfiguration.Builder.rawTls",
                "kmipkit.client.Endpoint.port",
            ),
            ("python-kmipkit", "kmipkit.client.TransportInput.file"): (
                "kmipkit_transport::TransportConfigBuilder",
                "kmipkit_transport_input_v1_t.source_kind",
                "org.kmipkit.extensions.TransportConfiguration.Builder",
                "kmipkit.client.TransportInput.file",
            ),
        }
        for (package_id, export_name), target_names in expected.items():
            with self.subTest(export=export_name):
                declaration = declarations[(package_id, export_name)]
                self.assertEqual("production_transport", declaration["capabilityId"])
                actual = tuple(
                    declaration["languageMappings"][language]["name"]
                    for language in ("rust", "c", "java", "python")
                )
                self.assertEqual(target_names, actual)

    def test_client_execution_error_taxonomy_is_public_and_mapped(self) -> None:
        expected = {
            "protocol": (8, "org.kmipkit.ProtocolException"),
            "transport": (9, "org.kmipkit.TransportException"),
            "server_result": (10, "org.kmipkit.ServerResultException"),
        }
        categories = {row["id"]: row for row in self.manifest["errorCategories"]}
        execute = next(row for row in self.manifest["functions"] if row["id"] == "client_execute")
        for category_id, (status, java_exception) in expected.items():
            with self.subTest(category=category_id):
                category = categories[category_id]
                self.assertEqual(status, category["cValue"])
                self.assertEqual(java_exception, category["javaException"])
                self.assertEqual("kmipkit.errors.ClientExecutionError", category["pythonException"])
                self.assertIn(category_id, execute["c"]["errorCategories"])
                self.assertIn(category_id, execute["java"]["errorCategories"])
                self.assertIn(category_id, execute["python"]["errorCategories"])

        header = generate.render_c_header(self.manifest)
        for name, status in (
            ("KMIPKIT_ERROR_PROTOCOL", 8),
            ("KMIPKIT_ERROR_TRANSPORT", 9),
            ("KMIPKIT_ERROR_SERVER_RESULT", 10),
        ):
            self.assertIn(
                f"#define {name} ((int32_t){status})",
                header,
                f"missing generated status macro {name}={status}",
            )

        last_error = next(
            row for row in self.manifest["functions"] if row["id"] == "ffi_last_error_get"
        )
        last_error_docs = " ".join(last_error["c"].get("documentation", []))
        self.assertIn("status 8", last_error_docs.lower())
        self.assertIn("status 9", last_error_docs.lower())
        self.assertIn("status 10", last_error_docs.lower())
        self.assertIn("delivery", last_error_docs)
        self.assertNotIn("current C surface does not execute client requests", header)

        ffi_source = (ROOT / "crates" / "kmipkit-ffi" / "src" / "extension_registry.rs").read_text(
            encoding="utf-8"
        )
        jni_source = (ROOT / "bindings" / "java" / "native" / "kmipkit_jni.cpp").read_text(
            encoding="utf-8"
        )
        python_errors = (ROOT / "bindings" / "python" / "src" / "kmipkit" / "errors.py").read_text(
            encoding="utf-8"
        )
        for category_id, (status, java_exception) in expected.items():
            rust_category = category_id.upper()
            self.assertRegex(ffi_source, rf"const ERROR_{rust_category}: i32 = {status};")
            self.assertRegex(
                ffi_source,
                rf"const ERROR_CATEGORY_{rust_category}: u32 = { {'protocol': 2, 'transport': 3, 'server_result': 4}[category_id] };",
            )
            java_simple_name = java_exception.rsplit(".", 1)[-1]
            self.assertIn(java_simple_name, jni_source)
            python_category = {"protocol": 2, "transport": 3, "server_result": 4}[category_id]
            self.assertRegex(
                python_errors,
                rf"\b{rust_category}\s*=\s*{python_category}\b",
            )

    def test_per_exchange_request_options_use_additive_versioned_abi(self) -> None:
        """Per-exchange timeout overrides must not replace the stable execute ABI."""
        legacy = next(
            row for row in self.manifest["functions"] if row["id"] == "client_execute"
        )
        options_execute = next(
            (
                row
                for row in self.manifest["functions"]
                if row["id"] == "client_execute_with_options"
            ),
            None,
        )
        self.assertIsNotNone(
            options_execute,
            "T076 needs an additive versioned C execute path for RequestOptions",
        )

        self.assertEqual("kmipkit_client_execute", legacy["c"]["symbol"])
        self.assertEqual(
            ["client", "batch", "limits", "out_response"],
            [parameter["name"] for parameter in legacy["c"]["parameters"]],
        )
        self.assertEqual(
            "kmipkit_client_execute_with_options_v1",
            options_execute["c"]["symbol"],
        )
        self.assertEqual(
            "kmipkit_client::Client::execute_with_options",
            f"{options_execute['rust']['module']}::{options_execute['rust']['name']}",
        )
        self.assertEqual("executeWithOptions", options_execute["java"]["method"])
        self.assertEqual("Client.execute_with_options", options_execute["python"]["function"])

        header = generate.render_c_header(self.manifest)
        self.assertIn("kmipkit_request_options_v1_t", header)
        self.assertIn("kmipkit_client_execute_with_options_v1", header)
        generic_ttlv = next(
            row
            for row in self.manifest["functions"]
            if row["c"]["symbol"] == "kmipkit_client_execute_with_error_ttlv_v1"
        )
        self.assertEqual("blocked", generic_ttlv["declarationState"])
        self.assertNotIn("kmipkit_client_execute_with_error_ttlv_v1(", header)

    def test_request_options_type_maps_all_four_timeout_overrides(self) -> None:
        request_options = next(
            (row for row in self.manifest["types"] if row["id"] == "RequestOptions"),
            None,
        )
        self.assertIsNotNone(
            request_options,
            "RequestOptions must be a shared product type, not only a C layout carrier",
        )
        self.assertEqual("kmipkit_request_options_v1_t", request_options["c"]["name"])
        self.assertEqual("fixed-width-value", request_options["c"]["kind"])
        self.assertTrue(request_options["c"]["versioned"])
        self.assertEqual(
            [
                "struct_size",
                "abi_version",
                "connect_timeout",
                "write_timeout",
                "read_timeout",
                "total_timeout",
            ],
            [field["name"] for field in request_options["c"]["fields"]],
        )

        expected_fields = {
            "connect_timeout": "with_connect",
            "write_timeout": "with_write",
            "read_timeout": "with_read",
            "total_timeout": "with_total",
        }
        declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "c-kmipkit"
        }
        for field, builder_method in expected_fields.items():
            path = f"kmipkit_request_options_v1_t.{field}"
            with self.subTest(field=field):
                declaration = declarations[path]
                self.assertEqual("field", declaration["kind"])
                self.assertEqual("client_execution", declaration["capabilityId"])
                self.assertIn("KMIPKIT-0013-FR-011", declaration["requirementIds"])
                self.assertEqual(
                    f"kmipkit_transport::RequestOptions::{builder_method}",
                    declaration["languageMappings"]["rust"]["name"],
                )
                self.assertEqual(
                    f"org.kmipkit.extensions.RequestOptions.{field.removesuffix('_timeout')}Timeout",
                    declaration["languageMappings"]["java"]["name"],
                )
                self.assertEqual(
                    f"kmipkit.client.RequestOptions.{field}",
                    declaration["languageMappings"]["python"]["name"],
                )

    def test_last_error_get_is_mapped_to_shared_error_taxonomy(self) -> None:
        declarations = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        declaration = declarations.get("kmipkit_last_error_get")
        self.assertIsNotNone(
            declaration,
            "the C last-error getter exposes the shared category, cause, and delivery taxonomy",
        )
        self.assertEqual("declared", declaration["declarationState"])
        self.assertEqual("client_execution", declaration["capabilityId"])
        self.assertIn("KMIPKIT-0043-FR-013", declaration["requirementIds"])
        self.assertEqual(
            {"rust", "c", "java", "python"}, set(declaration["languageMappings"])
        )
        self.assertEqual(
            {
                "rust": "kmipkit_client::ClientError",
                "c": "kmipkit_last_error_get",
                "java": "org.kmipkit.ClientOperationException",
                "python": "kmipkit.errors.ClientExecutionError",
            },
            {language: mapping["name"] for language, mapping in declaration["languageMappings"].items()},
        )

        declarations_by_path = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        for path in (
            "org.kmipkit.ClientOperationException.statusCode",
            "org.kmipkit.ClientOperationException.categoryCode",
            "org.kmipkit.ClientOperationException.causeCode",
            "org.kmipkit.ClientOperationException.deliveryStateCode",
        ):
            with self.subTest(java_error_accessor=path):
                self.assertEqual("declared", declarations_by_path[path]["declarationState"])
                self.assertIn("KMIPKIT-0043-FR-013", declarations_by_path[path]["requirementIds"])

        field_targets = {
            "status": (
                "kmipkit_client::ClientError",
                "org.kmipkit.ClientOperationException.statusCode",
            ),
            "client_category": (
                "kmipkit_client::ClientError::category",
                "org.kmipkit.ClientOperationException.categoryCode",
            ),
            "cause": (
                "kmipkit_client::ClientError::cause_category",
                "org.kmipkit.ClientOperationException.causeCode",
            ),
            "delivery": (
                "kmipkit_client::ClientError::delivery_state",
                "org.kmipkit.ClientOperationException.deliveryStateCode",
            ),
        }
        for field, (rust_target, java_target) in field_targets.items():
            with self.subTest(error_field=field):
                path = f"kmipkit.errors.ClientExecutionError.{field}"
                field_row = declarations_by_path[path]
                self.assertEqual(rust_target, field_row["languageMappings"]["rust"]["name"])
                self.assertEqual("kmipkit_last_error_get", field_row["languageMappings"]["c"]["name"])
                self.assertEqual(java_target, field_row["languageMappings"]["java"]["name"])

        for category_id, java_exception in (
            ("protocol", "org.kmipkit.ProtocolException"),
            ("transport", "org.kmipkit.TransportException"),
            ("server_result", "org.kmipkit.ServerResultException"),
        ):
            status_name = {
                "protocol": "KMIPKIT_ERROR_PROTOCOL",
                "transport": "KMIPKIT_ERROR_TRANSPORT",
                "server_result": "KMIPKIT_ERROR_SERVER_RESULT",
            }[category_id]
            variant = {
                "protocol": "Protocol",
                "transport": "Transport",
                "server_result": "ServerResult",
            }[category_id]
            status_declaration = declarations_by_path[status_name]
            self.assertEqual(
                f"kmipkit_client::ClientError::{variant}",
                status_declaration["languageMappings"]["rust"]["name"],
            )
            self.assertEqual(java_exception, status_declaration["languageMappings"]["java"]["name"])

        clear_exclusion = next(
            row
            for row in self.manifest["approvedExclusions"]
            if row["packageId"] == "c-kmipkit"
            and row["exportName"] == "kmipkit_last_error_clear"
        )
        self.assertIn("thread-local", clear_exclusion["reason"])
        self.assertIn("mechanism", clear_exclusion["reason"])

    def test_python_error_taxonomy_values_map_to_all_four_facades(self) -> None:
        declarations = {
            row["exportName"]: row for row in self.manifest["publicDeclarations"]
        }
        groups = {
            "kmipkit.errors.ClientErrorCategory": (
                "kmipkit_client::ClientError::category",
                "org.kmipkit.ClientOperationException.categoryCode",
                (
                    "VALIDATION", "PROTOCOL", "TRANSPORT", "SERVER_RESULT", "OTHER",
                ),
            ),
            "kmipkit.errors.ClientErrorCause": (
                "kmipkit_client::ClientError::cause_category",
                "org.kmipkit.ClientOperationException.causeCode",
                (
                    "NONE", "INVALID_INPUT", "PROTOCOL_INVALID_VALUE",
                    "PROTOCOL_INVALID_ENCODING", "PROTOCOL_OTHER", "TRANSPORT_IO",
                    "TRANSPORT_TLS", "TRANSPORT_HTTP", "TIMEOUT", "TRANSPORT_OTHER",
                    "OTHER",
                ),
            ),
            "kmipkit.errors.DeliveryState": (
                "kmipkit_client::ClientError::delivery_state",
                "org.kmipkit.ClientOperationException.deliveryStateCode",
                ("NONE", "NOT_SENT", "POSSIBLY_SENT", "RESPONSE_STARTED"),
            ),
        }
        for python_type, (rust_target, java_target, members) in groups.items():
            type_row = declarations.get(python_type)
            self.assertIsNotNone(type_row, f"missing Python taxonomy type {python_type}")
            for member in members:
                path = f"{python_type}.{member}"
                with self.subTest(value=path):
                    row = declarations.get(path)
                    self.assertIsNotNone(row, f"missing public taxonomy value {path}")
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual("client_execution", row["capabilityId"])
                    self.assertIn("KMIPKIT-0043-FR-013", row["requirementIds"])
                    self.assertEqual(rust_target, row["languageMappings"]["rust"]["name"])
                    self.assertEqual("kmipkit_last_error_get", row["languageMappings"]["c"]["name"])
                    self.assertEqual(java_target, row["languageMappings"]["java"]["name"])
                    self.assertEqual(path, row["languageMappings"]["python"]["name"])

    def test_client_response_byte_spans_document_their_borrowed_lifetime(self) -> None:
        functions = {row["id"]: row for row in self.manifest["functions"]}
        header = generate.render_c_header(self.manifest)
        for function_id in (
            "client_batch_item_response_unique_batch_item_id",
            "client_batch_item_response_pending_correlation_value",
        ):
            with self.subTest(function=function_id):
                documentation = " ".join(functions[function_id]["c"].get("documentation", []))
                self.assertIn("borrowed", documentation.lower())
                self.assertIn("item-response handle", documentation.lower())
                self.assertIn("closing the parent batch-response handle", documentation.lower())
                self.assertIn("valid and unchanged only while", documentation.lower())
                for line in functions[function_id]["c"]["documentation"]:
                    self.assertIn(line, header)

        architecture = " ".join(
            (ROOT / "docs" / "architecture" / "ffi-and-bindings.md")
            .read_text(encoding="utf-8")
            .split()
        ).lower()
        self.assertIn("clientbatchitemresponse", architecture)
        self.assertIn("retains the response owner", architecture)
        self.assertIn("valid only while the item-response handle remains open", architecture)
        self.assertIn("copy the bytes to keep them longer", architecture)

    def test_unique_batch_item_id_documents_consumed_input_ownership(self) -> None:
        function = next(
            row
            for row in self.manifest["functions"]
            if row["id"] == "client_batch_item_with_unique_batch_item_id"
        )
        item = next(parameter for parameter in function["c"]["parameters"] if parameter["name"] == "item")
        self.assertEqual("consumed", item["ownership"])
        documentation = " ".join(function["c"].get("documentation", []))
        self.assertIn("after the item handle and identifier span have been validated", documentation)
        self.assertIn("must not use or release item", documentation)
        self.assertIn("even if the call returns an error", documentation)
        header = generate.render_c_header(self.manifest)
        for line in function["c"]["documentation"]:
            self.assertIn(line, header)

    def test_public_python_instance_fields_are_mapped_or_exactly_excluded(self) -> None:
        semantic_fields = {
            "kmipkit.errors.ClientExecutionError.status",
            "kmipkit.errors.ClientExecutionError.client_category",
            "kmipkit.errors.ClientExecutionError.cause",
            "kmipkit.errors.ClientExecutionError.delivery",
            "kmipkit.extensions.ClientConfiguration.limits",
            "kmipkit.extensions.ClientExtensionRegistry.limits",
            "kmipkit.extensions.Discriminator.path_tags",
            "kmipkit.extensions.Discriminator.scalar_type",
            "kmipkit.extensions.Discriminator.scalar_value",
            "kmipkit.extensions.ExtensionDefinition.identity",
            "kmipkit.extensions.ExtensionDefinition.discriminator",
            "kmipkit.extensions.ExtensionDefinition.information",
            "kmipkit.extensions.ExtensionDefinition.compatibility",
            "kmipkit.extensions.ExtensionDefinition.schema",
            "kmipkit.extensions.ExtensionIdentity.vendor_identifier",
            "kmipkit.extensions.ExtensionIdentity.name",
            "kmipkit.extensions.ExtensionIdentity.version",
            "kmipkit.extensions.ExtensionInformation.extension_name",
            "kmipkit.extensions.ExtensionInformation.tag",
            "kmipkit.extensions.ExtensionInformation.item_type",
            "kmipkit.extensions.ExtensionInformation.enumeration",
            "kmipkit.extensions.ExtensionInformation.attribute",
            "kmipkit.extensions.ExtensionInformation.parent_structure_tag",
            "kmipkit.extensions.ExtensionInformation.description",
            "kmipkit.extensions.RegisteredExtensionValue.identity",
            "kmipkit.extensions.TtlvPath.tags",
            "kmipkit.extensions.ValidatedExtensionValue.identity",
        }
        fields = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit" and row["kind"] == "field"
        }
        self.assertEqual(27, len(semantic_fields))
        self.assertTrue(semantic_fields.issubset(fields))
        for path in semantic_fields:
            with self.subTest(field=path):
                declaration = fields[path]
                self.assertIn(declaration["declarationState"], {"declared", "verified"})
                self.assertEqual(
                    {"rust", "c", "java", "python"},
                    set(declaration["languageMappings"]),
                )
                for mapping in declaration["languageMappings"].values():
                    self.assertTrue(mapping["name"])

        python_export_paths = {
            "kmipkit.client.BatchItemResult.pending_correlation_value",
            "kmipkit.client.BatchItemResult.unique_batch_item_id",
            "kmipkit.client.BatchResult.items",
        }
        all_declarations = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit"
        }
        for path in python_export_paths:
            with self.subTest(field=path):
                declaration = all_declarations.get(path)
                self.assertIsNotNone(declaration)
                if declaration is not None:
                    self.assertEqual("field", declaration["kind"])
                    self.assertEqual("declared", declaration["declarationState"])

        args_exclusion = next(
            (
                row
                for row in self.manifest["approvedExclusions"]
                if row["packageId"] == "python-kmipkit"
                and row["exportName"] == "kmipkit.errors.ClientExecutionError.args"
            ),
            None,
        )
        self.assertIsNotNone(args_exclusion)
        self.assertIn("BaseException", args_exclusion["reason"])

    def test_python_extension_fields_map_to_0012_facade_operations(self) -> None:
        expected = {
            "kmipkit.extensions.ClientConfiguration.limits": (
                "rust-kmipkit-client",
                "kmipkit_client::extension_registry::limits",
                "c-kmipkit",
                "kmipkit_client_extension_registry_create_v1",
                "java-kmipkit",
                "org.kmipkit.extensions.ClientExtensionRegistry.limits",
            ),
            "kmipkit.extensions.ClientExtensionRegistry.limits": (
                "rust-kmipkit-client",
                "kmipkit_client::extension_registry::limits",
                "c-kmipkit",
                "kmipkit_client_extension_registry_create_v1",
                "java-kmipkit",
                "org.kmipkit.extensions.ClientExtensionRegistry.limits",
            ),
            "kmipkit.extensions.Discriminator.path_tags": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::discriminator",
                "c-kmipkit",
                "kmipkit_extension_discriminator_create",
                "java-kmipkit",
                "org.kmipkit.extensions.Discriminator.create",
            ),
            "kmipkit.extensions.Discriminator.scalar_type": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::discriminator",
                "c-kmipkit",
                "kmipkit_extension_discriminator_create",
                "java-kmipkit",
                "org.kmipkit.extensions.Discriminator.create",
            ),
            "kmipkit.extensions.Discriminator.scalar_value": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::discriminator",
                "c-kmipkit",
                "kmipkit_extension_discriminator_create",
                "java-kmipkit",
                "org.kmipkit.extensions.Discriminator.create",
            ),
            "kmipkit.extensions.ExtensionDefinition.identity": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::identity",
                "c-kmipkit",
                "kmipkit_extension_definition_identity",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionDefinition.identity",
            ),
            "kmipkit.extensions.ExtensionDefinition.discriminator": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::extension_definition",
                "c-kmipkit",
                "kmipkit_extension_definition_create",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionDefinition.create",
            ),
            "kmipkit.extensions.ExtensionDefinition.information": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::information",
                "c-kmipkit",
                "kmipkit_extension_definition_information",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionDefinition.information",
            ),
            "kmipkit.extensions.ExtensionDefinition.compatibility": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::extension_definition",
                "c-kmipkit",
                "kmipkit_extension_definition_create",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionDefinition.create",
            ),
            "kmipkit.extensions.ExtensionDefinition.schema": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::extension_definition",
                "c-kmipkit",
                "kmipkit_extension_definition_create",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionDefinition.create",
            ),
            "kmipkit.extensions.ExtensionIdentity.vendor_identifier": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::ExtensionIdentity::vendor_identifier",
                "c-kmipkit",
                "kmipkit_extension_identity_vendor_identifier",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionIdentity.vendorIdentifier",
            ),
            "kmipkit.extensions.ExtensionIdentity.name": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::ExtensionIdentity::name",
                "c-kmipkit",
                "kmipkit_extension_identity_name",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionIdentity.name",
            ),
            "kmipkit.extensions.ExtensionIdentity.version": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::ExtensionIdentity::version",
                "c-kmipkit",
                "kmipkit_extension_identity_version",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionIdentity.version",
            ),
            "kmipkit.extensions.ExtensionInformation.extension_name": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::extension_information",
                "c-kmipkit",
                "kmipkit_extension_information_create",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.create",
            ),
            "kmipkit.extensions.ExtensionInformation.tag": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_tag",
                "c-kmipkit",
                "kmipkit_extension_information_tag",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_tag",
            ),
            "kmipkit.extensions.ExtensionInformation.item_type": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_type",
                "c-kmipkit",
                "kmipkit_extension_information_type",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_type",
            ),
            "kmipkit.extensions.ExtensionInformation.enumeration": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_enumeration",
                "c-kmipkit",
                "kmipkit_extension_information_enumeration",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_enumeration",
            ),
            "kmipkit.extensions.ExtensionInformation.attribute": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_attribute",
                "c-kmipkit",
                "kmipkit_extension_information_attribute",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_attribute",
            ),
            "kmipkit.extensions.ExtensionInformation.parent_structure_tag": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_parent_structure_tag",
                "c-kmipkit",
                "kmipkit_extension_information_parent_structure_tag",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_parent_structure_tag",
            ),
            "kmipkit.extensions.ExtensionInformation.description": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_description",
                "c-kmipkit",
                "kmipkit_extension_information_description",
                "java-kmipkit",
                "org.kmipkit.extensions.ExtensionInformation.with_description",
            ),
            "kmipkit.extensions.RegisteredExtensionValue.identity": (
                "rust-kmipkit-client",
                "kmipkit_client::extension_registry::validate_extension_value",
                "c-kmipkit",
                "kmipkit_client_extension_registry_validate",
                "java-kmipkit",
                "org.kmipkit.extensions.ClientExtensionRegistry.validateExtensionValue",
            ),
            "kmipkit.extensions.TtlvPath.tags": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::with_child_tag",
                "c-kmipkit",
                "kmipkit_ttlv_path_with_child_tag",
                "java-kmipkit",
                "org.kmipkit.extensions.TtlvPath.withChildTag",
            ),
            "kmipkit.extensions.ValidatedExtensionValue.identity": (
                "rust-kmipkit-protocol",
                "kmipkit_protocol::extension::validated_extension_value_identity",
                "c-kmipkit",
                "kmipkit_validated_extension_value_identity",
                "java-kmipkit",
                "org.kmipkit.extensions.ValidatedExtensionValue.identity",
            ),
        }
        fields = {
            row["exportName"]: row
            for row in self.manifest["publicDeclarations"]
            if row["packageId"] == "python-kmipkit" and row["kind"] == "field"
        }
        self.assertEqual(23, len(expected))
        for path, (rust_package, rust_name, c_package, c_name, java_package, java_name) in expected.items():
            with self.subTest(field=path):
                self.assertEqual(
                    {
                        "rust": {"packageId": rust_package, "name": rust_name},
                        "c": {"packageId": c_package, "name": c_name},
                        "java": {"packageId": java_package, "name": java_name},
                        "python": {"packageId": "python-kmipkit", "name": path},
                    },
                    fields[path]["languageMappings"],
                )
                self.assertEqual("declared", fields[path]["declarationState"])
                self.assertIn("KMIPKIT-0012-FR-010", fields[path]["requirementIds"])
                self.assertIn("KMIPKIT-0043-FR-021", fields[path]["requirementIds"])
                self.assertEqual(
                    "specs/012-vendor-extension-registry/spec.md",
                    fields[path]["approvalSource"]["path"],
                )
        self.assertNotIn(
            "org.kmipkit.extensions.Discriminator.path_tags",
            {row["exportName"] for row in self.manifest["publicDeclarations"]},
        )

    def test_java_client_outcomes_and_owned_handle_lifecycles_are_accounted(self) -> None:
        declarations = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["publicDeclarations"]
        }
        exclusions = {
            (row["packageId"], row["exportName"]): row
            for row in self.manifest["approvedExclusions"]
        }

        outcome_mappings = {
            "org.kmipkit.extensions.BatchOutcomeKind": (
                "type",
                "kmipkit_client::ClientBatchItemResponse::outcome",
                "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
                "org.kmipkit.extensions.BatchOutcomeKind",
                "kmipkit.client.BatchOutcome",
            ),
            "org.kmipkit.extensions.BatchOutcomeKind.<init>": (
                "java-member",
                "kmipkit_client::ClientBatchItemResponse::outcome",
                "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
                "org.kmipkit.extensions.BatchOutcomeKind.<init>",
                "kmipkit.client.BatchOutcome",
            ),
            "org.kmipkit.extensions.BatchOutcomeKind.raw": (
                "java-member",
                "kmipkit_client::ClientBatchItemResponse::outcome",
                "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
                "org.kmipkit.extensions.BatchOutcomeKind.raw",
                "kmipkit.client.BatchItemResult.outcome",
            ),
            "org.kmipkit.extensions.BatchOutcomeKind.COMPLETED_VALUE": (
                "java-member",
                "kmipkit_client::ClientBatchItemResponse::outcome",
                "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
                "org.kmipkit.extensions.BatchOutcomeKind.COMPLETED_VALUE",
                "kmipkit.client.BatchOutcome.COMPLETED",
            ),
            "org.kmipkit.extensions.BatchOutcomeKind.PENDING_VALUE": (
                "java-member",
                "kmipkit_client::ClientBatchItemResponse::outcome",
                "kmipkit_client_batch_item_metadata_v1_t.outcome_kind",
                "org.kmipkit.extensions.BatchOutcomeKind.PENDING_VALUE",
                "kmipkit.client.BatchOutcome.PENDING",
            ),
        }
        for path, (kind, rust_name, c_name, java_name, python_name) in outcome_mappings.items():
            with self.subTest(outcome=path):
                row = declarations.get(("java-kmipkit", path))
                self.assertIsNotNone(row, f"unaccounted Java outcome export: {path}")
                self.assertEqual(kind, row["kind"])
                self.assertEqual("declared", row["declarationState"])
                self.assertIn("KMIPKIT-0043-FR-012", row["requirementIds"])
                self.assertEqual(
                    {
                        "rust": {"packageId": "rust-kmipkit-client", "name": rust_name},
                        "c": {"packageId": "c-kmipkit", "name": c_name},
                        "java": {"packageId": "java-kmipkit", "name": java_name},
                        "python": {"packageId": "python-kmipkit", "name": python_name},
                    },
                    row["languageMappings"],
                )

        for path in (
            "org.kmipkit.extensions.BatchOutcomeKind.isCompleted",
            "org.kmipkit.extensions.BatchOutcomeKind.isPending",
        ):
            with self.subTest(convenience=path):
                exclusion = exclusions.get(("java-kmipkit", path))
                self.assertIsNotNone(exclusion, f"unclassified Java convenience export: {path}")
                self.assertIn("predicate", exclusion["reason"].lower())
                self.assertIn("KMIPKIT-0043-FR-012", exclusion["evidence"])
                self.assertIn("KMIPKIT-0043-FR-021", exclusion["evidence"])

        for path in (
            "org.kmipkit.extensions.ClientBatchItemResult.close",
            "org.kmipkit.extensions.TransportConfiguration.close",
        ):
            with self.subTest(adapter=path):
                exclusion = exclusions.get(("java-kmipkit", path))
                self.assertIsNotNone(exclusion, f"unclassified Java lifecycle adapter: {path}")
                self.assertTrue(exclusion["reason"])
                self.assertIn("KMIPKIT-0043-FR-007", exclusion["evidence"])

        lifecycle_mappings = (
            (
                "client_execution",
                "kmipkit_client::Client",
                "kmipkit_client_release",
                "org.kmipkit.extensions.Client.close",
                "kmipkit.client.Client.close",
            ),
            (
                "client_batch_model",
                "kmipkit_client::ClientBatch",
                "kmipkit_client_batch_release",
                "org.kmipkit.extensions.ClientBatch.close",
                "kmipkit.client.Batch.close",
            ),
            (
                "client_execution",
                "kmipkit_client::ClientBatchResponse",
                "kmipkit_client_batch_response_release",
                "org.kmipkit.extensions.ClientBatchResponse.close",
                "kmipkit.client.BatchResult.close",
            ),
        )
        for capability, rust_type, c_release, java_close, python_close in lifecycle_mappings:
            for package_id, path in (
                ("c-kmipkit", c_release),
                ("java-kmipkit", java_close),
                ("python-kmipkit", python_close),
            ):
                with self.subTest(lifecycle=path):
                    row = declarations.get((package_id, path))
                    self.assertIsNotNone(row, f"unaccounted lifecycle export: {path}")
                    self.assertEqual("declared", row["declarationState"])
                    self.assertEqual(capability, row["capabilityId"])
                    self.assertEqual("native-handle-release", row["mappingSemantics"])
                    self.assertEqual(
                        {
                            "rust": {"packageId": "rust-kmipkit-client", "name": rust_type},
                            "c": {"packageId": "c-kmipkit", "name": c_release},
                            "java": {"packageId": "java-kmipkit", "name": java_close},
                            "python": {"packageId": "python-kmipkit", "name": python_close},
                        },
                        row["languageMappings"],
                    )
                    self.assertNotIn((package_id, path), exclusions)

        config_close = exclusions.get(
            ("java-kmipkit", "org.kmipkit.extensions.TransportConfiguration.close")
        )
        self.assertIsNotNone(config_close)
        self.assertIn("transport config release", config_close["reason"].lower())
        self.assertIn("KMIPKIT-0043-FR-007", config_close["evidence"])



if __name__ == "__main__":
    unittest.main()
