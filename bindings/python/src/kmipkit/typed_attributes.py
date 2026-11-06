"""Catalog-backed typed wrappers for standard KMIP direct attributes."""

from __future__ import annotations

from typing import Any, Self

from . import errors, extensions, ttlv
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._handles import synchronized_native_call, _wrap_owned_handle
from ._generated.standard_attribute import StandardAttribute


class TypedAttribute(ttlv.TtlvItem):
    """Owned Item checked against one catalogued direct attribute definition."""

    __slots__ = ("_attribute",)

    def __init__(self, handle: Any, *, _token: object, attribute: StandardAttribute) -> None:
        super().__init__(handle, _token=_token)
        self._attribute = attribute

    @classmethod
    @synchronized_native_call
    def from_item(cls, attribute: StandardAttribute, item: ttlv.TtlvItem) -> Self:
        """Copy an Item after checking its assigned tag and catalogued TTLV type."""
        if not isinstance(attribute, StandardAttribute) or not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        output = _client_ffi.new("kmipkit_ttlv_item_t **")
        status = int(
            _client_lib.kmipkit_typed_attribute_from_item(
                attribute.tag, extensions._client_native_item(item), output
            )
        )
        errors.raise_for_status(status)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
        return _wrap_owned_handle(cls, handle, attribute=attribute)

    @property
    def attribute(self) -> StandardAttribute:
        """Return the source catalog definition associated with this checked Item."""
        self._pointer()
        return self._attribute

    @synchronized_native_call
    def as_item(self) -> ttlv.TtlvItem:
        """Return an independently owned generic Item copy."""
        output = _client_ffi.new("kmipkit_ttlv_item_t **")
        status = int(
            _client_lib.kmipkit_typed_attribute_from_item(
                self._attribute.tag, extensions._client_native_item(self), output
            )
        )
        errors.raise_for_status(status)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
        return _wrap_owned_handle(ttlv.TtlvItem, handle)

    def view(self) -> ttlv.TtlvItemView:
        """Return a borrowed TTLV view that retains this typed Item as owner."""
        return ttlv.ttlv_item_view_from_item(self)


__all__ = ["StandardAttribute", "TypedAttribute"]
