"""Audit public Rust declaration paths against manifest v2, without changing it.

The runner uses the existing Rust 1.94.0 toolchain, locked offline dependencies,
and rustdoc JSON format 57. Trait implementation behavior is explicitly outside
this first audit's accurate subset: it is reported as an incomplete audit,
never silently treated as complete. See ``rust-export-audit.md``.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any


TOOLCHAIN = "1.94.0"
FORMAT_VERSION = 57
PACKAGES = (
    "kmipkit-ttlv", "kmipkit-protocol", "kmipkit-transport", "kmipkit-client", "kmipkit",
)
KINDS = {
    "module": "module", "struct": "type", "enum": "type", "union": "type",
    "type_alias": "type", "trait": "trait", "function": "function",
    "constant": "constant", "static": "constant", "macro": "macro", "proc_macro": "macro",
    "variant": "variant", "struct_field": "field",
    "assoc_type": "associated-item", "assoc_const": "associated-item",
}


def _kind(item: dict[str, Any]) -> str:
    return next(iter(item["inner"]))


def _location(item: dict[str, Any]) -> dict[str, Any] | None:
    span = item.get("span")
    if span is None:
        return None
    return {
        "path": span["filename"].replace("\\", "/"),
        "line": span["begin"][0], "column": span["begin"][1],
    }


def _key(record: dict[str, Any]) -> tuple[str, str]:
    return record["packageId"], record["exportName"]


class _Inventory:
    def __init__(self, documents: dict[str, dict[str, Any]]) -> None:
        self.documents = documents
        self.records: dict[tuple[str, str], dict[str, Any]] = {}
        self.issues: list[dict[str, Any]] = []
        self.definitions: dict[tuple[str, ...], tuple[str, str]] = {}
        self.member_origins: dict[tuple[str, str], tuple[str, ...]] = {}
        for crate, doc in sorted(documents.items()):
            if doc.get("format_version") != FORMAT_VERSION or doc.get("includes_private") is not True:
                raise ValueError(f"{crate}: expected rustdoc format {FORMAT_VERSION} with private items")
            root = doc["index"][str(doc["root"])]
            if root["name"] != crate or _kind(root) != "module":
                raise ValueError(f"{crate}: unexpected rustdoc root")
            self.definitions[(crate,)] = crate, str(doc["root"])
            for identifier, path in doc["paths"].items():
                if identifier in doc["index"] and doc["index"][identifier].get("crate_id") == 0:
                    self.definitions[tuple(path["path"])] = crate, identifier
            for identifier, owner in doc["index"].items():
                if owner.get("crate_id") != 0 or _kind(owner) not in {"struct", "union", "enum", "trait"}:
                    continue
                canonical = tuple(doc["paths"].get(identifier, {}).get("path", [crate, owner["name"]]))
                self.index_member_origins(crate, owner, canonical)

    def index_member_origins(self, crate: str, owner: dict[str, Any], canonical: tuple[str, ...]) -> None:
        """Keep macro-expanded members distinct even when their source spans coincide."""
        kind = _kind(owner)
        inner = owner["inner"][kind]
        children = []
        if kind in {"struct", "variant"}:
            children = self.fields(inner["kind"])
        elif kind == "union":
            children = inner["fields"]
        elif kind == "enum":
            children = inner["variants"]
        elif kind == "trait":
            children = inner["items"]
        for impl_id in inner.get("impls", []):
            children = [*children, *self.documents[crate]["index"][str(impl_id)]["inner"]["impl"]["items"]]
        for identifier in children:
            if identifier is None:
                continue
            member = self.documents[crate]["index"].get(str(identifier))
            if member is None:
                continue
            origin = (*canonical, member["name"])
            self.member_origins[(crate, str(identifier))] = origin
            if _kind(member) == "variant":
                self.index_member_origins(crate, member, origin)

    def issue(self, package: str, name: str, reason: str, item: dict[str, Any], **details: Any) -> None:
        self.issues.append({
            "packageId": package, "exportName": name, "reason": reason,
            "location": _location(item), **details,
        })

    def resolve(self, crate: str, identifier: Any) -> tuple[str, str] | None:
        doc = self.documents[crate]
        identifier = str(identifier)
        local = doc["index"].get(identifier)
        if local is not None and local.get("crate_id") == 0:
            return crate, identifier
        path = doc["paths"].get(identifier)
        return None if path is None else self.definitions.get(tuple(path["path"]))

    def add(self, crate: str, item: dict[str, Any], package: str, path: tuple[str, ...],
            exports: tuple[dict[str, Any] | None, ...], associated: bool = False) -> None:
        kind = _kind(item)
        location = _location(item)
        origin_path = self.member_origins.get((crate, str(item["id"])))
        if origin_path is None:
            origin_path = self.documents[crate]["paths"].get(str(item["id"]), {}).get("path", [crate, item["name"]])
        identity = json.dumps([crate, location, kind, origin_path], sort_keys=True, separators=(",", ":"))
        name = "::".join(path)
        record = {
            "packageId": package, "exportName": name,
            "kind": "associated-item" if associated else KINDS[kind],
            "identity": identity, "location": location,
            "exportLocations": [e for e in exports if e is not None],
        }
        previous = self.records.get((package, name))
        if previous is not None:
            if previous["identity"] != identity:
                self.issue(package, name, "multiple-definitions-for-export-path", item)
            previous["exportLocations"].extend(record["exportLocations"])
        else:
            self.records[(package, name)] = record

    def children(self, crate: str, ids: list[Any], package: str, path: tuple[str, ...],
                 ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
                 default_public: bool = False, associated: bool = False) -> None:
        for identifier in ids:
            if identifier is None:
                continue
            item = self.documents[crate]["index"].get(str(identifier))
            if item is None:
                self.issue(package, "::".join(path), "missing-rustdoc-item", {"span": None}, itemId=str(identifier))
                continue
            if item["visibility"] == "public" or (default_public and item["visibility"] == "default"):
                self.visit(crate, str(identifier), package, path, ancestors, exports, associated=associated)

    def visit(self, crate: str, identifier: str, package: str, parent: tuple[str, ...],
              ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
              alias: str | None = None, associated: bool = False) -> None:
        item = self.documents[crate]["index"][identifier]
        kind = _kind(item)
        inner = item["inner"][kind]
        if kind == "use":
            target = self.resolve(crate, inner.get("id"))
            name = "::".join((*parent, inner["name"]))
            if target is None:
                self.issue(package, name, "unresolved-reexport", item)
                return
            target_crate, target_id = target
            target_item = self.documents[target_crate]["index"][target_id]
            export_sites = (*exports, _location(item))
            if inner["is_glob"]:
                if (target_crate, target_id) in ancestors:
                    return
                target_kind = _kind(target_item)
                if target_kind not in {"module", "enum"}:
                    self.issue(package, name, "unsupported-glob-target", item)
                    return
                values = target_item["inner"][target_kind]
                self.children(target_crate, values["items" if target_kind == "module" else "variants"],
                              package, parent, ancestors | {(target_crate, target_id)}, export_sites,
                              default_public=target_kind == "enum")
            else:
                self.visit(target_crate, target_id, package, parent, ancestors, export_sites, alias=inner["name"])
            return
        path = (*parent, alias if alias is not None else item["name"])
        if kind not in KINDS:
            self.issue(package, "::".join(path), "unsupported-item-kind", item, itemKind=kind)
            return
        self.add(crate, item, package, path, exports, associated)
        if (crate, identifier) in ancestors:
            return
        ancestors = ancestors | {(crate, identifier)}
        if kind == "module":
            self.children(crate, inner["items"], package, path, ancestors, exports)
        elif kind in {"struct", "union"}:
            fields = inner["fields"] if kind == "union" else self.fields(inner["kind"])
            self.children(crate, fields, package, path, ancestors, exports)
        elif kind == "enum":
            self.children(crate, inner["variants"], package, path, ancestors, exports, default_public=True)
        elif kind == "variant":
            self.children(crate, self.fields(inner["kind"]), package, path, ancestors, exports, default_public=True)
        elif kind == "trait":
            self.children(crate, inner["items"], package, path, ancestors, exports, default_public=True, associated=True)
        elif kind == "type_alias":
            self.issue(package, "::".join(path), "type-alias-members-not-audited", item)
        if isinstance(inner, dict):
            self.implementation_members(crate, inner.get("impls", []), package, path, ancestors, exports, item)

    def implementation_members(self, crate: str, impl_ids: list[Any], package: str, path: tuple[str, ...],
                               ancestors: frozenset[tuple[str, str]], exports: tuple[dict[str, Any] | None, ...],
                               owner: dict[str, Any]) -> None:
        """Visit inherent members and retain an honest boundary around trait behavior."""
        trait_members: set[str] = set()
        for impl_id in impl_ids:
            impl_item = self.documents[crate]["index"][str(impl_id)]
            impl = impl_item["inner"]["impl"]
            if impl.get("trait") is None:
                self.children(crate, impl["items"], package, path, ancestors, exports, associated=True)
            elif impl["items"] or impl.get("provided_trait_methods"):
                trait_members.add(impl["trait"]["path"])
        if trait_members:
            self.issue(package, "::".join(path), "trait-implementation-members-not-audited", owner,
                       traits=sorted(trait_members))

    @staticmethod
    def fields(kind: Any) -> list[Any]:
        if isinstance(kind, str):
            return []
        if "tuple" in kind:
            return kind["tuple"]
        return kind.get("plain", kind.get("struct", {})).get("fields", [])

    def run(self) -> dict[str, Any]:
        for crate, doc in sorted(self.documents.items()):
            root_id = str(doc["root"])
            root = doc["index"][root_id]
            self.children(crate, root["inner"]["module"]["items"], "rust-" + crate.replace("_", "-"),
                          (crate,), frozenset({(crate, root_id)}), ())
        aliases: dict[str, list[str]] = {}
        for record in self.records.values():
            aliases.setdefault(record["identity"], []).append(record["exportName"])
        records = sorted(self.records.values(), key=_key)
        for record in records:
            record["aliases"] = sorted(set(aliases[record["identity"]]))
            sites = {json.dumps(site, sort_keys=True): site for site in record["exportLocations"]}
            record["exportLocations"] = [sites[key] for key in sorted(sites)]
        issues = {json.dumps(issue, sort_keys=True): issue for issue in self.issues}
        return {
            "formatVersion": 1, "toolchain": TOOLCHAIN, "rustdocFormat": FORMAT_VERSION,
            "crates": sorted(self.documents), "complete": not issues,
            "declarations": records,
            "issues": sorted(issues.values(), key=lambda i: (*_key(i), i["reason"], json.dumps(i, sort_keys=True))),
        }


def inventory(documents: dict[str, dict[str, Any]]) -> dict[str, Any]:
    """Enumerate actual public paths in the supported rustdoc declaration subset."""
    return _Inventory(documents).run()


def compare(observed: dict[str, Any], manifest: dict[str, Any]) -> dict[str, Any]:
    """Compare exact package/name/kind records; blocked entries still count as inventory."""
    package_ids = {"rust-" + name.replace("_", "-") for name in observed["crates"]}
    expected: dict[tuple[str, str], dict[str, Any]] = {}
    for row in manifest["publicDeclarations"]:
        if row["packageId"] in package_ids:
            if _key(row) in expected:
                raise ValueError(f"duplicate manifest declaration: {row['exportName']}")
            expected[_key(row)] = row
    exclusions = {_key(row) for row in manifest.get("approvedExclusions", [])}
    actual = {_key(row): row for row in observed["declarations"]}
    matched, missing, excluded, wrong_kind = [], [], [], []
    for key, row in sorted(actual.items()):
        if key in expected:
            if row["kind"] == expected[key]["kind"]:
                matched.append({**row, "declarationState": expected[key]["declarationState"]})
            else:
                wrong_kind.append({**row, "manifestKind": expected[key]["kind"]})
        elif key in exclusions:
            excluded.append(row)
        else:
            missing.append(row)
    not_observed = [expected[key] for key in sorted(expected.keys() - actual.keys())]
    return {
        **observed, "matchesManifest": not (missing or not_observed or wrong_kind),
        "matched": matched, "missingFromManifest": missing, "notObserved": not_observed,
        "kindMismatches": wrong_kind, "excluded": excluded,
        "counts": {"observed": len(actual), "manifest": len(expected), "matched": len(matched),
                   "missingFromManifest": len(missing), "notObserved": len(not_observed),
                   "kindMismatches": len(wrong_kind), "excluded": len(excluded), "issues": len(observed["issues"])},
    }


def require_installed_toolchain(toolchains: str) -> None:
    """Fail before invoking a rustup proxy that could install a missing compiler."""
    if not any(line.split()[0].startswith(f"{TOOLCHAIN}-") for line in toolchains.splitlines() if line.strip()):
        raise ValueError(f"Rust {TOOLCHAIN} must be already installed; the auditor never downloads a toolchain")


def build_documents(repo_root: Path, target_dir: Path) -> dict[str, dict[str, Any]]:
    """Produce fresh JSON with an installed pinned compiler and no network fallback."""
    installed = subprocess.run(["rustup", "toolchain", "list"], check=True, capture_output=True, text=True)
    require_installed_toolchain(installed.stdout)
    version = subprocess.run(["rustc", f"+{TOOLCHAIN}", "--version"], check=True, capture_output=True, text=True)
    if not version.stdout.startswith(f"rustc {TOOLCHAIN} "):
        raise ValueError(f"the installed toolchain must be Rust {TOOLCHAIN}")
    documents = {}
    for package in PACKAGES:
        crate = package.replace("-", "_")
        environment = dict(os.environ)
        for key in ("RUSTFLAGS", "CARGO_ENCODED_RUSTFLAGS", "RUSTDOCFLAGS", "CARGO_ENCODED_RUSTDOCFLAGS",
                    "RUSTC", "RUSTDOC", "CARGO_BUILD_TARGET"):
            environment.pop(key, None)
        environment.update({"CARGO_TARGET_DIR": str(target_dir), "RUSTC_BOOTSTRAP": "1"})
        subprocess.run([
            "cargo", f"+{TOOLCHAIN}", "rustdoc", "--locked", "--offline", "--all-features", "--lib", "-p", package,
            "--", "-Z", "unstable-options", "--output-format", "json", "--document-private-items", "--document-hidden-items",
        ], cwd=repo_root, env=environment, check=True)
        documents[crate] = json.loads((target_dir / "doc" / f"{crate}.json").read_text(encoding="utf-8"))
    return documents


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--report", type=Path, help="write deterministic JSON; default: standard output")
    parser.add_argument("--check", action="store_true", help="exit 1 for inventory mismatches or incomplete audit scope")
    args = parser.parse_args(argv)
    root = args.repo_root.resolve()
    try:
        with tempfile.TemporaryDirectory(prefix="kmipkit-rust-export-audit-") as build_dir:
            documents = build_documents(root, Path(build_dir))
        manifest = json.loads((root / "specification/api/public-api.json").read_text(encoding="utf-8"))
        report = compare(inventory(documents), manifest)
        output = json.dumps(report, indent=2, sort_keys=True, ensure_ascii=False) + "\n"
        if args.report is None:
            sys.stdout.write(output)
        else:
            args.report.write_text(output, encoding="utf-8", newline="\n")
            print(json.dumps(report["counts"], sort_keys=True))
        return int(args.check and not (report["complete"] and report["matchesManifest"]))
    except (ValueError, KeyError, OSError, subprocess.CalledProcessError) as error:
        print(f"Rust export audit failed: {error}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
