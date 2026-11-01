"""Standalone typed request models for KMIP 2.1 attribute operations."""

from __future__ import annotations

from typing import Any, ClassVar

from . import errors, extensions, ttlv
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._ffi import ffi
from ._handles import NativeHandle, _wrap_owned_handle, synchronized_native_call


class _AttributeRequest(NativeHandle):
    __slots__ = ("_unique_identifier", "_attribute_reference_value", "_adjustment_type")

    _request_type: ClassVar[str]
    _payload_function: ClassVar[str]
    _batch_function: ClassVar[str]
    _operation: ClassVar[str]

    def __init__(
        self,
        handle: Any,
        unique_identifier: extensions.UniqueIdentifier | None,
        attribute_reference: extensions.AttributeReference | None,
        adjustment_type: int | None,
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, _token=_token)
        self._unique_identifier = unique_identifier
        self._attribute_reference_value = attribute_reference
        self._adjustment_type = adjustment_type

    @property
    def unique_identifier(self) -> extensions.UniqueIdentifier | None:
        """Return the exact optional Unique Identifier supplied at construction."""
        self._pointer()
        return self._unique_identifier

    def _reference_value(self) -> extensions.AttributeReference | None:
        self._pointer()
        return self._attribute_reference_value

    def _native_request(self) -> Any:
        return _client_ffi.cast(
            f"{self._request_type} *", int(ffi.cast("uintptr_t", self._pointer()))
        )

    def _copy_ttlv_handle(
        self, getter: str, output_type: str, *, optional: bool
    ) -> ttlv.TtlvItem | ttlv.Value | None:
        request = self._native_request()
        output = _client_ffi.new(f"{output_type} **")
        if optional:
            present = _client_ffi.new("uint8_t *")
            status = int(getattr(_client_lib, getter)(request, present, output))
        else:
            present = None
            status = int(getattr(_client_lib, getter)(request, output))
        errors.raise_for_status(status)
        if optional and not present[0]:
            return None
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        native_type = "kmipkit_ttlv_item_t" if "item_t" in output_type else "kmipkit_ttlv_value_t"
        wrapper = ttlv.TtlvItem if native_type.endswith("item_t") else ttlv.Value
        return _wrap_owned_handle(wrapper, ffi.cast(f"{native_type} *", address))

    def to_ttlv_payload(self) -> ttlv.Structure:
        """Return an independently owned generic TTLV copy of this request payload."""
        output = _client_ffi.new("kmipkit_ttlv_structure_t **")
        status = int(getattr(_client_lib, self._payload_function)(self._native_request(), output))
        errors.raise_for_status(status)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        return ttlv._structure_from_native_address(address)

    def to_batch_item(self) -> extensions.ClientBatchItem:
        """Create a batch item copy while leaving this standalone request open."""
        handle = extensions._make_handle(
            "kmipkit_client_batch_item_t",
            self._batch_function,
            (self._native_request(),),
        )
        return _wrap_owned_handle(extensions.ClientBatchItem, handle)


def _validate_identifier(value: extensions.UniqueIdentifier | None) -> None:
    if value is not None and (
        not isinstance(value, extensions.UniqueIdentifier)
        or value.kind is not extensions.UniqueIdentifierKind.TEXT_STRING
    ):
        errors.raise_for_status(4)


class AttributeSet(NativeHandle):
    """Owned validated direct Attribute Items in their original order."""

    __slots__ = ()
    _release_name = "kmipkit_attribute_set_release"

    def _native_attribute_set(self) -> Any:
        address = int(ffi.cast("uintptr_t", self._pointer()))
        return _client_ffi.cast("kmipkit_attribute_set_t *", address)

    @classmethod
    @synchronized_native_call
    def try_new(cls, items: ttlv.Structure) -> AttributeSet:
        """Copy and validate direct Items from a generic TTLV Structure."""
        if not isinstance(items, ttlv.Structure):
            errors.raise_for_status(4)
        output = _client_ffi.new("kmipkit_attribute_set_t **")
        status = int(
            _client_lib.kmipkit_attribute_set_create(
                extensions._client_native_structure(items, required=True), output
            )
        )
        errors.raise_for_status(status)
        return _wrap_owned_handle(cls, output[0])

    @synchronized_native_call
    def try_push(self, item: ttlv.TtlvItem) -> None:
        """Copy and validate one Item; a rejected value leaves this set unchanged."""
        if not isinstance(item, ttlv.TtlvItem):
            errors.raise_for_status(4)
        status = int(
            _client_lib.kmipkit_attribute_set_try_push(
                self._native_attribute_set(), extensions._client_native_item(item)
            )
        )
        errors.raise_for_status(status)

    @synchronized_native_call
    def as_items(self) -> ttlv.Structure:
        """Return an independently owned generic Structure copy in source order."""
        output = _client_ffi.new("kmipkit_ttlv_structure_t **")
        status = int(
            _client_lib.kmipkit_attribute_set_as_items(
                self._native_attribute_set(), output
            )
        )
        errors.raise_for_status(status)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        return ttlv._structure_from_native_address(address)

    @synchronized_native_call
    def len(self) -> int:
        """Return the number of direct Items."""
        output = _client_ffi.new("uint64_t *")
        status = int(
            _client_lib.kmipkit_attribute_set_len(self._native_attribute_set(), output)
        )
        errors.raise_for_status(status)
        return int(output[0])

    @synchronized_native_call
    def is_empty(self) -> bool:
        """Return whether this collection has no direct Items."""
        output = _client_ffi.new("uint8_t *")
        status = int(
            _client_lib.kmipkit_attribute_set_is_empty(
                self._native_attribute_set(), output
            )
        )
        errors.raise_for_status(status)
        if output[0] > 1:
            errors.raise_for_status(4)
        return bool(output[0])

    @synchronized_native_call
    def into_items(self) -> ttlv.Structure:
        """Consume this set and return its direct Items in original order."""
        output = _client_ffi.new("kmipkit_ttlv_structure_t **")
        status = int(
            _client_lib.kmipkit_attribute_set_into_items(
                self._native_attribute_set(), output
            )
        )
        errors.raise_for_status(status)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        structure = ttlv._structure_from_native_address(address)
        self._take()
        return structure


class AddAttributeRequest(_AttributeRequest):
    """Owned, inspectable KMIP Add Attribute Request model."""

    _release_name = "kmipkit_add_attribute_request_release"
    _request_type = "kmipkit_add_attribute_request_t"
    _payload_function = "kmipkit_add_attribute_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_add_attribute_from_request"

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        new_attribute: ttlv.TtlvItem,
    ) -> AddAttributeRequest:
        """Copy an optional identifier and required New Attribute Item."""
        _validate_identifier(unique_identifier)
        if not isinstance(new_attribute, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            native_item = extensions._client_native_item(new_attribute)
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_add_attribute_request_create",
                (identifier, native_item),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, None, None)

    @property
    def new_attribute(self) -> ttlv.TtlvItem:
        """Return an independently owned copy of New Attribute."""
        return self._copy_ttlv_handle(
            "kmipkit_add_attribute_request_new_attribute",
            "kmipkit_ttlv_item_t",
            optional=False,
        )  # type: ignore[return-value]


class AdjustAttributeRequest(_AttributeRequest):
    """Owned, inspectable KMIP Adjust Attribute Request model."""

    _release_name = "kmipkit_adjust_attribute_request_release"
    _request_type = "kmipkit_adjust_attribute_request_t"
    _payload_function = "kmipkit_adjust_attribute_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_adjust_attribute_from_request"

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        attribute_reference: extensions.AttributeReference,
        adjustment_type: int,
        adjustment_value: ttlv.Value | None = None,
    ) -> AdjustAttributeRequest:
        """Copy the reference and optional value, preserving raw Adjustment Type."""
        _validate_identifier(unique_identifier)
        if not isinstance(attribute_reference, extensions.AttributeReference):
            errors.raise_for_status(4)
        raw_adjustment_type = extensions._integer_range(adjustment_type, 0, 0xFFFF_FFFF)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            reference = extensions._client_native_attribute_reference(
                attribute_reference, staging_buffers
            )
            native_value = extensions._client_native_value(adjustment_value)
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_adjust_attribute_request_create",
                (identifier, reference, raw_adjustment_type, native_value),
            )
        return _wrap_owned_handle(
            cls, handle, unique_identifier, attribute_reference, raw_adjustment_type
        )

    @property
    def adjustment_type(self) -> int:
        """Return the raw Adjustment Type value, including future values."""
        self._pointer()
        value = self._adjustment_type
        if value is None:
            errors.raise_for_status(4)
        return value

    @property
    def adjustment_value(self) -> ttlv.Value | None:
        """Return a copied optional Adjustment Value."""
        return self._copy_ttlv_handle(
            "kmipkit_adjust_attribute_request_adjustment_value",
            "kmipkit_ttlv_value_t",
            optional=True,
        )  # type: ignore[return-value]

    @property
    def attribute_reference(self) -> extensions.AttributeReference:
        """Return the required exact Attribute Reference."""
        reference = self._reference_value()
        if reference is None:
            errors.raise_for_status(4)
        return reference


class DeleteAttributeRequest(_AttributeRequest):
    """Owned, inspectable KMIP Delete Attribute Request model."""

    _release_name = "kmipkit_delete_attribute_request_release"
    _request_type = "kmipkit_delete_attribute_request_t"
    _payload_function = "kmipkit_delete_attribute_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_delete_attribute_from_request"

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        current_attribute: ttlv.TtlvItem | None,
        attribute_reference: extensions.AttributeReference | None,
    ) -> DeleteAttributeRequest:
        """Copy the two independently optional Attribute selectors."""
        _validate_identifier(unique_identifier)
        if current_attribute is not None and not isinstance(current_attribute, ttlv.TtlvItem):
            errors.raise_for_status(4)
        if attribute_reference is not None and not isinstance(
            attribute_reference, extensions.AttributeReference
        ):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            native_item = (
                extensions._client_native_item(current_attribute)
                if current_attribute is not None
                else _client_ffi.NULL
            )
            reference = (
                extensions._client_native_attribute_reference(
                    attribute_reference, staging_buffers
                )
                if attribute_reference is not None
                else _client_ffi.NULL
            )
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_delete_attribute_request_create",
                (identifier, native_item, reference),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, attribute_reference, None)

    @property
    def current_attribute(self) -> ttlv.TtlvItem | None:
        """Return a copied optional Current Attribute Item."""
        return self._copy_ttlv_handle(
            "kmipkit_delete_attribute_request_current_attribute",
            "kmipkit_ttlv_item_t",
            optional=True,
        )  # type: ignore[return-value]

    @property
    def attribute_reference(self) -> extensions.AttributeReference | None:
        """Return the independently optional Attribute Reference selector."""
        return self._reference_value()


class ModifyAttributeRequest(_AttributeRequest):
    """Owned, inspectable KMIP Modify Attribute Request model."""

    _release_name = "kmipkit_modify_attribute_request_release"
    _request_type = "kmipkit_modify_attribute_request_t"
    _payload_function = "kmipkit_modify_attribute_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_modify_attribute_from_request"

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        current_attribute: ttlv.TtlvItem | None,
        new_attribute: ttlv.TtlvItem,
    ) -> ModifyAttributeRequest:
        """Copy optional Current Attribute and required New Attribute Items."""
        _validate_identifier(unique_identifier)
        if current_attribute is not None and not isinstance(current_attribute, ttlv.TtlvItem):
            errors.raise_for_status(4)
        if not isinstance(new_attribute, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            native_current = (
                extensions._client_native_item(current_attribute)
                if current_attribute is not None
                else _client_ffi.NULL
            )
            native_new = extensions._client_native_item(new_attribute)
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_modify_attribute_request_create",
                (identifier, native_current, native_new),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, None, None)

    @property
    def current_attribute(self) -> ttlv.TtlvItem | None:
        """Return a copied optional Current Attribute Item."""
        return self._copy_ttlv_handle(
            "kmipkit_modify_attribute_request_current_attribute",
            "kmipkit_ttlv_item_t",
            optional=True,
        )  # type: ignore[return-value]

    @property
    def new_attribute(self) -> ttlv.TtlvItem:
        """Return an independently owned copy of New Attribute."""
        return self._copy_ttlv_handle(
            "kmipkit_modify_attribute_request_new_attribute",
            "kmipkit_ttlv_item_t",
            optional=False,
        )  # type: ignore[return-value]


class SetAttributeRequest(_AttributeRequest):
    """Owned, inspectable KMIP Set Attribute Request model."""

    _release_name = "kmipkit_set_attribute_request_release"
    _request_type = "kmipkit_set_attribute_request_t"
    _payload_function = "kmipkit_set_attribute_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_set_attribute_from_request"

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        new_attribute: ttlv.TtlvItem,
    ) -> SetAttributeRequest:
        """Copy an optional identifier and required New Attribute Item."""
        _validate_identifier(unique_identifier)
        if not isinstance(new_attribute, ttlv.TtlvItem):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            native_item = extensions._client_native_item(new_attribute)
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_set_attribute_request_create",
                (identifier, native_item),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, None, None)

    @property
    def new_attribute(self) -> ttlv.TtlvItem:
        """Return an independently owned copy of New Attribute."""
        return self._copy_ttlv_handle(
            "kmipkit_set_attribute_request_new_attribute",
            "kmipkit_ttlv_item_t",
            optional=False,
        )  # type: ignore[return-value]


class GetAttributesRequest(_AttributeRequest):
    """Owned KMIP Get Attributes request with ordered Attribute References."""

    __slots__ = ("_references",)
    _release_name = "kmipkit_get_attributes_request_release"
    _request_type = "kmipkit_get_attributes_request_t"
    _payload_function = "kmipkit_get_attributes_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_get_attributes_from_request"

    def __init__(
        self,
        handle: Any,
        identifier: extensions.UniqueIdentifier | None,
        _attribute_reference: extensions.AttributeReference | None,
        _adjustment_type: int | None,
        references: tuple[extensions.AttributeReference, ...],
        *,
        _token: object,
    ) -> None:
        super().__init__(handle, identifier, None, None, _token=_token)
        self._references = references

    @classmethod
    def create(
        cls,
        unique_identifier: extensions.UniqueIdentifier | None,
        attribute_references: list[extensions.AttributeReference]
        | tuple[extensions.AttributeReference, ...],
    ) -> GetAttributesRequest:
        """Copy an optional identifier and ordered, unique reference sequence."""
        _validate_identifier(unique_identifier)
        references = extensions._bounded_snapshot(
            attribute_references, extensions._MAX_ATTRIBUTE_REFERENCES
        )
        if any(
            not isinstance(reference, extensions.AttributeReference)
            for reference in references
        ):
            errors.raise_for_status(4)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            native_references, count = extensions._client_native_attribute_reference_array(
                references, staging_buffers
            )
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_get_attributes_request_create",
                (
                    identifier,
                    native_references if native_references is not None else _client_ffi.NULL,
                    count,
                ),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, None, None, references)

    @property
    def attribute_references(self) -> tuple[extensions.AttributeReference, ...]:
        """Return the request's ordered Attribute References."""
        self._pointer()
        return self._references


class GetAttributeListRequest(_AttributeRequest):
    """Owned KMIP Get Attribute List request model."""

    _release_name = "kmipkit_get_attribute_list_request_release"
    _request_type = "kmipkit_get_attribute_list_request_t"
    _payload_function = "kmipkit_get_attribute_list_request_to_ttlv_payload"
    _batch_function = "kmipkit_client_batch_item_get_attribute_list_from_request"

    @classmethod
    def create(
        cls, unique_identifier: extensions.UniqueIdentifier | None
    ) -> GetAttributeListRequest:
        """Copy the optional Unique Identifier from KMIP Table 226."""
        _validate_identifier(unique_identifier)
        with extensions._staging_buffer_scope() as staging_buffers:
            identifier = extensions._client_native_unique_identifier(
                unique_identifier, staging_buffers
            )
            handle = extensions._make_handle(
                cls._request_type,
                "kmipkit_get_attribute_list_request_create",
                (identifier,),
            )
        return _wrap_owned_handle(cls, handle, unique_identifier, None, None)


__all__ = [
    "AttributeSet",
    "AddAttributeRequest",
    "AdjustAttributeRequest",
    "DeleteAttributeRequest",
    "GetAttributeListRequest",
    "GetAttributesRequest",
    "ModifyAttributeRequest",
    "SetAttributeRequest",
]
