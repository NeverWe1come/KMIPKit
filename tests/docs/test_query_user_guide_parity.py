from __future__ import annotations

import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
GUIDES = {
    "en": REPOSITORY_ROOT / "docs/user-guide/en/typed-bindings-and-response-data.md",
    "es": REPOSITORY_ROOT / "docs/user-guide/es/enlaces-tipados-validacion-respuestas.md",
}


class QueryUserGuideParityTests(unittest.TestCase):
    def test_each_guide_covers_t052_t053_surfaces_and_bulk_ttlv_copies(self) -> None:
        required_terms = {
            "en": (
                "## Typed attributes",
                "## Managed objects and key structures",
                "## Ownership and secret data",
                "## Cryptographic Parameters",
                "## Request Header Authentication and batch parameters",
                "## Validate before sending",
                "## Typed responses and decoded TTLV access",
                "### Typed Query response fields",
                "kmipkit_typed_attribute_from_value",
                "kmipkit_key_block_from_fields",
                "kmipkit_cryptographic_parameters_from_fields_v1",
                "kmipkit_client_batch_set_authentication",
                "kmipkit_client_batch_item_validation_report",
                "kmipkit_client_batch_item_response_raw_ttlv",
                "NotSent",
                "kmipkit_ttlv_value_view_byte_length",
                "kmipkit_ttlv_value_view_byte_at",
                "TtlvValueView.bytes()",
                "ttlv_value_view_bytes(view)",
                "bulk copy",
                "temporary native staging",
                "Java-owned byte array",
                "Python-owned immutable bytes",
                "no public C bulk-copy function",
            ),
            "es": (
                "## Atributos tipados",
                "## Objetos gestionados y estructuras de clave",
                "## Propiedad y datos secretos",
                "## Cryptographic Parameters",
                "## Authentication del Request Header y parámetros del lote",
                "## Copias en bloque de bytes TTLV",
                "## Validar antes del envío",
                "## Respuestas tipadas y acceso TTLV decodificado",
                "### Campos tipados de la respuesta Query",
                "kmipkit_typed_attribute_from_value",
                "kmipkit_key_block_from_fields",
                "kmipkit_cryptographic_parameters_from_fields_v1",
                "kmipkit_client_batch_set_authentication",
                "kmipkit_client_batch_item_validation_report",
                "kmipkit_client_batch_item_response_raw_ttlv",
                "NotSent",
                "kmipkit_ttlv_value_view_byte_length",
                "kmipkit_ttlv_value_view_byte_at",
                "TtlvValueView.bytes()",
                "ttlv_value_view_bytes(view)",
                "copia en bloque",
                "búfer nativo temporal",
                "arrays de bytes de Java",
                "bytes inmutables de Python",
                "C no publica una función de copia en bloque",
            ),
        }

        for language, path in GUIDES.items():
            with self.subTest(language=language):
                document = path.read_text(encoding="utf-8")
                for term in required_terms[language]:
                    with self.subTest(term=term):
                        self.assertTrue(
                            term in document,
                            f"{language} guide is missing required term {term!r}",
                        )

    def test_each_guide_names_typed_query_apis_for_c_java_and_python(self) -> None:
        required_apis = (
            "kmipkit_client_batch_item_response_query_ttlv",
            "kmipkit_client_batch_item_response_query_field_count",
            "kmipkit_client_batch_item_response_query_field_at",
            "ClientBatchItemResult.typedQueryResponse()",
            "BatchItemResult.typed_query_response()",
        )

        for language, path in GUIDES.items():
            with self.subTest(language=language):
                document = path.read_text(encoding="utf-8")
                for api in required_apis:
                    with self.subTest(api=api):
                        self.assertIn(api, document)

    def test_each_guide_explains_query_fallback_and_preservation_contract(self) -> None:
        required_terms = {
            "en": (
                "rawView()",
                "raw_view()",
                "unknown field",
                "unsigned value",
                "added after this KMIPKit release",
                "complete interleaved field order",
                "repeated fields",
                "KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN",
                "kmipkit_ttlv_item_release",
                "close()",
                "context manager",
                "Close Java",
            ),
            "es": (
                "rawView()",
                "raw_view()",
                "campo desconocido",
                "valor sin signo",
                "después de esta versión de KMIPKit",
                "orden intercalado completo",
                "campos repetidos",
                "KMIPKIT_QUERY_RESPONSE_FIELD_KIND_UNKNOWN",
                "kmipkit_ttlv_item_release",
                "close()",
                "context manager",
                "Cierra",
            ),
        }

        for language, path in GUIDES.items():
            with self.subTest(language=language):
                document = path.read_text(encoding="utf-8")
                for term in required_terms[language]:
                    with self.subTest(term=term):
                        self.assertIn(term, document)

    def test_local_markdown_links_in_both_guides_resolve(self) -> None:
        link_pattern = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")

        for language, path in GUIDES.items():
            with self.subTest(language=language):
                document = path.read_text(encoding="utf-8")
                for target in link_pattern.findall(document):
                    if target.startswith(("http://", "https://")):
                        continue
                    relative_path = target.split("#", maxsplit=1)[0]
                    if relative_path:
                        with self.subTest(target=target):
                            self.assertTrue((path.parent / relative_path).is_file())


if __name__ == "__main__":
    unittest.main()
