from __future__ import annotations

import hashlib
from importlib.metadata import distribution
import importlib.util
import json
import os
from pathlib import Path
from typing import Any
from zipfile import ZipFile


ROOT = Path(__file__).resolve().parents[5]
SHARED = ROOT / "verification" / "interop" / "shared"
DEFAULT_CATALOG = SHARED / "catalog" / "kmipkit-1.0-test-catalog.json"
CONNECTION_SCHEMA = SHARED / "schemas" / "server-connection.schema.json"
CATALOG_SCHEMA = SHARED / "schemas" / "test-catalog.schema.json"
RESULT_SCHEMA = SHARED / "schemas" / "validation-results.schema.json"
ARTIFACT_MANIFEST = Path(__file__).resolve().parents[2] / "build" / "artifact-manifest.json"


def _validate(value: Any, schema_path: Path, message: str) -> None:
    try:
        import jsonschema
    except ImportError as error:
        raise ValueError("schema validator is not installed") from error
    schema = json.loads(schema_path.read_text(encoding="utf-8"))
    try:
        jsonschema.validate(value, schema)
    except jsonschema.ValidationError as error:
        raise ValueError(message) from error


def load_json(path: Path, error_message: str) -> tuple[Any, bytes]:
    try:
        resolved = path.expanduser().resolve(strict=True)
        if not resolved.is_file():
            raise OSError
        raw = resolved.read_bytes()
        return json.loads(raw), raw
    except (OSError, UnicodeError, json.JSONDecodeError) as error:
        raise ValueError(error_message) from error


def load_connection(path: Path) -> tuple[dict[str, Any], bytes]:
    value, raw = load_json(path, "connection configuration is unavailable or invalid")
    _validate(value, CONNECTION_SCHEMA, "connection configuration violates the shared schema")
    return value, raw


def load_catalog(path: Path = DEFAULT_CATALOG) -> tuple[dict[str, Any], bytes]:
    value, raw = load_json(path, "shared test catalog is unavailable or invalid")
    _validate(value, CATALOG_SCHEMA, "shared test catalog violates the shared schema")
    return value, raw


def applicable_cases(catalog: dict[str, Any], protocol_version: str) -> list[dict[str, Any]]:
    return [
        case
        for case in catalog["test_cases"]
        if protocol_version in case["execution"]["protocol_versions"]
    ]


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def validate_local_inputs(connection: dict[str, Any]) -> list[str]:
    issues: list[str] = []
    tls = connection.get("tls")
    if not isinstance(tls, dict):
        return ["TLS identity configuration is required"]
    for key in ("ca_cert_path", "client_cert_path", "client_key_path"):
        value = tls.get(key)
        if not isinstance(value, str) or not Path(value).expanduser().is_file():
            issues.append("configured TLS input is unavailable")
    authentication = connection.get("authentication", {}).get("username_password")
    if authentication:
        import os
        if not os.environ.get(authentication["username_env"]) or not os.environ.get(authentication["password_env"]):
            issues.append("configured authentication environment is unavailable")
    return issues


def validate_artifact(manifest_path: Path = ARTIFACT_MANIFEST) -> dict[str, str]:
    value, _ = load_json(manifest_path, "installed KMIPKit artifact manifest is unavailable")
    if not isinstance(value, dict):
        raise ValueError("installed KMIPKit artifact manifest is invalid")
    commit = value.get("library_commit")
    artifact_hash = value.get("artifact_sha256")
    artifact = value.get("artifact_file")
    if (
        not isinstance(commit, str)
        or len(commit) != 40
        or any(character not in "0123456789abcdef" for character in commit)
        or not isinstance(artifact_hash, str)
        or len(artifact_hash) != 64
        or not isinstance(artifact, str)
    ):
        raise ValueError("installed KMIPKit artifact manifest is invalid")
    artifact_path = Path(artifact).resolve(strict=True)
    if not artifact_path.is_file() or sha256(artifact_path.read_bytes()) != artifact_hash:
        raise ValueError("installed KMIPKit artifact digest does not match its manifest")
    if value.get("release_ref") != "release/1.0.0-review":
        raise ValueError("installed KMIPKit artifact is not pinned to release/1.0.0-review")
    pin_path = ARTIFACT_MANIFEST.parents[1] / "library-pin.json"
    pin, _ = load_json(pin_path, "KMIPKit release pin is unavailable")
    if value.get("library_commit") != pin.get("commit") or value.get("package_version") != "1.0.0":
        raise ValueError("installed KMIPKit artifact differs from the exact release pin")
    if os.environ.get("KMIPKIT_NATIVE_LIBRARY"):
        raise ValueError("Campaigns require the native library bundled in the wheel")
    package = distribution("kmipkit")
    imported = importlib.util.find_spec("kmipkit")
    expected_root = Path(package.locate_file("kmipkit")).resolve(strict=True)
    if imported is None or imported.origin is None or Path(imported.origin).resolve().parent != expected_root:
        raise ValueError("The import path differs from the installed distribution")
    with ZipFile(artifact_path) as wheel:
        for member in wheel.infolist():
            if member.is_dir() or not member.filename.startswith("kmipkit/"):
                continue
            path = Path(package.locate_file(member.filename)).resolve(strict=True)
            if not path.is_relative_to(expected_root) or path.read_bytes() != wheel.read(member):
                raise ValueError("An installed file differs from the verified wheel")
    return {"library_commit": commit, "artifact_sha256": artifact_hash, "artifact_file": str(artifact_path)}
