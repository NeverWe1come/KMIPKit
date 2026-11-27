"""Stage 2 64/64 fluent operation parity tests for Python bindings."""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path
import struct

from kmipkit import errors, extensions, ttlv
from kmipkit.fluent import (
    AdjustAttributeBuilder,
    CancelBuilder,
    DelegatedLoginBuilder,
    GetConstraintsBuilder,
    InteropBuilder,
    LogBuilder,
    LoginBuilder,
    LogoutBuilder,
    Pkcs11Builder,
    PollBuilder,
    ProcessBuilder,
    QueryAsyncRequestsBuilder,
    ReProvisionBuilder,
    SetConstraintsBuilder,
    SetDefaultsBuilder,
    SetEndpointRoleBuilder,
)

REPO_ROOT = Path(__file__).resolve().parents[3]


def _encode_item(tag: int, item_type: int, raw_value: bytes) -> bytes:
    header = struct.pack(">I", tag)[1:4] + struct.pack(">BI", item_type, len(raw_value))
    pad = (8 - (len(raw_value) % 8)) % 8
    return header + raw_value + (b"\x00" * pad)


def _structure_from_encoded_children(*child_wires: bytes) -> ttlv.Structure:
    limits = ttlv.codec_limits_defaults()
    try:
        s = ttlv.ttlv_structure_create()
        for child_wire in child_wires:
            child = ttlv.ttlv_decode(child_wire)
            s = ttlv.ttlv_structure_with_item(s, child, limits)
        return s
    finally:
        limits.close()


def _structure_from_single_enum(child_tag: int, enum_val: int) -> ttlv.Structure:
    return _structure_from_encoded_children(
        _encode_item(child_tag, 0x05, struct.pack(">I", enum_val))
    )


def test_cross_language_spec_parity_and_c11_header() -> None:
    gen = REPO_ROOT / "bindings" / "shared" / "generate_fluent_c11.py"
    res = subprocess.run(
        [sys.executable, str(gen), "--check", "--verify-parity"],
        capture_output=True,
        text=True,
        check=False,
    )
    assert res.returncode == 0, res.stdout + res.stderr
    assert "64/64" in res.stdout


def test_python_stage2_16_new_builders_construct_valid_batch_items() -> None:
    uid = extensions.UniqueIdentifier.text_string("obj-1")

    with AdjustAttributeBuilder() as b:
        b.unique_identifier(uid)
        b.attribute_reference(extensions.AttributeReference(tag_value=0x420008))
        b.adjustment_type(1)
        b.adjustment_value(ttlv.ttlv_value_integer(10))
        with b.build() as item:
            assert item.operation == 0x30

    with GetConstraintsBuilder() as b, b.build() as item:
        assert item.operation == 0x38

    with CancelBuilder().correlation_value(b"\x01\x02") as b, b.build() as item:
        assert item.operation == 0x19

    with PollBuilder().correlation_value(b"\x01\x02") as b, b.build() as item:
        assert item.operation == 0x1A

    with ProcessBuilder().correlation_value(b"\x01\x02") as b, b.build() as item:
        assert item.operation == 0x3A

    with QueryAsyncRequestsBuilder() as b:
        b.correlation_value(b"\x01\x02").operation(0x01)
        with b.build() as item:
            assert item.operation == 0x39

    with LoginBuilder() as b:
        b.lease_time_interval(120).request_count(4)
        with b.build() as item:
            assert item.operation == 0x2D

    with _structure_from_encoded_children(
        _encode_item(0x42014A, 0x05, struct.pack(">I", 1)),
        _encode_item(0x42014B, 0x08, b"ticket"),
    ) as ticket:
        with LogoutBuilder(ticket=ticket) as b, b.build() as item:
            assert item.operation == 0x2E

    with _structure_from_single_enum(0x42014E, 1) as rights:
        with DelegatedLoginBuilder(rights=rights) as b:
            b.lease_time_date_time(1_700_000_000)
            with b.build() as item:
                assert item.operation == 0x2F

    with SetEndpointRoleBuilder().endpoint_role(1).vendor_identification("KMIPKit") as b:
        with b.build() as item:
            assert item.operation == 0x32

    with _structure_from_single_enum(0x42015A, 1) as fn_struct:
        with Pkcs11Builder(function=fn_struct) as b, b.build() as item:
            assert item.operation == 0x33

    with InteropBuilder().interop_function(1).interop_identifier("op-id") as b:
        with b.build() as item:
            assert item.operation == 0x34

    with ReProvisionBuilder() as b, b.build() as item:
        assert item.operation == 0x35

    with SetDefaultsBuilder() as b, b.build() as item:
        assert item.operation == 0x36

    with ttlv.ttlv_structure_create() as constraints:
        with SetConstraintsBuilder(constraints=constraints) as b, b.build() as item:
            assert item.operation == 0x37

    with LogBuilder().log_message("audit-msg") as b, b.build() as item:
        assert item.operation == 0x2C
