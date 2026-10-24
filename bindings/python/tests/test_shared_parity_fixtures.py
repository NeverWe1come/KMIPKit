"""Python 3.12 consumer checks for the shared KMIPKIT-0043 parity cases."""

from __future__ import annotations

import ast
import gc
import inspect
import json
import shutil
import subprocess
import sys
import zipfile
from dataclasses import fields
from pathlib import Path
from typing import Any, get_type_hints

import pytest

from kmipkit import client, errors, extensions, metadata, ttlv
from kmipkit import _handles
from kmipkit._handles import _wrap_owned_handle


ROOT = Path(__file__).resolve().parents[3]
PARITY_FIXTURE = ROOT / "tests" / "fixtures" / "api" / "shared-capability-parity.json"
NATIVE_RELEASES = {
    "kmipkit_client_release",
    "kmipkit_client_batch_release",
    "kmipkit_client_batch_response_release",
    "kmipkit_ttlv_structure_release",
    "kmipkit_ttlv_structure_view_release",
}


def _parity_cases() -> dict[str, dict[str, Any]]:
    corpus = json.loads(PARITY_FIXTURE.read_text(encoding="utf-8"))
    return {case["id"]: case for case in corpus["cases"]}


def _available_python_cases() -> list[dict[str, Any]]:
    return [
        case
        for case in _parity_cases().values()
        if case["consumerTests"].get("python", {}).get("status") == "available"
    ]


@pytest.mark.parametrize(
    "case", _available_python_cases(), ids=lambda case: case["id"]
)
def test_shared_fixture_python_link_names_an_existing_consumer_test(
    case: dict[str, Any],
) -> None:
    consumer = case["consumerTests"]["python"]
    test_path = ROOT / consumer["testFile"]

    assert test_path.is_file()
    tree = ast.parse(test_path.read_text(encoding="utf-8"), filename=str(test_path))
    assert any(
        isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == consumer["test"]
        for node in ast.walk(tree)
    )


def test_api_level_query_matches_shared_fixture_locally(monkeypatch: pytest.MonkeyPatch) -> None:
    case = _parity_cases()["api-levels.shared.v1"]
    expected = case["expectedOutput"]
    consumer = case["consumerTests"]["python"]

    assert consumer["status"] == "available"
    assert expected["networkCalls"] == 0
    assert expected["distinctFromKmipOperation"] == "Discover Versions"
    assert expected["operationMappingCompleteness"] == "not-asserted"
    assert metadata.capability_is_available(case["capability"])

    native_library = metadata.lib
    queried_symbols: list[str] = []

    class LocalMetadataApi:
        def __getattr__(self, name: str) -> Any:
            queried_symbols.append(name)
            return getattr(native_library, name)

    monkeypatch.setattr(metadata, "lib", LocalMetadataApi())
    assert list(metadata.get_api_levels()) == expected["apiLevels"]
    assert queried_symbols == [
        "kmipkit_api_levels_get_v1",
    ]


def test_generic_ttlv_shared_fixture_has_an_executable_python_link() -> None:
    case = _parity_cases()["ttlv.open-values-and-child-order.v1"]
    python_consumer = case["consumerTests"]["python"]
    assert python_consumer["status"] == "available"
    assert python_consumer["testFile"] == "bindings/python/tests/test_shared_parity_fixtures.py"
    assert python_consumer["test"] == (
        "test_generic_ttlv_shared_fixture_preserves_open_values_order_and_owner_lifecycle"
    )


def test_creation_create_matches_shared_fixture() -> None:
    case = _parity_cases()["creation.create.request-response.v1"]
    assert case["input"]["attributesPresent"] is True
    assert case["expectedOutput"]["resultMessage"] is None
    assert case["expectedOutput"]["uniqueIdentifier"] == {
        "wireType": "Integer",
        "integer": -8,
    }
    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    item = None
    try:
        item = extensions.client_batch_item_create(0xFFFF_FFFF, attributes, masks)
        assert item is not None
        assert {"operation", "result_message"}.issubset(
            {field.name for field in fields(client.BatchItemResult)}
        )
    finally:
        if item is not None:
            item.close()
        attributes.close()
        masks.close()


def test_creation_result_message_matches_failure_fixture() -> None:
    case = _parity_cases()["creation.result-message-on-failure.v1"]
    expected = case["expectedOutput"]
    assert expected["resultStatus"] == 1
    assert expected["resultReason"] == 5
    metadata = client.BatchItemMetadataV2(
        expected["operation"],
        expected["outcomeKind"],
        expected["resultStatus"],
        expected["resultReason"],
        expected["resultMessage"],
    )
    assert metadata.result_message == "création refusée ✓"
    assert "création refusée" not in repr(metadata)


def test_creation_key_pair_matches_shared_fixture() -> None:
    case = _parity_cases()["creation.key-pair.request-response.v1"]
    expected = case["expectedOutput"]
    assert expected["groupsRemainDistinct"] is True
    assert expected["privateAttributeTagsInOrder"][-2:] == ["0x540002", "0x540002"]
    structures = [ttlv.ttlv_structure_create() for _ in range(5)]
    common, private, public, common_masks, public_masks = structures
    item = None
    try:
        item = extensions.client_batch_item_create_key_pair(
            common, private, public, common_masks, None, public_masks
        )
        assert item is not None
    finally:
        if item is not None:
            item.close()
        for structure in structures:
            structure.close()


def test_creation_split_key_matches_shared_fixture() -> None:
    case = _parity_cases()["creation.split-key.request-response.v1"]
    request = case["input"]
    expected = case["expectedOutput"]
    assert request["splitKeyParts"] == -3
    assert expected["responseUniqueIdentifiersInWireOrder"][0] == {
        "wireType": "TextString",
        "text": "part-a",
    }
    assert expected["pendingMetadata"]["correlationValueHex"] == "00ff80"
    attributes = ttlv.ttlv_structure_create()
    masks = ttlv.ttlv_structure_create()
    item = None
    try:
        item = extensions.client_batch_item_create_split_key(
            request["objectType"],
            request["splitKeyParts"],
            request["splitKeyThreshold"],
            request["splitKeyMethod"],
            attributes,
            unique_identifier=extensions.UniqueIdentifier.integer(-7),
            prime_field_size=None,
            protection_storage_masks=masks,
        )
        assert item is not None
    finally:
        if item is not None:
            item.close()
        attributes.close()
        masks.close()


def _structure_from_shared_ttlv_fixture(
    structure_input: dict[str, Any], limits: ttlv.CodecLimits
) -> ttlv.Structure:
    structure = ttlv.ttlv_structure_create()
    for child in structure_input["children"]:
        raw_tag = ttlv.raw_tag_from_raw(int(child["tag"], 16))
        tag = ttlv.raw_tag_try_checked(raw_tag)
        raw_tag.close()
        if child["type"] == "Enumeration":
            value = ttlv.ttlv_value_enumeration(child["value"])
        elif child["type"] == "ByteString":
            value = ttlv.ttlv_value_byte_string(
                bytes.fromhex(child["valueHex"]), limits
            )
        else:
            raise AssertionError(f"unsupported shared TTLV fixture type: {child['type']}")
        item = ttlv.ttlv_item_create(tag, value, limits)
        structure = ttlv.ttlv_structure_with_item(structure, item, limits)
        tag.close()
    return structure


def test_generic_ttlv_shared_fixture_preserves_open_values_order_and_owner_lifecycle() -> (
    None
):
    case = _parity_cases()["ttlv.open-values-and-child-order.v1"]
    structure_input = case["input"]["structure"]
    expected = case["expectedOutput"]
    assert expected["unknownValuesPreserved"] is True
    limits = ttlv.codec_limits_defaults()
    structure = _structure_from_shared_ttlv_fixture(structure_input, limits)

    released: list[str] = []
    native_library = _handles.lib

    class ReleaseRecorder:
        def __getattr__(self, name: str) -> Any:
            function = getattr(native_library, name)
            if name not in NATIVE_RELEASES:
                return function

            def release(handle: object) -> int:
                released.append(name)
                return function(handle)

            return release

    _handles.lib = ReleaseRecorder()
    view = None
    try:
        view = ttlv.ttlv_structure_view(structure)
        del structure
        gc.collect()
        assert "kmipkit_ttlv_structure_release" not in released

        observed_tags: list[str] = []
        observed_values: list[tuple[str, object]] = []
        for index in range(ttlv.ttlv_structure_view_item_count(view)):
            item_view = ttlv.ttlv_structure_view_item_at(view, index)
            tag_view = ttlv.ttlv_item_view_tag(item_view)
            value_view = ttlv.ttlv_item_view_value(item_view)
            try:
                observed_tags.append(f"0x{ttlv.tag_value(tag_view):06x}")
                item_type = ttlv.ttlv_item_view_type(item_view)
                if item_type is ttlv.ItemType.Enumeration:
                    observed_values.append(
                        ("Enumeration", ttlv.ttlv_value_view_enumeration(value_view))
                    )
                elif item_type is ttlv.ItemType.ByteString:
                    observed_values.append(
                        (
                            "ByteString",
                            bytes(
                                ttlv.ttlv_value_view_byte_at(value_view, offset)
                                for offset in range(
                                    ttlv.ttlv_value_view_byte_length(value_view)
                                )
                            ).hex(),
                        )
                    )
                else:
                    raise AssertionError(
                        f"unexpected shared TTLV item type: {item_type}"
                    )
            finally:
                tag_view.close()
                value_view.close()
                item_view.close()

        assert observed_tags == expected["childTagsInOrder"]
        expected_values: list[tuple[str, object]] = []
        for child in structure_input["children"]:
            if child["type"] == "Enumeration":
                expected_values.append((child["type"], expected["enumerationValue"]))
            elif child["type"] == "ByteString":
                expected_values.append((child["type"], expected["byteStringHex"]))
            else:
                raise AssertionError(
                    f"unexpected shared TTLV item type: {child['type']}"
                )
        assert observed_values == expected_values

        view.close()
        view.close()
        assert view.closed
        assert released.count("kmipkit_ttlv_structure_view_release") == 1
        assert released.count("kmipkit_ttlv_structure_release") == 1
    finally:
        if view is not None:
            view.close()
        limits.close()
        _handles.lib = native_library


def test_codec_limits_shared_fixture_matches_defaults_ranges_and_errors() -> None:
    case = _parity_cases()["codec-limits.defaults-and-ranges.v1"]
    request = case["input"]
    expected = case["expectedOutput"]
    assert request["defaults"] is True

    defaults = ttlv.codec_limits_defaults()
    configured = None
    try:
        assert (
            ttlv.codec_limits_max_message_bytes(defaults)
            == expected["defaults"]["maxMessageBytes"]
        )
        assert (
            ttlv.codec_limits_max_structure_depth(defaults)
            == expected["defaults"]["maxStructureDepth"]
        )
        assert (
            ttlv.codec_limits_max_elements(defaults)
            == expected["defaults"]["maxElements"]
        )

        configured_input = request["configured"]
        configured = ttlv.codec_limits_create(
            configured_input["maxMessageBytes"],
            configured_input["maxStructureDepth"],
            configured_input["maxElements"],
        )
        configured_expected = expected["configured"]
        assert (
            ttlv.codec_limits_max_message_bytes(configured)
            == configured_expected["maxMessageBytes"]
        )
        assert (
            ttlv.codec_limits_max_structure_depth(configured)
            == configured_expected["maxStructureDepth"]
        )
        assert (
            ttlv.codec_limits_max_elements(configured)
            == configured_expected["maxElements"]
        )

        invalid = expected["invalidDepth"]
        assert invalid["value"] == request["invalidDepth"]
        assert invalid["accepted"] is False
        with pytest.raises(errors.ResourceLimitError) as caught:
            ttlv.codec_limits_create(
                configured_input["maxMessageBytes"],
                invalid["value"],
                configured_input["maxElements"],
            )
        assert caught.value.category == invalid["errorCategory"]
    finally:
        if configured is not None:
            configured.close()
        defaults.close()


@pytest.mark.parametrize(
    ("wrapper", "c_type", "release_name"),
    [
        (client.Client, "kmipkit_client_t", "kmipkit_client_release"),
        (client.Batch, "kmipkit_client_batch_t", "kmipkit_client_batch_release"),
        (
            client.BatchResult,
            "kmipkit_client_batch_response_t",
            "kmipkit_client_batch_response_release",
        ),
    ],
    ids=["client", "batch", "batch-result"],
)
@pytest.mark.parametrize("exceptional_exit", [False, True], ids=["normal", "exception"])
def test_client_batch_and_result_context_exit_closes_once(
    monkeypatch: pytest.MonkeyPatch,
    wrapper: type[Any],
    c_type: str,
    release_name: str,
    exceptional_exit: bool,
) -> None:
    storage = client.ffi.new("uint8_t[]", 1)
    pointer = client.ffi.cast(f"{c_type} *", storage)
    released: list[object] = []
    monkeypatch.setattr(
        client,
        "lib",
        type("ReleaseLibrary", (), {release_name: lambda _self, handle: released.append(handle)})(),
    )
    resource = (
        _wrap_owned_handle(wrapper, pointer, ())
        if wrapper is client.BatchResult
        else _wrap_owned_handle(wrapper, pointer)
    )

    if exceptional_exit:
        with pytest.raises(RuntimeError, match="fixture exception"):
            with resource as entered:
                assert entered is resource
                raise RuntimeError("fixture exception")
    else:
        with resource as entered:
            assert entered is resource
            assert not resource.closed

    resource.close()
    assert resource.closed
    assert released == [pointer]


def _public_functions_and_classes() -> list[object]:
    public: list[object] = []
    for module in (client, errors, extensions, metadata, ttlv):
        names = getattr(module, "__all__", None)
        if names is None:
            names = [
                name
                for name, value in vars(module).items()
                if not name.startswith("_")
                and getattr(value, "__module__", None) == module.__name__
                and (inspect.isfunction(value) or inspect.isclass(value))
            ]
        public.extend(
            getattr(module, name)
            for name in names
            if inspect.isfunction(getattr(module, name))
            or inspect.isclass(getattr(module, name))
        )
    return public


def _missing_annotations(function: object, label: str) -> list[str]:
    signature = inspect.signature(function)
    missing = [
        f"{label}.{parameter.name}"
        for parameter in signature.parameters.values()
        if parameter.name not in {"self", "cls"}
        and parameter.annotation is inspect.Signature.empty
    ]
    if signature.return_annotation is inspect.Signature.empty:
        missing.append(f"{label}.return")
    get_type_hints(function)
    return missing


def test_complete_public_python_api_has_resolvable_parameter_and_return_annotations() -> None:
    missing: list[str] = []
    for public in _public_functions_and_classes():
        name = getattr(public, "__qualname__", repr(public))
        if inspect.isfunction(public):
            missing.extend(_missing_annotations(public, name))
            continue

        if getattr(public, "__annotations__", None):
            get_type_hints(public)
        for owner in public.__mro__:
            if owner is object or owner.__module__ in {"builtins", "enum"}:
                continue
            for member_name, raw_member in vars(owner).items():
                if member_name.startswith("_") and member_name not in {
                    "__enter__",
                    "__exit__",
                    "__init__",
                    "__post_init__",
                }:
                    continue
                member = raw_member
                if isinstance(member, property):
                    for accessor_name, accessor in (
                        ("getter", member.fget),
                        ("setter", member.fset),
                        ("deleter", member.fdel),
                    ):
                        if accessor is not None:
                            missing.extend(
                                _missing_annotations(
                                    accessor,
                                    f"{name}.{member_name}.{accessor_name}",
                                )
                            )
                    continue
                if isinstance(member, (classmethod, staticmethod)):
                    member = member.__func__
                if inspect.isfunction(member):
                    missing.extend(
                        _missing_annotations(member, f"{name}.{member_name}")
                    )

    assert not missing, f"public Python API declarations without complete annotations: {missing}"


def test_built_wheel_artifact_contains_py_typed_and_adjacent_native_library(
    tmp_path: Path,
) -> None:
    wheelhouse = tmp_path / "wheelhouse"
    wheelhouse.mkdir()
    executable_name = "maturin.exe" if sys.platform == "win32" else "maturin"
    executable_directory = "Scripts" if sys.platform == "win32" else "bin"
    maturin_path = Path(sys.prefix) / executable_directory / executable_name
    if not maturin_path.is_file():
        path_executable = shutil.which("maturin")
        assert path_executable is not None, (
            "the Python package build requires the pinned maturin tool"
        )
        maturin_path = Path(path_executable)
    result = subprocess.run(
        [
            str(maturin_path),
            "build",
            "--offline",
            "--interpreter",
            sys.executable,
            "--out",
            str(wheelhouse),
        ],
        cwd=ROOT / "bindings" / "python",
        capture_output=True,
        check=False,
        encoding="utf-8",
        errors="replace",
        text=True,
    )
    assert result.returncode == 0, f"Python wheel build failed:\n{result.stdout}\n{result.stderr}"

    wheels = list(wheelhouse.glob("kmipkit-*.whl"))
    assert len(wheels) == 1
    with zipfile.ZipFile(wheels[0]) as wheel:
        members = set(wheel.namelist())

    assert "kmipkit/py.typed" in members
    native_libraries = {
        name
        for name in members
        if Path(name).parent.as_posix() == "kmipkit/_native_ffi"
        and Path(name).suffix.lower() in {".dll", ".dylib", ".so"}
    }
    assert len(native_libraries) == 1
