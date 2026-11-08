"""Local request and generic TTLV validation binding parity."""

from __future__ import annotations

import socket
import re
import threading
from pathlib import Path
from types import SimpleNamespace

import pytest

from _test_support import mtls_transport_config
from kmipkit import extensions, ttlv, validation
from kmipkit import client, errors

_TLS_FIXTURES = Path(__file__).parent / "fixtures" / "login-mtls"


def test_generic_structure_without_schema_does_not_guess_an_operation():
    with ttlv.ttlv_structure_create() as structure:
        with validation.validate_structure(structure) as report:
            assert report.is_valid
            assert report.issues == ()


def test_explicit_create_schema_reports_redacted_stable_metadata():
    with ttlv.ttlv_structure_create() as structure:
        with validation.validate_structure(
            structure, schema="RequestPayload/Create"
        ) as report:
            assert not report.is_valid
            assert len(report.issues) == 2
            issue = report.issues[0]
            assert issue.code == "KMIPKIT-0051-R019"
            assert issue.path == "Request Payload.Object Type"
            assert issue.severity == "ERROR"
            assert issue.source_identifier == "KMIP v2.1"
            assert issue.source_reference == "§6.1.8, Table 186"
            assert "SECRET" not in str(issue)


def test_explicit_schema_identifiers_cover_every_rust_schema_variant():
    root = Path(__file__).resolve().parents[3]
    schema_source = (
        root / "crates" / "kmipkit-protocol" / "src" / "validation" / "schema.rs"
    ).read_text(encoding="utf-8")
    request_source = (
        root
        / "crates"
        / "kmipkit-protocol"
        / "src"
        / "validation"
        / "schemas_generated.rs"
    ).read_text(encoding="utf-8")

    def variants(source: str, name: str) -> list[str]:
        match = re.search(
            rf"pub\s+enum\s+{name}\s*\{{(.*?)^\}}",
            source,
            re.MULTILINE | re.DOTALL,
        )
        assert match is not None, name
        return re.findall(r"^\s{4}([A-Z][A-Za-z0-9_]*)\s*,?\s*$", match.group(1), re.MULTILINE)

    schemas = [
        *(f"RequestPayload/{name}" for name in variants(request_source, "RequestPayloadSchema")),
        *(f"Nested/{name}" for name in variants(schema_source, "NestedStructureSchema")),
        "Unsupported",
    ]
    with ttlv.ttlv_structure_create() as structure:
        for schema in schemas:
            with validation.validate_structure(structure, schema=schema) as report:
                assert isinstance(report.is_valid, bool), schema


def test_typed_request_uses_the_shared_rust_validation_report():
    with extensions.client_batch_item_ping() as item:
        with item.validation_report() as report:
            assert report.is_valid
            assert report.issues == ()
            assert "ValidationReport(closed=False)" == repr(report)


def test_validation_binding_rejects_invalid_inputs_and_decodes_issue_spans():
    with ttlv.ttlv_structure_create() as structure:
        with pytest.raises(errors.InvalidInputError):
            validation.validate_structure(object())  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            validation.validate_structure(structure, schema=object())  # type: ignore[arg-type]
        with pytest.raises(errors.InvalidInputError):
            validation.validate_structure(structure, schema="\ud800")
        with pytest.raises(errors.ResourceLimitError):
            validation.validate_structure(structure, schema="x" * 129)

    with pytest.raises(errors.InvalidInputError):
        validation.validate_request(object())

    assert validation._span_text(SimpleNamespace(length=0)) == ""
    with pytest.raises(errors.InvalidInputError):
        validation._span_text(SimpleNamespace(data=validation.ffi.NULL, length=1))

    short_buffer = validation.ffi.new("uint8_t[]", b"x")
    with pytest.raises(errors.ResourceLimitError):
        validation._span_text(SimpleNamespace(data=short_buffer, length=4097))

    invalid_utf8 = validation.ffi.new("uint8_t[]", b"\xff")
    with pytest.raises(errors.InvalidInputError):
        validation._span_text(SimpleNamespace(data=invalid_utf8, length=1))


def test_invalid_typed_request_is_redacted_and_never_connects():
    canary = "VALIDATION_SECRET_CANARY"
    listener = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    listener.bind(("127.0.0.1", 0))
    listener.listen(1)
    listener.settimeout(1.0)
    port = listener.getsockname()[1]
    accepted: list[bool] = []

    def observe_connection() -> None:
        try:
            connection, _address = listener.accept()
        except TimeoutError:
            accepted.append(False)
        else:
            accepted.append(True)
            connection.close()
        finally:
            listener.close()

    observer = threading.Thread(target=observe_connection, daemon=True)
    observer.start()

    registry = extensions.create_client_extension_registry(
        (), extensions.default_extension_registry_limits()
    )
    configuration = extensions.create_client_configuration(registry)
    with (
        ttlv.codec_limits_defaults() as limits,
        ttlv.raw_tag_from_raw(0x0042_00FD) as raw_comment,
        ttlv.raw_tag_try_checked(raw_comment) as comment,
        ttlv.ttlv_structure_create() as empty_attributes,
    ):
        attributes = empty_attributes
        for _ in range(2):
            with ttlv.ttlv_value_text_string(canary.encode(), limits) as value:
                with ttlv.ttlv_item_create(comment, value, limits) as item:
                    attributes = ttlv.ttlv_structure_with_item(
                        attributes, item, limits
                    )

        with (
            attributes,
            extensions.client_batch_item_create(2, attributes) as request,
            validation.validate_request(request) as report,
            client.Batch.create() as batch,
            client.Client.create(
                configuration, mtls_transport_config(port, _TLS_FIXTURES)
            ) as kmip,
        ):
            assert not report.is_valid
            issue = report.issues[0]
            assert issue.code == "KMIPKIT-0051-R190"
            assert issue.path == "Request Payload.Attributes"
            assert issue.source_reference == "§4, Table 46"
            assert canary not in repr(issue)

            batch.add_item(request)
            with pytest.raises(errors.ClientExecutionError) as failure:
                kmip.execute(batch, limits)
            assert failure.value.client_category is errors.ClientErrorCategory.VALIDATION
            assert failure.value.delivery is errors.DeliveryState.NOT_SENT
            assert canary not in str(failure.value)

    observer.join(timeout=2)
    assert not observer.is_alive(), "the connection observer must finish"
    assert accepted == [False], "invalid local requests must make zero TCP connections"
