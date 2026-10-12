"""Project source-owned declarations from real rustdoc v57 JSON for tests."""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any


def project(raw_path: Path, output_path: Path) -> None:
    raw = json.loads(raw_path.read_text(encoding="utf-8"))
    raw_index = raw["index"]
    root_id = str(raw["root"])
    root = raw_index[root_id]
    root_items = root["inner"]["module"]["items"]
    def source_traits(item_ids: list[Any]) -> set[str]:
        names: set[str] = set()
        for item_id in item_ids:
            item = raw_index[str(item_id)]
            inner = item["inner"]
            if "trait" in inner:
                names.add(item["name"])
            elif "module" in inner:
                names.update(source_traits(inner["module"]["items"]))
        return names

    source_trait_names = source_traits(root_items)
    selected_impls: dict[str, list[Any]] = {}
    for item_id in root_items:
        item = raw_index[str(item_id)]
        item_kind = next(iter(item["inner"]))
        if item_kind not in {"struct", "union"}:
            continue
        inner = item["inner"][item_kind]
        impl_ids = []
        for impl_id in inner.get("impls", []):
            impl = raw_index[str(impl_id)]["inner"]["impl"]
            trait = impl.get("trait")
            derived = "automatically_derived" in raw_index[str(impl_id)].get("attrs", [])
            representative_synthetic = item.get("name") == "Fixed" and trait is not None \
                and trait.get("path") == "Send" and impl.get("is_synthetic") is True
            representative_blanket = item.get("name") == "Fixed" and trait is not None \
                and trait.get("path") == "Borrow" and impl.get("blanket_impl") is not None
            if trait is None or trait.get("path") in source_trait_names or derived \
                    or representative_synthetic or representative_blanket:
                impl_ids.append(impl_id)
        selected_impls[str(item_id)] = impl_ids

    selected: dict[str, dict[str, Any]] = {}

    def include(identifier: Any) -> dict[str, Any]:
        key = str(identifier)
        if key not in selected:
            selected[key] = raw_index[key]
        return selected[key]

    projected_root = include(root_id)
    projected_root["inner"]["module"]["items"] = root_items
    for item_id in root_items:
        item = include(item_id)
        kind = next(iter(item["inner"]))
        inner = item["inner"][kind]
        if kind == "module":
            for nested_id in inner["items"]:
                nested = include(nested_id)
                if "trait" in nested["inner"]:
                    for member_id in nested["inner"]["trait"]["items"]:
                        include(member_id)
        elif kind == "trait":
            for member_id in inner["items"]:
                include(member_id)
        elif kind in {"struct", "union"}:
            structure_kind = inner.get("kind", {})
            if kind == "union":
                fields = inner.get("fields", [])
            elif isinstance(structure_kind, dict):
                fields = structure_kind.get("plain", {}).get("fields", structure_kind.get("tuple", []))
            else:
                fields = []
            for field_id in fields:
                if field_id is not None:
                    include(field_id)
            impl_ids = selected_impls[str(item_id)]
            inner["impls"] = impl_ids
            for impl_id in impl_ids:
                impl_item = include(impl_id)
                impl = impl_item["inner"]["impl"]
                for member_id in impl["items"]:
                    include(member_id)
        elif kind == "enum":
            inner["impls"] = []
            for variant_id in inner["variants"]:
                variant = include(variant_id)
                variant_inner = variant["inner"]["variant"]
                variant_kind = variant_inner["kind"]
                fields = variant_kind.get("tuple", []) if isinstance(variant_kind, dict) else []
                if isinstance(variant_kind, dict):
                    fields = variant_kind.get("tuple", variant_kind.get("struct", {}).get("fields", []))
                for field_id in fields:
                    if field_id is not None:
                        include(field_id)

    referenced_ids: set[str] = set()

    def collect_references(value: Any) -> None:
        if isinstance(value, dict):
            for key, child in value.items():
                if key == "id" and isinstance(child, (int, str)):
                    referenced_ids.add(str(child))
                else:
                    collect_references(child)
        elif isinstance(value, list):
            for child in value:
                collect_references(child)

    for item in selected.values():
        collect_references(item)
    external_paths = {
        identifier: raw["paths"][identifier]
        for identifier in sorted(referenced_ids)
        if identifier not in selected and identifier in raw.get("paths", {})
    }

    projected = {
        "root": raw["root"],
        "index": selected,
        "paths": external_paths,
        "format_version": raw["format_version"],
        "includes_private": raw.get("includes_private", True),
        "target": {"triple": raw["target"]["triple"]},
    }
    output_path.write_text(json.dumps(projected, indent=2, sort_keys=True) + "\n", encoding="utf-8")


if __name__ == "__main__":
    if len(sys.argv) != 3:
        raise SystemExit("usage: project_fixture.py RAW_RUSTDOC_JSON OUTPUT_JSON")
    project(Path(sys.argv[1]), Path(sys.argv[2]))
