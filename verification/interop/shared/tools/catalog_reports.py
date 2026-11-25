"""Bring installed-validator execution reports into a campaign."""

import hashlib
import json
from collections import Counter
from collections.abc import Callable
from pathlib import Path


def load_catalog_reports(
    reports: list[tuple[str, Path]], catalog_paths: tuple[Path, ...], languages: tuple[str, ...],
    versions: tuple[str, ...], read_validated: Callable,
) -> list[dict]:
    if not reports:
        return []
    catalogs = {}
    for catalog_path in catalog_paths:
        catalog_bytes = catalog_path.read_bytes()
        catalogs[hashlib.sha256(catalog_bytes).hexdigest()] = (
            catalog_path.name, {case["case_id"]: case for case in json.loads(catalog_bytes)["test_cases"]},
        )
    run_ids = set()
    normalized = []
    for scope, path in reports:
        payload = read_validated(path, "validation-results.schema.json")
        digest = payload["catalog_sha256"]
        if digest not in catalogs:
            raise ValueError(f"{path.name}: report uses a different execution catalog")
        catalog_name, catalog = catalogs[digest]
        language, version = payload["language"], payload["protocol_version"]
        if language not in languages or version not in versions:
            raise ValueError(f"{path.name}: report language or protocol is outside the campaign")
        if payload["run_id"] in run_ids:
            raise ValueError(f"{path.name}: duplicate catalog run ID")
        run_ids.add(payload["run_id"])
        cases = materialize_cases(payload["cases"], catalog, scope, path)
        normalized.append({
            "run_id": payload["run_id"], "scope": scope, "language": language,
            "protocol_version": version, "catalog": catalog_name, "catalog_sha256": digest, "server": payload["server"],
            "execution": payload["execution"],
            "summary": dict(Counter(case["status"] for case in cases)),
            "cases": cases,
        })
    return normalized


def materialize_cases(cases, catalog, scope, path):
    seen = set()
    rows = []
    for case in cases:
        case_id = case["case_id"]
        if case_id not in catalog or case_id in seen:
            raise ValueError(f"{path.name}: unknown or duplicate execution case ID")
        seen.add(case_id)
        metadata = catalog[case_id]
        row = {
            key: case[key]
            for key in ("case_id", "status", "library_correctness", "result_status_code", "result_reason_code")
            if key in case
        }
        row["operation"] = metadata["operation"]
        row["mode"] = metadata["execution"]["mode"]
        row["assertions"] = metadata["execution"]["assertions"]
        if scope == "live":
            row["server_compatibility"] = case["server_compatibility"]
        rows.append(row)
    return rows
