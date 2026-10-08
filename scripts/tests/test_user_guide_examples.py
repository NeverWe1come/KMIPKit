"""Tests for compiling the Rust examples published in the client guides."""

from __future__ import annotations

import importlib.util
import tempfile
import unittest
from pathlib import Path


REPOSITORY_ROOT = Path(__file__).resolve().parents[2]
RUNNER_PATH = REPOSITORY_ROOT / "scripts" / "test_user_guide_examples.py"
SPEC = importlib.util.spec_from_file_location("user_guide_examples_runner", RUNNER_PATH)
if SPEC is None or SPEC.loader is None:
    raise RuntimeError("the user-guide example runner module could not be loaded")
runner = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runner)


class ExampleExtractionTests(unittest.TestCase):
    def test_extracts_only_marked_rust_fences_in_document_order(self) -> None:
        source = Path("guide.md")
        markdown = """# Guide

```rust,kmipkit-test
fn first() {}
```

```rust
fn ignored() {}
```

```rust,kmipkit-test
fn second() {}
```
"""

        examples = runner.extract_examples(markdown, source)

        self.assertEqual(
            [(example.source, example.line, example.code) for example in examples],
            [
                (source, 3, "fn first() {}"),
                (source, 11, "fn second() {}"),
            ],
        )

    def test_rejects_an_unclosed_marked_rust_fence(self) -> None:
        with self.assertRaisesRegex(ValueError, r"guide\.md:1: unclosed"):
            runner.extract_examples("```rust,kmipkit-test\nfn main() {}\n", Path("guide.md"))

    def test_rejects_an_empty_marked_rust_fence(self) -> None:
        with self.assertRaisesRegex(ValueError, r"guide\.md:1: empty"):
            runner.extract_examples(
                "```rust,kmipkit-test\n\n```\n",
                Path("guide.md"),
            )


class ExampleProjectTests(unittest.TestCase):
    def test_each_example_becomes_a_separate_bin_and_uses_the_workspace_lock(self) -> None:
        source = Path("client-execution.md")
        examples = runner.extract_examples(
            "```rust,kmipkit-test\nfn main() { assert_eq!(2, 2); }\n```\n"
            "```rust,kmipkit-test\nfn main() { assert_eq!(3, 3); }\n```\n",
            source,
        )
        with tempfile.TemporaryDirectory() as temporary:
            project = Path(temporary) / "examples"
            runner.write_project(REPOSITORY_ROOT, project, examples)

            manifest = (project / "Cargo.toml").read_text(encoding="utf-8")
            self.assertIn('name = "guide_example_0001"', manifest)
            self.assertIn('name = "guide_example_0002"', manifest)
            self.assertIn(
                f'kmipkit-client = {{ path = "{(REPOSITORY_ROOT / "crates" / "kmipkit-client").as_posix()}" }}',
                manifest,
            )
            self.assertEqual(
                (project / "src" / "bin" / "guide_example_0001.rs").read_text(encoding="utf-8"),
                "fn main() { assert_eq!(2, 2); }\n",
            )
            self.assertEqual(
                (project / "src" / "bin" / "guide_example_0002.rs").read_text(encoding="utf-8"),
                "fn main() { assert_eq!(3, 3); }\n",
            )
            self.assertEqual(
                (project / "Cargo.lock").read_bytes(),
                (REPOSITORY_ROOT / "Cargo.lock").read_bytes(),
            )


if __name__ == "__main__":
    unittest.main()
