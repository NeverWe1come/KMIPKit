"""Typed, lossless KMIP Request and Response Message inspection."""

from __future__ import annotations

from typing import Any, Self, TypeVar, cast

from . import errors, ttlv
from ._client_ffi import ffi
from ._client_ffi import lib as _native_lib
from ._ffi import ffi as _base_ffi
from ._handles import _NATIVE_CALL_LOCK, NativeHandle, _wrap_owned_handle
from .client import ProtocolVersion

lib: Any = _native_lib

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
    """A redacted message error containing only a stable kind and optional indexes."""

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
        message = (
            f"KMIPKit message validation failed (kind {self.kind}; "
            f"structure index {structure_index}; field index {field_index})"
        )
        self.args = (message,)


class _MessageHandle(NativeHandle):
    """An opaque message-model handle released by the private CFFI bridge."""

    def close(self) -> None:
        with _NATIVE_CALL_LOCK:
            if self.closed:
                return
            handle, owner = self._take()
            getattr(_native_lib, self._release_name)(handle)
            del owner


_MessageHandleT = TypeVar("_MessageHandleT", bound=_MessageHandle)
_FieldValueT = TypeVar("_FieldValueT")


def _call(function: Any, *arguments: object) -> int:
    with _NATIVE_CALL_LOCK:
        try:
            return int(function(*arguments))
        except (OverflowError, TypeError, ValueError):
            raise errors.InvalidInputError() from None


def _status(status: int) -> None:
    errors.raise_for_status(status)


def _scalar(c_type: str, function: Any, *arguments: object) -> int:
    output = ffi.new(f"{c_type} *")
    _status(_call(function, *arguments, output))
    return int(output[0])


def _optional_scalar_pair(
    function: Any, handle: NativeHandle, value_type: str
) -> int | None:
    present = ffi.new("uint8_t *")
    value = ffi.new(f"{value_type} *")
    _status(_call(function, handle._pointer(), present, value))
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    return int(value[0]) if present[0] == 1 else None


def _out_handle(c_type: str, function: Any, *arguments: object) -> Any:
    output = ffi.new(f"{c_type} **")
    _status(_call(function, *arguments, output))
    if output[0] == ffi.NULL:
        errors.raise_for_status(4)
    return output[0]


def _wrap_handle(
    wrapper: type[_MessageHandleT], handle: object, owner: object | None = None
) -> _MessageHandleT:
    return _wrap_owned_handle(wrapper, handle, owner=owner)


def _as_client_structure(structure: ttlv.Structure) -> object:
    address = int(_base_ffi.cast("uintptr_t", structure._pointer()))
    return ffi.cast("kmipkit_ttlv_structure_t *", address)


def _consume_and_validate_message(
    structure: ttlv.Structure,
    *,
    handle_type: str,
    wrapper: type[_MessageHandleT],
    constructor: Any,
    error_release: Any,
    message_release: Any,
) -> _MessageHandleT:
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
                kind = _scalar(
                    "uint32_t", lib.kmipkit_message_validation_error_kind, error_handle
                )
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


def _optional_index(function: Any, handle: object) -> int | None:
    present = ffi.new("uint8_t *")
    value = ffi.new("uint64_t *")
    _status(_call(function, handle, present, value))
    if present[0] not in (0, 1):
        errors.raise_for_status(4)
    if present[0] == 0:
        return None
    return int(value[0])


def _copy_ttlv(function: Any, handle: NativeHandle) -> ttlv.Structure:
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
    if value_type in (
        ttlv.ItemType.BigInteger,
        ttlv.ItemType.TextString,
        ttlv.ItemType.ByteString,
    ):
        length = ttlv.ttlv_value_view_byte_length(value_view)
        if length > _MAX_MESSAGE_BYTES:
            errors.raise_for_status(6)
        raw = bytes(
            ttlv.ttlv_value_view_byte_at(value_view, index) for index in range(length)
        )
        if value_type == ttlv.ItemType.TextString:
            try:
                return raw.decode("utf-8")
            except UnicodeDecodeError:
                errors.raise_for_status(4)
        return raw
    errors.raise_for_status(4)


def _field_values(
    copy_function: Any,
    handle: NativeHandle,
    raw_tag: int,
) -> tuple[object, ...]:
    structure = _copy_ttlv(copy_function, handle)
    values: list[object] = []
    try:
        with ttlv.ttlv_structure_view(structure) as structure_view:
            count = ttlv.ttlv_structure_view_item_count(structure_view)
            for index in range(count):
                with ttlv.ttlv_structure_view_item_at(
                    structure_view, index
                ) as item_view:
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


def _field_value(
    copy_function: Any, handle: NativeHandle, raw_tag: int
) -> object | None:
    values = _field_values(copy_function, handle, raw_tag)
    if not values:
        return None
    if len(values) != 1:
        for value in values:
            if isinstance(value, NativeHandle):
                value.close()
        errors.raise_for_status(4)
    return values[0]


def _typed_field_value(
    copy_function: Any,
    handle: NativeHandle,
    raw_tag: int,
    value_type: type[_FieldValueT],
) -> _FieldValueT | None:
    value = _field_value(copy_function, handle, raw_tag)
    if value is None:
        return None
    if not isinstance(value, value_type):
        errors.raise_for_status(4)
    return cast(_FieldValueT, value)


def _typed_field_values(
    copy_function: Any,
    handle: NativeHandle,
    raw_tag: int,
    value_type: type[_FieldValueT],
) -> tuple[_FieldValueT, ...]:
    values = _field_values(copy_function, handle, raw_tag)
    if any(not isinstance(value, value_type) for value in values):
        errors.raise_for_status(4)
    return tuple(cast(_FieldValueT, value) for value in values)


def _copy_message_ttlv(handle: NativeHandle, function: Any) -> ttlv.Structure:
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
        return handle

    @property
    def batch_item_count(self) -> int:
        """Return the number of validated batch items in source order."""
        return _scalar(
            "uint64_t", lib.kmipkit_request_message_batch_item_count, self._pointer()
        )

    def header(self) -> RequestHeaderView:
        """Return an owned, closeable view of the validated Request Header."""
        handle = _out_handle(
            "kmipkit_request_header_view_t",
            lib.kmipkit_request_message_header,
            self._pointer(),
        )
        return _wrap_handle(RequestHeaderView, handle, owner=self)

    def batch_item_at(self, index: int) -> RequestBatchItemView:
        """Return an owned view of the indexed request batch item."""
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
        return handle

    @property
    def batch_item_count(self) -> int:
        """Return the number of validated batch items in source order."""
        return _scalar(
            "uint64_t", lib.kmipkit_response_message_batch_item_count, self._pointer()
        )

    def header(self) -> ResponseHeaderView:
        """Return an owned, closeable view of the validated Response Header."""
        handle = _out_handle(
            "kmipkit_response_header_view_t",
            lib.kmipkit_response_message_header,
            self._pointer(),
        )
        return _wrap_handle(ResponseHeaderView, handle, owner=self)

    def batch_item_at(self, index: int) -> ResponseBatchItemView:
        """Return an owned view of the indexed response batch item."""
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
    if (
        isinstance(index, bool)
        or not isinstance(index, int)
        or not 0 <= index <= _MAX_U64
    ):
        errors.raise_for_status(4)
    return index


class RequestHeaderView(_MessageHandle):
    """Read-only typed access to a validated Request Header."""

    _release_name = "kmipkit_request_header_view_release"

    @property
    def protocol_version(self) -> ProtocolVersion:
        """Return the raw signed protocol version pair."""
        version = ffi.new("kmipkit_protocol_version_v1_t *")
        _status(
            _call(
                lib.kmipkit_request_header_view_protocol_version,
                self._pointer(),
                version,
            )
        )
        version_value = cast(Any, version)
        return ProtocolVersion(int(version_value.major), int(version_value.minor))

    @property
    def batch_count(self) -> int:
        """Return the validated positive Batch Count."""
        return _scalar(
            "int32_t", lib.kmipkit_request_header_view_batch_count, self._pointer()
        )

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete header in source order."""
        return _copy_ttlv(lib.kmipkit_request_header_view_ttlv, self)

    @property
    def maximum_response_size(self) -> int | None:
        """Return the optional Maximum Response Size integer."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv, self, _MAXIMUM_RESPONSE_SIZE, int
        )

    @property
    def client_correlation_value(self) -> str | None:
        """Return the optional Client Correlation Value."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _CLIENT_CORRELATION_VALUE,
            str,
        )

    @property
    def asynchronous_indicator(self) -> int | None:
        """Return the optional raw Asynchronous Indicator Enumeration."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _ASYNCHRONOUS_INDICATOR,
            int,
        )

    @property
    def effective_asynchronous_indicator(self) -> int:
        """Return the raw indicator or its OASIS default, Prohibited (`3`)."""
        value = self.asynchronous_indicator
        return 3 if value is None else int(value)

    @property
    def attestation_capable_indicator(self) -> bool | None:
        """Return the optional Attestation Capable Indicator."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _ATTESTATION_CAPABLE_INDICATOR,
            bool,
        )

    @property
    def effective_attestation_capable_indicator(self) -> bool:
        """Return the indicator or its OASIS default, `False`."""
        value = self.attestation_capable_indicator
        return False if value is None else bool(value)

    @property
    def attestation_types(self) -> tuple[int, ...]:
        """Return raw Attestation Type Enumeration values in source order."""
        return tuple(
            int(value)
            for value in _typed_field_values(
                lib.kmipkit_request_header_view_ttlv, self, _ATTESTATION_TYPE, int
            )
        )

    @property
    def authentication(self) -> ttlv.TtlvStructureView | None:
        """Return the optional Authentication Structure as a closeable TTLV view."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _AUTHENTICATION,
            ttlv.TtlvStructureView,
        )

    @property
    def batch_error_continuation_option(self) -> int | None:
        """Return the optional raw Batch Error Continuation Enumeration."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv,
            self,
            _BATCH_ERROR_CONTINUATION_OPTION,
            int,
        )

    @property
    def effective_batch_error_continuation_option(self) -> int:
        """Return the option or its OASIS default, Stop (`2`)."""
        value = self.batch_error_continuation_option
        return 2 if value is None else int(value)

    @property
    def batch_order_option(self) -> bool | None:
        """Return the optional Batch Order Option value."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv, self, _BATCH_ORDER_OPTION, bool
        )

    @property
    def effective_batch_order_option(self) -> bool:
        """Return the option or its OASIS default, `True`."""
        value = self.batch_order_option
        return True if value is None else bool(value)

    @property
    def time_stamp(self) -> int | None:
        """Return the optional Request Header Date-Time value unchanged."""
        return _typed_field_value(
            lib.kmipkit_request_header_view_ttlv, self, _TIME_STAMP, int
        )


class ResponseHeaderView(_MessageHandle):
    """Read-only typed access to a validated Response Header."""

    _release_name = "kmipkit_response_header_view_release"

    @property
    def protocol_version(self) -> ProtocolVersion:
        """Return the raw signed protocol version pair."""
        version = ffi.new("kmipkit_protocol_version_v1_t *")
        _status(
            _call(
                lib.kmipkit_response_header_view_protocol_version,
                self._pointer(),
                version,
            )
        )
        version_value = cast(Any, version)
        return ProtocolVersion(int(version_value.major), int(version_value.minor))

    @property
    def time_stamp(self) -> int:
        """Return the required response Date-Time value unchanged."""
        return _scalar(
            "int64_t", lib.kmipkit_response_header_view_time_stamp, self._pointer()
        )

    @property
    def batch_count(self) -> int:
        """Return the validated positive Batch Count."""
        return _scalar(
            "int32_t", lib.kmipkit_response_header_view_batch_count, self._pointer()
        )

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete header in source order."""
        return _copy_ttlv(lib.kmipkit_response_header_view_ttlv, self)

    @property
    def nonce(self) -> ttlv.TtlvStructureView | None:
        """Return the optional Nonce Structure as a closeable TTLV view."""
        return _typed_field_value(
            lib.kmipkit_response_header_view_ttlv,
            self,
            _NONCE,
            ttlv.TtlvStructureView,
        )

    @property
    def server_hashed_password(self) -> bytes | None:
        """Return an immutable caller-owned copy that Python cannot erase deterministically."""
        return _typed_field_value(
            lib.kmipkit_response_header_view_ttlv,
            self,
            _SERVER_HASHED_PASSWORD,
            bytes,
        )

    @property
    def attestation_types(self) -> tuple[int, ...]:
        """Return raw Attestation Type Enumeration values in source order."""
        return tuple(
            int(value)
            for value in _typed_field_values(
                lib.kmipkit_response_header_view_ttlv, self, _ATTESTATION_TYPE, int
            )
        )

    @property
    def client_correlation_value(self) -> str | None:
        """Return the optional Client Correlation Value."""
        return _typed_field_value(
            lib.kmipkit_response_header_view_ttlv,
            self,
            _CLIENT_CORRELATION_VALUE,
            str,
        )

    @property
    def server_correlation_value(self) -> str | None:
        """Return the optional Server Correlation Value."""
        return _typed_field_value(
            lib.kmipkit_response_header_view_ttlv,
            self,
            _SERVER_CORRELATION_VALUE,
            str,
        )


class RequestBatchItemView(_MessageHandle):
    """Read-only access to a validated Request Batch Item."""

    _release_name = "kmipkit_request_batch_item_release"

    @property
    def operation(self) -> int | None:
        """Return the optional raw Operation Enumeration."""
        return _optional_scalar_pair(
            lib.kmipkit_request_batch_item_view_operation, self, "uint32_t"
        )

    @property
    def ephemeral(self) -> bool | None:
        """Return the optional Ephemeral Boolean."""
        return _typed_field_value(
            lib.kmipkit_request_batch_item_view_ttlv, self, _EPHEMERAL, bool
        )

    @property
    def unique_batch_item_id(self) -> bytes | None:
        """Return a caller-owned copy of the optional per-item identifier."""
        return _typed_field_value(
            lib.kmipkit_request_batch_item_view_ttlv,
            self,
            _UNIQUE_BATCH_ITEM_ID,
            bytes,
        )

    @property
    def request_payload(self) -> ttlv.TtlvStructureView | None:
        """Return the Request Payload as a closeable generic TTLV view."""
        return _typed_field_value(
            lib.kmipkit_request_batch_item_view_ttlv,
            self,
            _REQUEST_PAYLOAD,
            ttlv.TtlvStructureView,
        )

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete batch item in source order."""
        return _copy_ttlv(lib.kmipkit_request_batch_item_view_ttlv, self)

    @property
    def extension_count(self) -> int:
        """Return the number of Message Extensions in this item."""
        return _scalar(
            "uint64_t", lib.kmipkit_request_batch_item_extension_count, self._pointer()
        )

    def extension_at(self, index: int) -> MessageExtensionView:
        """Return an owned view of the indexed Message Extension."""
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
        """Return the optional raw Operation Enumeration."""
        return _optional_scalar_pair(
            lib.kmipkit_response_batch_item_view_operation, self, "uint32_t"
        )

    @property
    def unique_batch_item_id(self) -> bytes | None:
        """Return a caller-owned copy of the optional echoed per-item identifier."""
        return _typed_field_value(
            lib.kmipkit_response_batch_item_view_ttlv,
            self,
            _UNIQUE_BATCH_ITEM_ID,
            bytes,
        )

    @property
    def result_status(self) -> int | None:
        """Return the optional raw Result Status Enumeration."""
        return _optional_scalar_pair(
            lib.kmipkit_response_batch_item_view_result_status, self, "uint32_t"
        )

    @property
    def result_reason(self) -> int | None:
        """Return the optional raw Result Reason Enumeration."""
        return _typed_field_value(
            lib.kmipkit_response_batch_item_view_ttlv, self, _RESULT_REASON, int
        )

    @property
    def result_message(self) -> str | None:
        """Return the optional Result Message text."""
        return _typed_field_value(
            lib.kmipkit_response_batch_item_view_ttlv, self, _RESULT_MESSAGE, str
        )

    @property
    def asynchronous_correlation_value(self) -> bytes | None:
        """Return a caller-owned copy of the optional asynchronous correlation bytes."""
        return _typed_field_value(
            lib.kmipkit_response_batch_item_view_ttlv,
            self,
            _ASYNCHRONOUS_CORRELATION_VALUE,
            bytes,
        )

    @property
    def response_payload(self) -> ttlv.TtlvStructureView | None:
        """Return the Response Payload as a closeable generic TTLV view."""
        return _typed_field_value(
            lib.kmipkit_response_batch_item_view_ttlv,
            self,
            _RESPONSE_PAYLOAD,
            ttlv.TtlvStructureView,
        )

    def to_ttlv(self) -> ttlv.Structure:
        """Return an owned copy of the complete batch item in source order."""
        return _copy_ttlv(lib.kmipkit_response_batch_item_view_ttlv, self)

    @property
    def extension_count(self) -> int:
        """Return the number of Message Extensions in this item."""
        return _scalar(
            "uint64_t", lib.kmipkit_response_batch_item_extension_count, self._pointer()
        )

    def extension_at(self, index: int) -> MessageExtensionView:
        """Return an owned view of the indexed Message Extension."""
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
        """Return an owned copy of the complete extension in source order."""
        return _copy_ttlv(lib.kmipkit_message_extension_view_ttlv, self)

    @property
    def vendor_identification(self) -> str | None:
        """Return the Vendor Identification text."""
        return _typed_field_value(
            lib.kmipkit_message_extension_view_ttlv, self, _VENDOR_IDENTIFICATION, str
        )

    @property
    def criticality_indicator(self) -> bool | None:
        """Return the Criticality Indicator Boolean."""
        return _typed_field_value(
            lib.kmipkit_message_extension_view_ttlv, self, _CRITICALITY_INDICATOR, bool
        )

    @property
    def vendor_extension(self) -> ttlv.TtlvStructureView | None:
        """Return the opaque Vendor Extension as a closeable TTLV view."""
        return _typed_field_value(
            lib.kmipkit_message_extension_view_ttlv,
            self,
            _VENDOR_EXTENSION,
            ttlv.TtlvStructureView,
        )


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
