from __future__ import annotations

import json
from pathlib import Path
from types import SimpleNamespace
from typing import Any

import pytest

from kmipkit import client, errors, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._ffi import ffi, lib as ttlv_lib


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"


def test_pkcs11_candidate_is_not_public_in_the_python_package_surface() -> None:
    assert not hasattr(extensions, "client_batch_item_pkcs11")
    assert not hasattr(client, "Pkcs11Response")
    assert not hasattr(client.BatchItemResult, "pkcs11_response")


def test_pkcs11_builder_preserves_generic_inputs_and_wipes_staging_bytes(
    monkeypatch: Any,
) -> None:
    function = object()
    interface = object()
    input_parameters = object()
    calls: list[tuple[str, tuple[object, ...]]] = []
    required_flags: list[bool] = []
    staged_owners: list[object] = []
    structure_addresses = {
        id(function): 0x101,
        id(interface): 0x202,
        id(input_parameters): 0x303,
    }

    def native_structure(value: object, *, required: bool = False) -> object:
        required_flags.append(required)
        return ffi.cast("kmipkit_ttlv_structure_t *", structure_addresses[id(value)])

    def make_handle(
        _kind: str, symbol: str, arguments: tuple[object, ...]
    ) -> int:
        span = arguments[2]
        staged = bytes(client_ffi.buffer(span[0].data, span[0].length))
        calls.append((symbol, arguments))
        assert staged == b"pkcs11-correlation-private-sentinel"
        return 41

    original_optional_span = extensions._native_optional_span

    def tracked_optional_span(
        value: object, buffers: list[Any]
    ) -> tuple[object, object]:
        span, owner = original_optional_span(value, buffers)  # type: ignore[arg-type]
        if owner is not None:
            staged_owners.append(owner)
        return span, owner

    monkeypatch.setattr(extensions, "_native_structure", native_structure)
    monkeypatch.setattr(extensions, "_native_optional_span", tracked_optional_span)
    monkeypatch.setattr(extensions, "_make_handle", make_handle)
    monkeypatch.setattr(extensions, "_wrap_owned_handle", lambda _kind, value: value)

    assert extensions._client_batch_item_pkcs11_candidate(
        function,
        interface,
        b"pkcs11-correlation-private-sentinel",
        input_parameters,
    ) == 41

    symbol, arguments = calls[0]
    assert symbol == "kmipkit_client_batch_item_pkcs11"
    assert int(client_ffi.cast("uintptr_t", arguments[0])) == 0x101
    assert int(client_ffi.cast("uintptr_t", arguments[1])) == 0x202
    assert int(client_ffi.cast("uintptr_t", arguments[3])) == 0x303
    assert required_flags == [True, False, False]
    expected_length = len(b"pkcs11-correlation-private-sentinel")
    assert bytes(ffi.buffer(staged_owners[0], expected_length)) == bytes(expected_length)


def test_pkcs11_builder_executes_through_the_client_cffi_declarations() -> None:
    function = ttlv.ttlv_structure_create()
    item = None
    try:
        item = extensions._client_batch_item_pkcs11_candidate(function)
        assert isinstance(item, extensions.ClientBatchItem)
        assert not item.closed
    finally:
        if item is not None:
            item.close()
        function.close()


def test_pkcs11_response_owns_generic_ttlv_and_redacts_repr(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    sentinel = b"pkcs11-response-private-sentinel"
    response_ttlv = _response_ttlv(sentinel)
    native_handle, owner = response_ttlv._take()
    assert owner is None

    def response(
        _item: object, present: object, output: object
    ) -> int:
        present[0] = 1  # type: ignore[index]
        output[0] = client_ffi.cast(
            "kmipkit_ttlv_structure_t *", int(ffi.cast("uintptr_t", native_handle))
        )  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_pkcs11_ttlv=response,
        ),
    )

    result = client._pkcs11_response(SimpleNamespace(_pointer=lambda: object()))

    assert isinstance(result, client._Pkcs11Response)
    with ttlv.ttlv_structure_view(result.response_ttlv) as view:
        assert ttlv.ttlv_structure_view_item_count(view) == 1
    assert sentinel.decode() not in repr(result)
    result.close()
    assert result.response_ttlv.closed


def test_pkcs11_response_releases_unexpected_owned_handle(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    handle = client_ffi.cast("kmipkit_ttlv_structure_t *", 123)
    released: list[object] = []

    def response(_item: object, present: object, output: object) -> int:
        present[0] = 0  # type: ignore[index]
        output[0] = handle  # type: ignore[index]
        return 0

    monkeypatch.setattr(
        client,
        "lib",
        SimpleNamespace(
            kmipkit_client_batch_item_response_pkcs11_ttlv=response,
            kmipkit_ttlv_structure_release=lambda value: released.append(value),
        ),
    )

    with pytest.raises(errors.InvalidInputError):
        client._pkcs11_response(SimpleNamespace(_pointer=lambda: object()))

    assert released == [handle]


def test_pkcs11_response_field_preserves_legacy_positional_batch_item_results() -> None:
    legacy_suffix = [object() for _ in range(7)]
    arguments: list[object] = [0, 0, None, b"legacy-batch-item", None]
    arguments.extend([None] * 26)
    arguments.extend(legacy_suffix)

    result = client.BatchItemResult(*arguments)

    assert len(arguments) == 38
    assert result.login_response is legacy_suffix[0]
    assert result.logout_response is legacy_suffix[1]
    assert result.obtain_lease_response is legacy_suffix[2]
    assert result.encrypt_response is legacy_suffix[3]
    assert result.decrypt_response is legacy_suffix[4]
    assert result.ping_response is legacy_suffix[5]
    assert result.query_response is legacy_suffix[6]


def test_pkcs11_shared_fixture_declares_the_canonical_four_language_contract() -> None:
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))

    assert fixture["capability"] == "KMIPKIT-ELEM-OP-C2S-PKCS-11"
    assert fixture["normativeSource"] == (
        "OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275"
    )
    assert set(fixture["consumerTests"]) == {"rust", "c", "java", "python"}
    assert fixture["expectedOutput"]["automaticRetry"] is False
    assert fixture["expectedOutput"]["automaticFollowUp"] is False
    assert "profileSupport" not in fixture["expectedOutput"]


def _response_ttlv(value: bytes) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    raw_tag = ttlv.raw_tag_from_raw(0x0054_1234)
    tag = ttlv.raw_tag_try_checked(raw_tag)
    ttlv_value = ttlv.ttlv_value_text_string(value, limits)
    item = ttlv.ttlv_item_create(tag, ttlv_value, limits)
    structure = ttlv.ttlv_structure_with_item(ttlv.ttlv_structure_create(), item, limits)
    limits.close()
    tag.close()
    raw_tag.close()
    ttlv_value.close()
    item.close()
    return structure
