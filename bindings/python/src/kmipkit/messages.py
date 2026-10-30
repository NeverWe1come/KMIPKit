"""Typed, lossless KMIP Request and Response Message inspection."""

from __future__ import annotations

from typing import Self

from . import errors, ttlv
from ._client_ffi import ffi, lib
from ._ffi import ffi as _base_ffi
from ._handles import NativeHandle, _NATIVE_CALL_LOCK, _wrap_owned_handle
from .client import ProtocolVersion

_MAX_MESSAGE_BYTES = 16 * 1024 * 1024
_MAX_U64 = (1 << 64) - 1

_REQUEST_HEADER = 0x0042_0077
_RESPONSE_HEADER = 0x0042_007A
_PROTOCOL_VERSION = 0x0042_0069
_PROTOCOL_VERSION_MAJOR = 0x0042_006A
_PROTOCOL_VERSION_MINOR = 0x0042_006B
_ASYNCHRONOUS_INDICATOR = 0x0042_0007
_ATTESTATION_CAPABLE_INDICATOR = 0x0042_00D3
_ATTESTATION_TYPE = 0x0042_00C7
_AUTHENTICATION = 0x0042_000C
_BATCH_COUNT = 0x0042_000D
_BATCH_ERROR_CONTINUATION_OPTION = 0x0042_000E
_BATCH_ORDER_OPTION = 0x0042_0010
_CLIENT_CORRELATION_VALUE = 0x0042_0105
_MAXIMUM_RESPONSE_SIZE = 0x0042_0050
_NONCE = 0x0042_00C8
_SERVER_CORRELATION_VALUE = 0x0042_0106
_SERVER_HASHED_PASSWORD = 0x0042_0155
_TIME_STAMP = 0x0042_0092
_OPERATION = 0x0042_005C
_EPHEMERAL = 0x0042_0154
_UNIQUE_BATCH_ITEM_ID = 0x0042_0093
_REQUEST_PAYLOAD = 0x0042_0079
_RESPONSE_PAYLOAD = 0x0042_007C
_RESULT_STATUS = 0x0042_007F
_RESULT_REASON = 0x0042_007E
_RESULT_MESSAGE = 0x0042_007D
_ASYNCHRONOUS_CORRELATION_VALUE = 0x0042_0006
_MESSAGE_EXTENSION = 0x0042_0051
_VENDOR_IDENTIFICATION = 0x0042_009D
_CRITICALITY_INDICATOR = 0x0042_0026
_VENDOR_EXTENSION = 0x0042_009C


class MessageValidationError(errors.KmipKitError):
    """A redacted structural message error with stable numeric metadata."""

    category = "message_validation"

    def __init__(
        self,
        kind: int,
        structure_index: int | None,
        field_index: int | None,
    ) -> None:
        self.kind = int(kind)
        self.structure_index = structure_index
        self.field_index = field_index
        super().__init__()
        self.args = (
            f"KMIPKit message validation failed (kind {self.kind}; "
            f"structure index {structure_index}; field index {field_index})",
        )


class _MessageHandle(NativeHandle):
    """An opaque message-model handle released by the private CFFI bridge."""

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            if self.closed:
                return
            handle, owner = self._take()
            getattr(lib, self._release_name)(handle)
            del owner


def _call(function: object, *arguments: object) -> int:
    with _NATIVE_CALL_LOCK:
        try:
            return int(function(*arguments))  # type: ignore[operator]
        except (OverflowError, TypeError, ValueError):
            raise errors.InvalidInputError() from None


def _status(status: int) -> None:
    errors.raise_for_status(status)


def _scalar(c_type: str, function: object, *arguments: object) -> int:
    output = ffi.new(f"{c_type} *")
    _status(_call(function, *arguments, output))
    return int(output[0])


def _optional_scalar_pair(
    function: object, handle: NativeHandle, value_type: str
) -> int | None:
    present = ffi.new("uint8_t *")
    value = ffi.new(f"{value_type} *")
    _status(_call(function, handle._pointer(), present, value))
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    return int(value[0]) if present[0] == 1 else None


def _out_handle(c_type: str, function: object, *arguments: object) -> object:
    output = ffi.new(f"{c_type} **")
    _status(_call(function, *arguments, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    return output[0]


def _wrap_handle(wrapper: type[_MessageHandle], handle: object, owner: object | None = None):
    return _wrap_owned_handle(wrapper, handle, owner=owner)


def _as_client_structure(structure: ttlv.Structure) -> object:
    address = int(_base_ffi.cast("uintptr_t", structure._pointer()))
    return ffi.cast("kmipkit_ttlv_structure_t *", address)


def _consume_and_validate_message(
    structure: ttlv.Structure,
    *,
    handle_type: str,
    wrapper: type[_MessageHandle],
    constructor: object,
    error_release: object,
    message_release: object,
) -> _MessageHandle:
    if not isinstance(structure, ttlv.Structure):
        errors.raise_for_status(4)
    output = ffi.new(f"{handle_type} **")
    validation_error = ffi.new("kmipkit_message_validation_error_t **")
    structure._acquire_ownership_lock()
    try:
        source_pointer = _as_client_structure(structure)
        original, owner = structure._take()
        try:
            status = _call(constructor, source_pointer, output, validation_error)
        except Exception:
            structure._restore(original, owner)
            raise
        if validation_error[0] != ffi.NULL:
            if status == 0 or output[0] != ffi.NULL:
                error_release(validation_error[0])
                if output[0] != ffi.NULL:
                    message_release(output[0])
                errors.raise_for_status(4)
            error_handle = validation_error[0]
            try:
                kind = _scalar("uint32_t", lib.kmipkit_message_validation_error_kind, error_handle)
                structure_index = _optional_index(
                    lib.kmipkit_message_validation_error_structure_index,
                    error_handle,
                )
                field_index = _optional_index(
                    lib.kmipkit_message_validation_error_field_index,
                    error_handle,
                )
            finally:
                error_release(error_handle)
            raise MessageValidationError(kind, structure_index, field_index)
        if status != 0:
            if output[0] != ffi.NULL:
                message_release(output[0])
            errors.raise_for_status(status)
        if output[0] == ffi.NULL:
            errors.raise_for_status(4)
        return _wrap_handle(wrapper, output[0])
    finally:
        structure._release_ownership_lock()


def _optional_index(function: object, handle: object) -> int | None:
    present = ffi.new("uint8_t *")
    value = ffi.new("uint64_t *")
    _status(_call(function, handle, present, value))
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    if present[0] == 0:
        return None
    return int(value[0])


def _copy_ttlv(function: object, handle: NativeHandle) -> ttlv.Structure:
    output = ffi.new("kmipkit_ttlv_structure_t **")
    _status(_call(function, handle._pointer(), output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    address = int(ffi.cast("uintptr_t", output[0]))
    try:
        return ttlv._structure_from_native_address(address)
    except Exception:
        lib.kmipkit_ttlv_structure_release(output[0])
        raise


def _materialize_value(value_view: ttlv.TtlvValueView) -> object:
    value_type = ttlv.ttlv_value_view_type(value_view)
    if value_type == ttlv.ItemType.Structure:
        return ttlv.ttlv_value_view_structure(value_view)
    if value_type == ttlv.ItemType.Integer:
        return ttlv.ttlv_value_view_integer(value_view)
    if value_type == ttlv.ItemType.LongInteger:
        return ttlv.ttlv_value_view_long_integer(value_view)
    if value_type == ttlv.ItemType.Enumeration:
        return ttlv.ttlv_value_view_enumeration(value_view)
    if value_type == ttlv.ItemType.Boolean:
        return ttlv.ttlv_value_view_boolean(value_view)
    if value_type == ttlv.ItemType.DateTime:
        return ttlv.ttlv_value_view_date_time(value_view)
    if value_type == ttlv.ItemType.Interval:
        return ttlv.ttlv_value_view_interval(value_view)
    if value_type == ttlv.ItemType.DateTimeExtended:
        return ttlv.ttlv_value_view_date_time_extended(value_view)
    if value_type in (ttlv.ItemType.BigInteger, ttlv.ItemType.TextString, ttlv.ItemType.ByteString):
        length = ttlv.ttlv_value_view_byte_length(value_view)
        if length > _MAX_MESSAGE_BYTES:
            errors.raise_for_status(6)
        raw = bytes(
            ttlv.ttlv_value_view_byte_at(value_view, index)
            for index in range(length)
        )
        if value_type == ttlv.ItemType.TextString:
            try:
                return raw.decode("utf-8")
            except UnicodeDecodeError:
                errors.raise_for_status(4)
        return raw
    errors.raise_for_status(4)


def _field_values(
    copy_function: object,
    handle: NativeHandle,
    raw_tag: int,
) -> tuple[object, ...]:
    structure = _copy_ttlv(copy_function, handle)
    values: list[object] = []
    try:
        with ttlv.ttlv_structure_view(structure) as structure_view:
            count = ttlv.ttlv_structure_view_item_count(structure_view)
            for index in range(count):
                with ttlv.ttlv_structure_view_item_at(structure_view, index) as item_view:
                    with ttlv.ttlv_item_view_tag(item_view) as tag:
                        matches = ttlv.tag_value(tag) == raw_tag
                    if not matches:
                        continue
                    value_view = ttlv.ttlv_item_view_value(item_view)
                    try:
                        value = _materialize_value(value_view)
                    except Exception:
                        value_view.close()
                        raise
                    if not isinstance(value, ttlv.TtlvStructureView):
                        value_view.close()
                    values.append(value)
    finally:
        structure.close()
    return tuple(values)


def _field_value(copy_function: object, handle: NativeHandle, raw_tag: int) -> object | None:
    values = _field_values(copy_function, handle, raw_tag)
    if not values:
        return None
    if len(values) != 1:
        for value in values:
            if isinstance(value, NativeHandle):
                value.close()
        errors.raise_for_status(4)
    return values[0]


def _copy_message_ttlv(handle: NativeHandle, function: object) -> ttlv.Structure:
    return _copy_ttlv(function, handle)


class RequestMessage(_MessageHandle):
    """A validated, ordered KMIP Request Message with deterministic ownership."""

    _release_name = "kmipkit_request_message_release"

    @classmethod
    def from_ttlv(cls, structure: ttlv.Structure) -> Self:
        """Consume and validate a generic TTLV Structure as a Request Message."""
        handle = _consume_and_validate_message(
            structure,
            handle_type="kmipkit_request_message_t",
            wrapper=cls,
            constructor=lib.kmipkit_request_message_from_ttlv,
            error_release=lib.kmipkit_message_validation_error_release,
            message_release=lib.kmipkit_request_message_release,
        )
        return handle  # type: ignore[return-value]

    @property
    def batch_item_count(self) -> int:
        return _scalar(
            "uint64_t", lib.kmipkit_request_message_batch_item_count, self._pointer()
        )

    def header(self) -> RequestHeaderView:
        handle = _out_handle(
            "kmipkit_request_header_view_t",
            lib.kmipkit_request_message_header,
            self._pointer(),
        )
        return _wrap_handle(RequestHeaderView, handle, owner=self)

    def batch_item_at(self, index: int) -> RequestBatchItemView:
        index = _checked_index(index)
        handle = _out_handle(
            "kmipkit_request_batch_item_view_t",
            lib.kmipkit_request_message_batch_item_at,
            self._pointer(),
            index,
        )
        return _wrap_handle(RequestBatchItemView, handle, owner=self)

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete original ordered TTLV tree."""
        return _copy_message_ttlv(self, lib.kmipkit_request_message_ttlv)


class ResponseMessage(_MessageHandle):
    """A validated, ordered KMIP Response Message with deterministic ownership."""

    _release_name = "kmipkit_response_message_release"

    @classmethod
    def from_ttlv(cls, structure: ttlv.Structure) -> Self:
        """Consume and validate a generic TTLV Structure as a Response Message."""
        handle = _consume_and_validate_message(
            structure,
            handle_type="kmipkit_response_message_t",
            wrapper=cls,
            constructor=lib.kmipkit_response_message_from_ttlv,
            error_release=lib.kmipkit_message_validation_error_release,
            message_release=lib.kmipkit_response_message_release,
        )
        return handle  # type: ignore[return-value]

    @property
    def batch_item_count(self) -> int:
        return _scalar(
            "uint64_t", lib.kmipkit_response_message_batch_item_count, self._pointer()
        )

    def header(self) -> ResponseHeaderView:
        handle = _out_handle(
            "kmipkit_response_header_view_t",
            lib.kmipkit_response_message_header,
            self._pointer(),
        )
        return _wrap_handle(ResponseHeaderView, handle, owner=self)

    def batch_item_at(self, index: int) -> ResponseBatchItemView:
        index = _checked_index(index)
        handle = _out_handle(
            "kmipkit_response_batch_item_view_t",
            lib.kmipkit_response_message_batch_item_at,
            self._pointer(),
            index,
        )
        return _wrap_handle(ResponseBatchItemView, handle, owner=self)

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete original ordered TTLV tree."""
        return _copy_message_ttlv(self, lib.kmipkit_response_message_ttlv)


def _checked_index(index: int) -> int:
    if isinstance(index, bool) or not isinstance(index, int) or not 0 <= index <= _MAX_U64:
        errors.raise_for_status(4)
    return index


class RequestHeaderView(_MessageHandle):
    """Read-only typed access to a validated Request Header."""

    _release_name = "kmipkit_request_header_view_release"

    @property
    def protocol_version(self) -> ProtocolVersion:
        version = ffi.new("kmipkit_protocol_version_v1_t *")
        _status(_call(lib.kmipkit_request_header_view_protocol_version, self._pointer(), version))
        return ProtocolVersion(int(version.major), int(version.minor))

    @property
    def batch_count(self) -> int:
        return _scalar("int32_t", lib.kmipkit_request_header_view_batch_count, self._pointer())

    def to_ttlv(self) -> ttlv.Structure:
        return _copy_ttlv(lib.kmipkit_request_header_view_ttlv, self)

    @property
    def maximum_response_size(self) -> int | None:
        return _field_value(lib.kmipkit_request_header_view_ttlv, self, _MAXIMUM_RESPONSE_SIZE)

    @property
    def client_correlation_value(self) -> str | None:
        return _field_value(lib.kmipkit_request_header_view_ttlv, self, _CLIENT_CORRELATION_VALUE)

    @property
    def asynchronous_indicator(self) -> int | None:
        return _field_value(lib.kmipkit_request_header_view_ttlv, self, _ASYNCHRONOUS_INDICATOR)

    @property
    def effective_asynchronous_indicator(self) -> int:
        value = self.asynchronous_indicator
        return 3 if value is None else int(value)

    @property
    def attestation_capable_indicator(self) -> bool | None:
        return _field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _ATTESTATION_CAPABLE_INDICATOR,
        )

    @property
    def effective_attestation_capable_indicator(self) -> bool:
        value = self.attestation_capable_indicator
        return False if value is None else bool(value)

    @property
    def attestation_types(self) -> tuple[int, ...]:
        return tuple(
            int(value)
            for value in _field_values(
                lib.kmipkit_request_header_view_ttlv, self, _ATTESTATION_TYPE
            )
        )

    @property
    def authentication(self) -> ttlv.TtlvStructureView | None:
        value = _field_value(lib.kmipkit_request_header_view_ttlv, self, _AUTHENTICATION)
        return value if isinstance(value, ttlv.TtlvStructureView) else None

    @property
    def batch_error_continuation_option(self) -> int | None:
        return _field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _BATCH_ERROR_CONTINUATION_OPTION,
        )

    @property
    def effective_batch_error_continuation_option(self) -> int:
        value = self.batch_error_continuation_option
        return 2 if value is None else int(value)

    @property
    def batch_order_option(self) -> bool | None:
        return _field_value(lib.kmipkit_request_header_view_ttlv, self, _BATCH_ORDER_OPTION)

    @property
    def effective_batch_order_option(self) -> bool:
        value = self.batch_order_option
        return True if value is None else bool(value)

    @property
    def time_stamp(self) -> int | None:
        return _field_value(lib.kmipkit_request_header_view_ttlv, self, _TIME_STAMP)


class ResponseHeaderView(_MessageHandle):
    """Read-only typed access to a validated Response Header."""

    _release_name = "kmipkit_response_header_view_release"

    @property
    def protocol_version(self) -> ProtocolVersion:
        version = ffi.new("kmipkit_protocol_version_v1_t *")
        _status(_call(lib.kmipkit_response_header_view_protocol_version, self._pointer(), version))
        return ProtocolVersion(int(version.major), int(version.minor))

    @property
    def time_stamp(self) -> int:
        return _scalar(
            "int64_t", lib.kmipkit_response_header_view_time_stamp, self._pointer()
        )

    @property
    def batch_count(self) -> int:
        return _scalar("int32_t", lib.kmipkit_response_header_view_batch_count, self._pointer())

    def to_ttlv(self) -> ttlv.Structure:
        return _copy_ttlv(lib.kmipkit_response_header_view_ttlv, self)

    @property
    def nonce(self) -> ttlv.TtlvStructureView | None:
        value = _field_value(lib.kmipkit_response_header_view_ttlv, self, _NONCE)
        return value if isinstance(value, ttlv.TtlvStructureView) else None

    @property
    def server_hashed_password(self) -> bytes | None:
        value = _field_value(lib.kmipkit_response_header_view_ttlv, self, _SERVER_HASHED_PASSWORD)
        return value if isinstance(value, bytes) else None

    @property
    def attestation_types(self) -> tuple[int, ...]:
        return tuple(
            int(value)
            for value in _field_values(
                lib.kmipkit_response_header_view_ttlv, self, _ATTESTATION_TYPE
            )
        )

    @property
    def client_correlation_value(self) -> str | None:
        return _field_value(
            lib.kmipkit_response_header_view_ttlv, self, _CLIENT_CORRELATION_VALUE
        )

    @property
    def server_correlation_value(self) -> str | None:
        return _field_value(
            lib.kmipkit_response_header_view_ttlv, self, _SERVER_CORRELATION_VALUE
        )


class RequestBatchItemView(_MessageHandle):
    """Read-only access to a validated Request Batch Item."""

    _release_name = "kmipkit_request_batch_item_release"

    @property
    def operation(self) -> int | None:
        return _optional_scalar_pair(
            lib.kmipkit_request_batch_item_view_operation, self, "uint32_t"
        )

    @property
    def ephemeral(self) -> bool | None:
        value = _field_value(lib.kmipkit_request_batch_item_view_ttlv, self, _EPHEMERAL)
        return value if isinstance(value, bool) else None

    @property
    def unique_batch_item_id(self) -> bytes | None:
        value = _field_value(
            lib.kmipkit_request_batch_item_view_ttlv, self, _UNIQUE_BATCH_ITEM_ID
        )
        return value if isinstance(value, bytes) else None

    @property
    def request_payload(self) -> ttlv.TtlvStructureView | None:
        value = _field_value(lib.kmipkit_request_batch_item_view_ttlv, self, _REQUEST_PAYLOAD)
        return value if isinstance(value, ttlv.TtlvStructureView) else None

    def to_ttlv(self) -> ttlv.Structure:
        return _copy_ttlv(lib.kmipkit_request_batch_item_view_ttlv, self)

    @property
    def extension_count(self) -> int:
        return _scalar(
            "uint64_t", lib.kmipkit_request_batch_item_extension_count, self._pointer()
        )

    def extension_at(self, index: int) -> MessageExtensionView:
        index = _checked_index(index)
        handle = _out_handle(
            "kmipkit_message_extension_view_t",
            lib.kmipkit_request_batch_item_extension_at,
            self._pointer(),
            index,
        )
        return _wrap_handle(MessageExtensionView, handle, owner=self)


class ResponseBatchItemView(_MessageHandle):
    """Read-only access to a validated Response Batch Item."""

    _release_name = "kmipkit_response_batch_item_release"

    @property
    def operation(self) -> int | None:
        return _optional_scalar_pair(
            lib.kmipkit_response_batch_item_view_operation, self, "uint32_t"
        )

    @property
    def unique_batch_item_id(self) -> bytes | None:
        value = _field_value(
            lib.kmipkit_response_batch_item_view_ttlv, self, _UNIQUE_BATCH_ITEM_ID
        )
        return value if isinstance(value, bytes) else None

    @property
    def result_status(self) -> int | None:
        return _optional_scalar_pair(
            lib.kmipkit_response_batch_item_view_result_status, self, "uint32_t"
        )

    @property
    def result_reason(self) -> int | None:
        return _field_value(lib.kmipkit_response_batch_item_view_ttlv, self, _RESULT_REASON)

    @property
    def result_message(self) -> str | None:
        return _field_value(lib.kmipkit_response_batch_item_view_ttlv, self, _RESULT_MESSAGE)

    @property
    def asynchronous_correlation_value(self) -> bytes | None:
        return _field_value(
            lib.kmipkit_response_batch_item_view_ttlv,
            self,
            _ASYNCHRONOUS_CORRELATION_VALUE,
        )

    @property
    def response_payload(self) -> ttlv.TtlvStructureView | None:
        value = _field_value(lib.kmipkit_response_batch_item_view_ttlv, self, _RESPONSE_PAYLOAD)
        return value if isinstance(value, ttlv.TtlvStructureView) else None

    def to_ttlv(self) -> ttlv.Structure:
        return _copy_ttlv(lib.kmipkit_response_batch_item_view_ttlv, self)

    @property
    def extension_count(self) -> int:
        return _scalar(
            "uint64_t", lib.kmipkit_response_batch_item_extension_count, self._pointer()
        )

    def extension_at(self, index: int) -> MessageExtensionView:
        index = _checked_index(index)
        handle = _out_handle(
            "kmipkit_message_extension_view_t",
            lib.kmipkit_response_batch_item_extension_at,
            self._pointer(),
            index,
        )
        return _wrap_handle(MessageExtensionView, handle, owner=self)


class MessageExtensionView(_MessageHandle):
    """Read-only access to an opaque Message Extension."""

    _release_name = "kmipkit_message_extension_view_release"

    def to_ttlv(self) -> ttlv.Structure:
        return _copy_ttlv(lib.kmipkit_message_extension_view_ttlv, self)

    @property
    def vendor_identification(self) -> str | None:
        return _field_value(
            lib.kmipkit_message_extension_view_ttlv, self, _VENDOR_IDENTIFICATION
        )

    @property
    def criticality_indicator(self) -> bool | None:
        return _field_value(
            lib.kmipkit_message_extension_view_ttlv, self, _CRITICALITY_INDICATOR
        )

    @property
    def vendor_extension(self) -> ttlv.TtlvStructureView | None:
        value = _field_value(lib.kmipkit_message_extension_view_ttlv, self, _VENDOR_EXTENSION)
        return value if isinstance(value, ttlv.TtlvStructureView) else None


__all__ = [
    "MessageExtensionView",
    "MessageValidationError",
    "RequestBatchItemView",
    "RequestHeaderView",
    "RequestMessage",
    "ResponseBatchItemView",
    "ResponseHeaderView",
    "ResponseMessage",
]
