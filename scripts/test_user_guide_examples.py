"""Compile explicitly marked Rust examples from the bilingual client guides."""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
import tempfile
from dataclasses import dataclass
from pathlib import Path


MARKER = "rust,kmipkit-test"
OPEN_FENCE = re.compile(r"^\s*```([^`]*)\s*$")


@dataclass(frozen=True)
class Example:
    source: Path
    line: int
    code: str


def extract_examples(markdown: str, source: Path) -> list[Example]:
    """Return marked Rust fences, rejecting empty or unterminated examples."""
    examples: list[Example] = []
    in_marked_fence = False
    in_other_fence = False
    opening_line = 0
    lines: list[str] = []

    for line_number, line in enumerate(markdown.splitlines(), start=1):
        if in_marked_fence:
            if line.strip() == "```":
                code = "\n".join(lines).strip()
                if not code:
                    raise ValueError(f"{source}:{opening_line}: empty {MARKER} example")
                examples.append(Example(source, opening_line, f"{code}\n"))
                in_marked_fence = False
                lines.clear()
            else:
                lines.append(line)
            continue

        if in_other_fence:
            if line.strip() == "```":
                in_other_fence = False
            continue

        match = OPEN_FENCE.match(line)
        if match is None:
            continue
        language = match.group(1).strip()
        if language == MARKER:
            in_marked_fence = True
            opening_line = line_number
        else:
            in_other_fence = True

    if in_marked_fence:
        raise ValueError(f"{source}:{opening_line}: unclosed {MARKER} example")
    return examples


def collect_examples(sources: list[Path]) -> list[Example]:
    """Read the configured guide files and require at least one marked block."""
    examples: list[Example] = []
    for source in sources:
        examples.extend(extract_examples(source.read_text(encoding="utf-8"), source))
    if not examples:
        raise ValueError(f"no ```{MARKER} examples were found")
    return examples


def _toml_string(value: str) -> str:
    """Quote a path as a TOML basic string."""
    escaped = value.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def write_project(root: Path, project: Path, examples: list[Example]) -> None:
    """Create one Cargo binary per example and copy the workspace lockfile."""
    binary_directory = project / "src" / "bin"
    binary_directory.mkdir(parents=True, exist_ok=True)
    repository = root.resolve()
    dependencies = {
        "kmipkit-client": repository / "crates" / "kmipkit-client",
        "kmipkit-protocol": repository / "crates" / "kmipkit-protocol",
        "kmipkit-ttlv": repository / "crates" / "kmipkit-ttlv",
    }

    manifest = [
        "[package]",
        'name = "kmipkit-guide-examples"',
        'version = "0.0.0"',
        'edition = "2024"',
        "",
        "[dependencies]",
    ]
    for name, dependency_path in dependencies.items():
        manifest.append(
            f"{name} = {{ path = {_toml_string(dependency_path.as_posix())} }}"
        )

    for index, example in enumerate(examples, start=1):
        name = f"guide_example_{index:04}"
        relative_source = Path("src") / "bin" / f"{name}.rs"
        manifest.extend(
            (
                "",
                "[[bin]]",
                f'name = "{name}"',
                f"path = {_toml_string(relative_source.as_posix())}",
            )
        )
        (project / relative_source).write_text(example.code, encoding="utf-8")

    (project / "Cargo.toml").write_text("\n".join(manifest) + "\n", encoding="utf-8")
    shutil.copyfile(repository / "Cargo.lock", project / "Cargo.lock")
    source_map = [
        f"guide_example_{index:04}: {example.source}:{example.line}"
        for index, example in enumerate(examples, start=1)
    ]
    (project / "examples.txt").write_text("\n".join(source_map) + "\n", encoding="utf-8")


def cargo_check_command(cargo: str, manifest: Path, locked: bool) -> list[str]:
    """Build a no-network Cargo check command for the generated examples."""
    command = [cargo, "check", "--offline"]
    if locked:
        command.append("--locked")
    command.extend(("--all-targets", "--manifest-path", str(manifest)))
    return command


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--guide",
        action="append",
        type=Path,
        help="guide Markdown file to scan (may be supplied more than once)",
    )
    arguments = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    sources = arguments.guide or [
        root / "docs" / "user-guide" / "en" / "client-execution.md",
        root / "docs" / "user-guide" / "en" / "encrypt-decrypt.md",
        root / "docs" / "user-guide" / "es" / "ejecucion-cliente.md",
        root / "docs" / "user-guide" / "es" / "cifrado-descifrado.md",
    ]

    try:
        examples = collect_examples(sources)
        cargo = os.environ.get("CARGO", "cargo")
        if shutil.which(cargo) is None:
            raise FileNotFoundError(f"Cargo executable not found: {cargo}")
        with tempfile.TemporaryDirectory(prefix="kmipkit-guide-examples-") as temporary:
            project = Path(temporary)
            write_project(root, project, examples)
            manifest = project / "Cargo.toml"
            result = subprocess.run(
                cargo_check_command(cargo, manifest, locked=False),
                cwd=root,
                check=False,
            )
            if result.returncode == 0:
                result = subprocess.run(
                    cargo_check_command(cargo, manifest, locked=True),
                    cwd=root,
                    check=False,
                )
        if result.returncode != 0:
            print("Rust guide examples compiled from:", file=sys.stderr)
            for example in examples:
                print(f"  {example.source}:{example.line}", file=sys.stderr)
            return result.returncode
        print(f"Compiled {len(examples)} marked Rust user-guide examples.")
        return 0
    except (OSError, ValueError) as error:
        print(f"user-guide example check failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
