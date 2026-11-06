"""Owned, typed views for KMIP managed objects and key structures."""

from __future__ import annotations

from typing import Any, Self

from . import errors, extensions, ttlv
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._handles import _wrap_owned_handle, synchronized_native_call


def _from_item(
    wrapper: type["ManagedTtlvItem"],
    function: Any,
    item: ttlv.TtlvItem,
    *arguments: int,
    metadata: int | None = None,
) -> "ManagedTtlvItem":
    if not isinstance(item, ttlv.TtlvItem):
        errors.raise_for_status(4)
    output = _client_ffi.new("kmipkit_ttlv_item_t **")
    status = int(function(*arguments, extensions._client_native_item(item), output))
    errors.raise_for_status(status)
    address = int(_client_ffi.cast("uintptr_t", output[0]))
    handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
    return _wrap_owned_handle(wrapper, handle, metadata=metadata)


class ManagedTtlvItem(ttlv.TtlvItem):
    """Typed wrapper around an independently owned generic TTLV Item."""

    __slots__ = ("_metadata",)

    def __init__(self, handle: Any, *, _token: object, metadata: int | None = None) -> None:
        super().__init__(handle, _token=_token)
        self._metadata = metadata

    def _metadata_value(self) -> int:
        self._pointer()
        metadata = self._metadata
        if metadata is None:
            errors.raise_for_status(4)
        return metadata

    def __repr__(self) -> str:
        state = "closed" if self.closed else "open"
        return f"{type(self).__name__}({state}, value=[REDACTED])"


class ManagedObject(ManagedTtlvItem):
    """Known or future KMIP managed object with lossless generic TTLV access."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, object_type: int, item: ttlv.TtlvItem) -> Self:
        """Copy and validate a managed object; unknown object values stay generic."""
        if (
            not isinstance(object_type, int)
            or isinstance(object_type, bool)
            or not 0 <= object_type <= 0xFFFF_FFFF
        ):
            errors.raise_for_status(4)
        return _from_item(
            cls,
            _client_lib.kmipkit_managed_object_from_item,
            item,
            object_type,
            metadata=object_type,
        )  # type: ignore[return-value]

    @property
    def object_type(self) -> int:
        return self._metadata_value()


class KeyBlock(ManagedTtlvItem):
    """Validated KMIP Key Block retaining child order and unknown Items."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, item: ttlv.TtlvItem) -> Self:
        return _from_item(cls, _client_lib.kmipkit_key_block_from_item, item)  # type: ignore[return-value]


class KeyValue(ManagedTtlvItem):
    """Validated plaintext KMIP Key Value for one explicit key format."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, format_raw: int, item: ttlv.TtlvItem) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_item(cls, _client_lib.kmipkit_key_value_from_item, item, format_raw, metadata=format_raw)  # type: ignore[return-value]

    @property
    def format_raw(self) -> int:
        return self._metadata_value()


class KeyMaterial(ManagedTtlvItem):
    """Validated KMIP Key Material with caller-selected open Key Format Type."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, format_raw: int, item: ttlv.TtlvItem) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_item(cls, _client_lib.kmipkit_key_material_from_item, item, format_raw, metadata=format_raw)  # type: ignore[return-value]

    @property
    def format_raw(self) -> int:
        return self._metadata_value()


class KeyWrappingData(ManagedTtlvItem):
    """Validated KMIP Key Wrapping Data with lossless nested TTLV access."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, item: ttlv.TtlvItem) -> Self:
        return _from_item(cls, _client_lib.kmipkit_key_wrapping_data_from_item, item)  # type: ignore[return-value]


__all__ = ["KeyBlock", "KeyMaterial", "KeyValue", "KeyWrappingData", "ManagedObject"]
