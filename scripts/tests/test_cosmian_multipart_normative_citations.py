"""Regression tests for KMIP multipart source citations in the Cosmian integration evidence."""

import html
import re
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
OASIS_SPEC = (
    REPOSITORY_ROOT
    / "specification"
    / "oasis"
    / "kmip-2.1"
    / "upstream"
    / "kmip-spec-v2.1-os.html"
)
SPEC_DIR = REPOSITORY_ROOT / "specs" / "015-cosmian-kms-integration"


def _normalized_source_text() -> str:
    source = OASIS_SPEC.read_bytes().decode("latin-1")
    return re.sub(r"\s+", " ", html.unescape(re.sub(r"<[^>]+>", " ", source)))


def _paragraph_starting(path: Path, prefix: str) -> str:
    lines = path.read_text(encoding="utf-8").splitlines()
    start = next(index for index, line in enumerate(lines) if line.startswith(prefix))
    end = start + 1
    while end < len(lines) and lines[end].strip():
        end += 1
    return " ".join(lines[start:end])


class CosmianMultipartNormativeCitationTests(unittest.TestCase):
    def test_pinned_source_locates_multipart_sequence_hash_payload_and_correlation(self) -> None:
        source = _normalized_source_text()

        self.assertIn("2.1 Certificate", source)
        self.assertIn("Multi-part cryptographic operations", source)
        self.assertIn("For multi-part cryptographic operations the following sequence is performed", source)
        self.assertIn("Yes for single-part. No for multi-part.", source)
        self.assertIn("server-generated correlation value SHALL be specified in any subsequent", source)

    def test_project_multipart_hash_evidence_cites_the_actual_source_sections(self) -> None:
        spec_path = SPEC_DIR / "spec.md"
        protocol_path = REPOSITORY_ROOT / "crates/kmipkit-protocol/src/cryptographic_operation.rs"
        contexts = {
            "normative source summary": _paragraph_starting(spec_path, "Normative operation payloads"),
            "FR-017": _paragraph_starting(spec_path, "- **KMIPKIT-0015-FR-017**"),
            "plan amendment": _paragraph_starting(SPEC_DIR / "plan.md", "FR-017 permits caller-provided"),
            "research analysis": _paragraph_starting(SPEC_DIR / "research.md", "The multipart follow-up"),
            "T046 evidence": _paragraph_starting(SPEC_DIR / "tasks.md", "- [x] T046"),
            "traceability summary": _paragraph_starting(SPEC_DIR / "traceability.md", "The live matrix records"),
            "task report": _paragraph_starting(SPEC_DIR / "task-6-report.md", "The 81 crypto response-validation failures"),
            "Cosmian README": _paragraph_starting(REPOSITORY_ROOT / "tests/integration/cosmian/README.md", "The targeted Hash follow-up"),
            "protocol rustdoc": _paragraph_starting(protocol_path, "/// Multipart sequencing"),
        }

        for label, context in contexts.items():
            with self.subTest(context=label):
                self.assertNotIn("§2.1", context)
                self.assertIn("§6.1", context)
                self.assertIn("§6.1.24", context)
                self.assertIn("Table 235", context)
                self.assertIn("§7.8", context)


if __name__ == "__main__":
    unittest.main()
