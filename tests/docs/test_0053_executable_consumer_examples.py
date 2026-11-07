from __future__ import annotations

import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[2]
CONSUMER_ROOT = ROOT / "tests/consumers/0053-parity"
CONSUMERS = {
    "rust": CONSUMER_ROOT / "rust/src/main.rs",
    "c": CONSUMER_ROOT / "c/main.c",
    "java": CONSUMER_ROOT / "java/src/main/java/Consumer.java",
    "python": CONSUMER_ROOT / "python/main.py",
}
GUIDES = (
    ROOT / "docs/user-guide/en/typed-bindings-and-response-data.md",
    ROOT / "docs/user-guide/es/enlaces-tipados-validacion-respuestas.md",
)
QUICKSTART = ROOT / "specs/053-api-parity-facade/quickstart.md"
MARKDOWN_LINK = re.compile(r"(?<!!)\[[^\]]+\]\(([^)]+)\)")


class ExecutableConsumerExampleTests(unittest.TestCase):
    def resolved_markdown_targets(self, markdown_path: Path) -> set[Path]:
        document = markdown_path.read_text(encoding="utf-8")
        return {
            (markdown_path.parent / target.split("#", maxsplit=1)[0]).resolve()
            for target in MARKDOWN_LINK.findall(document)
            if not target.startswith(("http://", "https://"))
            and target.split("#", maxsplit=1)[0]
        }

    def test_both_guides_link_to_quickstart_and_each_executable_consumer(self) -> None:
        expected_targets = {QUICKSTART.resolve()}
        expected_targets.update(path.resolve() for path in CONSUMERS.values())

        for guide in GUIDES:
            with self.subTest(guide=guide):
                document = guide.read_text(encoding="utf-8")
                self.assertIn("Runnable consumer examples", document)
                self.assertIn("Ejemplos ejecutables", document)
                linked_targets = self.resolved_markdown_targets(guide)
                self.assertTrue(expected_targets.issubset(linked_targets))

    def test_quickstart_links_all_sources_and_documents_platform_runners(self) -> None:
        document = QUICKSTART.read_text(encoding="utf-8")
        linked_targets = self.resolved_markdown_targets(QUICKSTART)

        self.assertTrue({path.resolve() for path in CONSUMERS.values()}.issubset(linked_targets))
        self.assertIn("tests/consumers/0053-parity/verify.ps1", document)
        self.assertIn("tests/consumers/0053-parity/verify.sh", document)
        self.assertIn("no KMIP server", document)
        self.assertIn("synthetic", document)

    def test_each_consumer_exercises_its_package_capabilities(self) -> None:
        required_markers = {
            "rust": (
                "TypedAttribute",
                "ManagedObject",
                "CryptographicParameters",
                "Authentication",
                "validation_report",
                "ClientBatchItem::ping",
                "ResponseMessage::try_from_ttlv",
            ),
            "c": (
                "kmipkit_typed_attribute_from_value",
                "kmipkit_opaque_object_from_fields",
                "kmipkit_cryptographic_parameters_from_fields_v1",
                "kmipkit_client_batch_set_authentication",
                "kmipkit_client_batch_item_validation_report",
                "kmipkit_client_batch_item_ping",
                "kmipkit_response_message_from_ttlv",
            ),
            "java": (
                "TypedAttribute",
                "ManagedObject",
                "CryptographicParameters",
                "Authentication",
                "ValidationReport",
                "ClientBatchItem.ping",
                "ResponseMessage.fromTtlv",
            ),
            "python": (
                "TypedAttribute",
                "ManagedObject",
                "CryptographicParameters",
                "Authentication",
                "validation.validate_request",
                "client_batch_item_ping",
                "ResponseMessage.from_ttlv",
            ),
        }

        for language, path in CONSUMERS.items():
            source = path.read_text(encoding="utf-8")
            for marker in required_markers[language]:
                with self.subTest(language=language, marker=marker):
                    self.assertIn(marker, source)


if __name__ == "__main__":
    unittest.main()
