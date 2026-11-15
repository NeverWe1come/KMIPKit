"""Prepare exact Java/Python counterpart rows for KMIPKIT-0053 mappings.

The prepared rows are reviewed inputs in ``additional-api-routes.json``. This
tool never edits the public manifest or generated output.
"""

from __future__ import annotations

import argparse
import copy
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any

from . import audit_binding_exports


LANGUAGE_PACKAGES = {
    "java": "java-kmipkit",
    "python": "python-kmipkit",
}
REQUIRED_LANGUAGES = ("rust", "c", "java", "python")


def _route_rank(target_name: str, route: dict[str, Any]) -> tuple[int, str, str]:
    """Prefer the Rust item whose owner/member names best describe the target."""
    parts = route["exportName"].split("::")
    rust_leaf = parts[-1].split("<", 1)[0]
    rust_owner = parts[-2].split("<", 1)[0] if len(parts) > 1 else ""
    target_parts = target_name.split(".")
    target_leaf = target_parts[-1]
    target_owner = target_parts[-2] if len(target_parts) > 1 else ""
    normalize = lambda value: "".join(character.lower() for character in value if character.isalnum())
    rust_leaf_key, target_leaf_key = normalize(rust_leaf), normalize(target_leaf)
    score = 0
    if rust_leaf_key == target_leaf_key:
        score += 8
    elif rust_leaf_key.endswith(target_leaf_key) or target_leaf_key.endswith(rust_leaf_key):
        score += 4
    if normalize(rust_owner) == normalize(target_owner):
        score += 8
    return (-score, route["exportName"], route["declarationId"])


def prepare_counterpart_declarations(
    manifest: dict[str, Any],
    route_document: dict[str, Any],
    observed_exports: dict[str, dict[str, dict[str, Any]]],
) -> list[dict[str, Any]]:
    """Return deterministic, source-observed declarations for unmapped targets."""
    declarations = manifest.get("publicDeclarations")
    resolutions = route_document.get("candidateResolutions")
    input_declarations = route_document.get("publicDeclarations")
    if not isinstance(declarations, list) or not isinstance(resolutions, list):
        raise ValueError("manifest or route document lacks public declarations")
    if not isinstance(input_declarations, list):
        raise ValueError("route document publicDeclarations must be a list")

    declarations_by_id = {
        row.get("declarationId"): row
        for row in declarations
        if isinstance(row, dict) and isinstance(row.get("declarationId"), str)
    }
    manifest_by_export = {
        (row.get("packageId"), row.get("exportName")): row
        for row in declarations
        if isinstance(row, dict)
    }
    input_by_export = {
        (row.get("packageId"), row.get("exportName")): row
        for row in input_declarations
        if isinstance(row, dict)
    }
    sources: dict[tuple[str, str], list[tuple[dict[str, Any], dict[str, Any]]]] = {}

    for route in resolutions:
        if not isinstance(route, dict) or route.get("resolution") != "mapped":
            continue
        declaration = declarations_by_id.get(route.get("declarationId"))
        if declaration is None:
            raise ValueError("mapped candidate route has no exact manifest declaration")
        mappings = route.get("languageMappings")
        if not isinstance(mappings, dict) or set(mappings) != set(REQUIRED_LANGUAGES):
            raise ValueError("mapped candidate route must name all four languages")
        for language, package_id in LANGUAGE_PACKAGES.items():
            mapping = mappings.get(language)
            if not isinstance(mapping, dict) or mapping.get("packageId") != package_id:
                raise ValueError(f"candidate route has an invalid {language} target")
            key = (package_id, mapping.get("name"))
            if not isinstance(key[1], str) or not key[1]:
                raise ValueError(f"candidate route has an empty {language} target")
            if key in manifest_by_export and manifest_by_export[key].get(
                "declarationState"
            ) in {"declared", "verified"}:
                continue
            sources.setdefault(key, []).append((declaration, route))

    prepared: list[dict[str, Any]] = []
    for (package_id, export_name), candidates in sorted(sources.items()):
        observed = observed_exports.get(package_id, {}).get(export_name)
        if not isinstance(observed, dict) or not isinstance(observed.get("kind"), str):
            raise ValueError(
                f"mapping target is not an exact observed export: {package_id} / {export_name}"
            )
        declaration, route = min(
            candidates, key=lambda pair: _route_rank(export_name, pair[1])
        )
        identifier = "0053-counterpart-" + hashlib.sha256(
            f"{package_id}\n{export_name}".encode("utf-8")
        ).hexdigest()[:24]
        row: dict[str, Any] = {
            "declarationId": identifier,
            "packageId": package_id,
            "kind": observed["kind"],
            "exportName": export_name,
            "capabilityId": route["capabilityId"],
            "requirementIds": list(declaration.get("requirementIds", [])),
            "declarationState": "verified",
            "parityScope": "four-language",
            "languageMappings": copy.deepcopy(route["languageMappings"]),
            "parityEvidenceIds": list(route.get("evidencePaths", [])),
            "approvalSource": copy.deepcopy(route.get("approvalSource")),
        }
        if package_id == "java-kmipkit" and observed.get("signatures"):
            row["javaSignatures"] = list(observed["signatures"])
        if not row["requirementIds"] or not row["parityEvidenceIds"] or not row["approvalSource"]:
            raise ValueError(f"mapping target lacks requirement, evidence, or approval: {export_name}")

        existing = input_by_export.get((package_id, export_name))
        if existing is not None and existing != row:
            raise ValueError(f"prepared counterpart differs from reviewed route: {export_name}")
        prepared.append(row)

    return prepared


def _observed_java_exports(root: Path) -> dict[str, dict[str, Any]]:
    java_root = root / "bindings/java/src/main/java"
    sources = sorted(java_root.rglob("*.java"))
    if not sources:
        raise ValueError("Java sources are missing")
    classpath = audit_binding_exports.resolve_java_compile_classpath(root)
    outputs: dict[str, str] = {}
    with tempfile.TemporaryDirectory(prefix="kmipkit-0053-javap-") as output:
        compiled = subprocess.run(
            ["javac", "--release", "17", "-classpath", classpath, "-d", output,
             *(str(path) for path in sources)],
            check=False,
            capture_output=True,
            text=True,
            encoding="utf-8",
        )
        if compiled.returncode != 0:
            raise ValueError(f"javac --release 17 failed: {compiled.stderr.strip()}")
        for class_file in sorted(Path(output).rglob("*.class")):
            binary_name = class_file.relative_to(output).with_suffix("").as_posix().replace("/", ".")
            if binary_name.endswith(("module-info", "package-info")):
                continue
            inspected = subprocess.run(
                ["javap", "-protected", "-classpath", output, binary_name],
                check=False,
                capture_output=True,
                text=True,
                encoding="utf-8",
            )
            if inspected.returncode != 0:
                raise ValueError(f"javap failed for {binary_name}: {inspected.stderr.strip()}")
            outputs[class_file.relative_to(output).as_posix()] = inspected.stdout
    report = audit_binding_exports.inventory_javap_outputs(outputs)
    if not report["complete"]:
        raise ValueError("Java export inventory is incomplete")
    return {
        row["exportName"]: row
        for row in report["declarations"]
        if row["packageId"] == "java-kmipkit"
    }


def _observed_python_exports(root: Path) -> dict[str, dict[str, Any]]:
    package_root = root / "bindings/python/src/kmipkit"
    sources = {
        path.relative_to(package_root.parent).as_posix(): path.read_text(encoding="utf-8")
        for path in sorted(package_root.rglob("*.py"))
    }
    report = audit_binding_exports.inventory_python_sources(sources)
    if not report["complete"]:
        raise ValueError("Python export inventory is incomplete")
    return {
        row["exportName"]: row
        for row in report["declarations"]
        if row["packageId"] == "python-kmipkit"
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--write", action="store_true", help="write reviewed route inputs")
    arguments = parser.parse_args()
    root = arguments.repo_root.resolve()
    manifest = json.loads((root / "specification/api/public-api.json").read_text(encoding="utf-8"))
    route_path = root / "specs/053-api-parity-facade/additional-api-routes.json"
    route_document = json.loads(route_path.read_text(encoding="utf-8"))
    observed = {
        "java-kmipkit": _observed_java_exports(root),
        "python-kmipkit": _observed_python_exports(root),
    }
    prepared = prepare_counterpart_declarations(manifest, route_document, observed)
    existing = route_document["publicDeclarations"]
    if arguments.write:
        by_export = {(row["packageId"], row["exportName"]): row for row in existing}
        for row in prepared:
            by_export.setdefault((row["packageId"], row["exportName"]), row)
        route_document["publicDeclarations"] = [
            by_export[key] for key in sorted(by_export)
        ]
        route_path.write_text(
            json.dumps(route_document, ensure_ascii=False, indent=2) + "\n",
            encoding="utf-8",
            newline="\n",
        )
    print(f"Prepared {len(prepared)} exact Java/Python counterpart declarations.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
