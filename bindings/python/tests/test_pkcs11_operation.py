"""Candidate-only CFFI smoke tests; these bindings are not shipped in the wheel."""

from __future__ import annotations

import json
import os
from pathlib import Path

import pytest

from kmipkit import client, extensions, ttlv
from kmipkit._client_ffi import ffi as client_ffi
from kmipkit._client_ffi import lib as client_lib
from kmipkit._ffi import ffi


ROOT = Path(__file__).resolve().parents[3]
FIXTURE = ROOT / "tests" / "fixtures" / "api" / "pkcs11-parity.json"
CANDIDATE_RUNTIME = os.environ.get("KMIPKIT_PKCS11_CANDIDATE_TEST_RUNTIME") == "1"

client_ffi.cdef(
    """
    int32_t kmipkit_client_batch_item_pkcs11(
        kmipkit_ttlv_structure_t *function,
        kmipkit_ttlv_structure_t *interface,
        const kmipkit_byte_span_v1_t *correlation_value,
        kmipkit_ttlv_structure_t *input_parameters,
        kmipkit_client_batch_item_t **out_item);
    int32_t kmipkit_client_batch_item_response_pkcs11_ttlv(
        kmipkit_client_batch_item_response_t *item,
        uint8_t *out_present,
        kmipkit_ttlv_structure_t **out_response_ttlv);
    """
)


def _client_structure(value: ttlv.Structure | None) -> object:
    if value is None:
        return client_ffi.NULL
    pointer = extensions._native_structure(value, required=True)
    address = int(ffi.cast("uintptr_t", pointer))
    return client_ffi.cast("kmipkit_ttlv_structure_t *", address)


def test_pkcs11_candidate_is_absent_from_default_python_facade() -> None:
    assert not hasattr(extensions, "client_batch_item_pkcs11")
    assert not hasattr(extensions, "_client_batch_item_pkcs11_candidate")
    assert not hasattr(client, "_Pkcs11Response")
    assert not hasattr(client, "_pkcs11_response")


def test_pkcs11_candidate_symbol_is_present_only_in_the_candidate_runtime() -> None:
    assert hasattr(client_lib, "kmipkit_client_batch_item_pkcs11") is CANDIDATE_RUNTIME


@pytest.mark.skipif(
    not CANDIDATE_RUNTIME,
    reason="owner-gated PKCS#11 symbols exist only in the isolated candidate runtime",
)
def test_pkcs11_cffi_candidate_constructor_accepts_optional_structures_and_bytes() -> None:
    function = ttlv.ttlv_structure_create()
    interface = ttlv.ttlv_structure_create()
    input_parameters = ttlv.ttlv_structure_create()
    correlation = b"pkcs11-correlation-private-sentinel"
    staged = client_ffi.new("uint8_t[]", correlation)
    correlation_span = client_ffi.new("kmipkit_byte_span_v1_t *")
    correlation_span.data = staged
    correlation_span.length = len(correlation)
    item = client_ffi.new("kmipkit_client_batch_item_t **")
    try:
        status = client_lib.kmipkit_client_batch_item_pkcs11(
            _client_structure(function),
            _client_structure(interface),
            correlation_span,
            _client_structure(input_parameters),
            item,
        )
        assert status == 0
        assert item[0] != client_ffi.NULL
        client_lib.kmipkit_client_batch_item_release(item[0])
        item[0] = client_ffi.NULL
    finally:
        client_ffi.buffer(staged, len(correlation))[:] = bytes(len(correlation))
        if item[0] != client_ffi.NULL:
            client_lib.kmipkit_client_batch_item_release(item[0])
        function.close()
        interface.close()
        input_parameters.close()


@pytest.mark.skipif(
    not CANDIDATE_RUNTIME,
    reason="owner-gated PKCS#11 symbols exist only in the isolated candidate runtime",
)
def test_pkcs11_cffi_response_accessor_clears_outputs_for_invalid_handle() -> None:
    present = client_ffi.new("uint8_t *", 0xFF)
    response = client_ffi.new(
        "kmipkit_ttlv_structure_t **",
        client_ffi.cast("kmipkit_ttlv_structure_t *", 1),
    )
    status = client_lib.kmipkit_client_batch_item_response_pkcs11_ttlv(
        client_ffi.NULL, present, response
    )
    assert status == 4
    assert present[0] == 0
    assert response[0] == client_ffi.NULL


def test_pkcs11_shared_fixture_declares_the_canonical_contract() -> None:
    fixture = json.loads(FIXTURE.read_text(encoding="utf-8"))
    assert fixture["capability"] == "KMIPKIT-ELEM-OP-C2S-PKCS-11"
    assert fixture["normativeSource"] == (
        "OASIS KMIP Specification v2.1 §6.1.37, Tables 273–275"
    )
    assert set(fixture["consumerTests"]) == {"rust", "c", "java", "python"}
    assert fixture["expectedOutput"]["automaticRetry"] is False
    assert fixture["expectedOutput"]["automaticFollowUp"] is False
    assert "profileSupport" not in fixture["expectedOutput"]
