"""Owned, typed views for KMIP managed objects and key structures."""

from __future__ import annotations

from typing import Any, Self, Sequence

from . import errors, extensions, ttlv
from ._client_ffi import ffi as _client_ffi, lib as _client_lib
from ._handles import _wrap_owned_handle, synchronized_native_call


def _from_item(
    wrapper: type["ManagedTtlvItem"],
    function: Any,
    item: ttlv.TtlvItem,
    *arguments: int,
    metadata: int | None = None,
    owner_kind: int,
    context_raw: int = 0,
) -> "ManagedTtlvItem":
    if not isinstance(item, ttlv.TtlvItem):
        errors.raise_for_status(4)
    output = _client_ffi.new("kmipkit_ttlv_item_t **")
    status = int(function(*arguments, extensions._client_native_item(item), output))
    errors.raise_for_status(status)
    address = int(_client_ffi.cast("uintptr_t", output[0]))
    handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
    return _wrap_owned_handle(
        wrapper, handle, metadata=metadata, owner_kind=owner_kind, context_raw=context_raw
    )


def _from_fields(
    wrapper: type["ManagedTtlvItem"],
    function: Any,
    fields: Sequence[ttlv.TtlvItem],
    *arguments: int,
    metadata: int | None = None,
    owner_kind: int,
    context_raw: int = 0,
) -> "ManagedTtlvItem":
    if not isinstance(fields, (list, tuple)) or any(
        not isinstance(field, ttlv.TtlvItem) for field in fields
    ):
        errors.raise_for_status(4)
    field_handles = [extensions._client_native_item(field) for field in fields]
    native_fields = (
        _client_ffi.new("kmipkit_ttlv_item_t *[]", field_handles)
        if field_handles
        else _client_ffi.NULL
    )
    output = _client_ffi.new("kmipkit_ttlv_item_t **")
    status = int(function(*arguments, native_fields, len(fields), output))
    errors.raise_for_status(status)
    address = int(_client_ffi.cast("uintptr_t", output[0]))
    handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
    return _wrap_owned_handle(
        wrapper, handle, metadata=metadata, owner_kind=owner_kind, context_raw=context_raw
    )


def _read_enumeration(item: ttlv.TtlvItem, raw_tag: int) -> int:
    return item.field_enumeration(raw_tag)  # type: ignore[attr-defined,no-any-return]


def _read_integer(item: ttlv.TtlvItem, raw_tag: int) -> int:
    return item.field_integer(raw_tag)  # type: ignore[attr-defined,no-any-return]


def _read_bytes(item: ttlv.TtlvItem, raw_tag: int) -> bytes:
    return item.field_bytes(raw_tag)  # type: ignore[attr-defined,no-any-return]


def _has_field(item: ttlv.TtlvItem, raw_tag: int) -> bool:
    return item.has_field(raw_tag)  # type: ignore[attr-defined,no-any-return]


def _read_type(item: ttlv.TtlvItem, raw_tag: int) -> ttlv.ItemType:
    return item.field_type(raw_tag)  # type: ignore[attr-defined,no-any-return]


class ManagedTtlvItem(ttlv.TtlvItem):
    """Typed wrapper around an independently owned generic TTLV Item."""

    __slots__ = ("_metadata", "_owner_kind", "_context_raw")

    def __init__(
        self,
        handle: Any,
        *,
        _token: object,
        metadata: int | None = None,
        owner_kind: int,
        context_raw: int = 0,
    ) -> None:
        super().__init__(handle, _token=_token)
        self._metadata = metadata
        self._owner_kind = owner_kind
        self._context_raw = context_raw

    def _metadata_value(self) -> int:
        self._pointer()
        metadata = self._metadata
        if metadata is None:
            errors.raise_for_status(4)
        return metadata

    def has_field(self, raw_tag: int) -> bool:
        """Report whether a direct child with this raw tag is present."""
        return self._native_field_read(raw_tag, 1)[0]

    @synchronized_native_call
    def field_item(self, raw_tag: int) -> ttlv.TtlvItem | None:
        """Return an independently owned raw Item for a direct child, if present."""
        self._pointer()
        output = _client_ffi.new("kmipkit_ttlv_item_t **")
        present = _client_ffi.new("uint8_t *")
        status = int(
            _client_lib.kmipkit_managed_value_field_item_read(
                self._owner_kind,
                self._context_raw,
                extensions._client_native_item(self),
                raw_tag,
                present,
                output,
            )
        )
        errors.raise_for_status(status)
        if not present[0]:
            return None
        if output[0] == _client_ffi.NULL:
            errors.raise_for_status(4)
        address = int(_client_ffi.cast("uintptr_t", output[0]))
        handle = ttlv.ffi.cast("kmipkit_ttlv_item_t *", address)
        return _wrap_owned_handle(ttlv.TtlvItem, handle)

    def _native_field_read(
        self,
        raw_tag: int,
        read_kind: int,
        output: Any = None,
        byte_capacity: int = 0,
    ) -> tuple[bool, ttlv.ItemType | None, int, int, int]:
        self._pointer()
        item_handle = extensions._client_native_item(self)
        present = _client_ffi.new("uint8_t *")
        item_type = _client_ffi.new("uint8_t *")
        enumeration = _client_ffi.new("uint32_t *")
        integer = _client_ffi.new("int32_t *")
        byte_length = _client_ffi.new("uint64_t *")
        output_bytes = _client_ffi.NULL if output is None else output
        status = int(
            _client_lib.kmipkit_managed_value_field_read(
                self._owner_kind,
                self._context_raw,
                item_handle,
                raw_tag,
                read_kind,
                present,
                item_type,
                enumeration,
                integer,
                output_bytes,
                byte_capacity,
                byte_length,
            )
        )
        errors.raise_for_status(status)
        is_present = bool(present[0])
        return (
            is_present,
            ttlv.ItemType(item_type[0]) if is_present else None,
            int(enumeration[0]),
            int(integer[0]),
            int(byte_length[0]),
        )

    def field_type(self, raw_tag: int) -> ttlv.ItemType:
        """Return the first direct child's TTLV type for this raw tag."""
        _, item_type, _, _, _ = self._native_field_read(raw_tag, 2)
        if item_type is None:
            errors.raise_for_status(4)
        return item_type

    def field_enumeration(self, raw_tag: int) -> int:
        """Read a direct child's raw Enumeration value."""
        return self._native_field_read(raw_tag, 3)[2]

    def field_integer(self, raw_tag: int) -> int:
        """Read a direct child's Integer value."""
        return self._native_field_read(raw_tag, 4)[3]

    def field_bytes(self, raw_tag: int) -> bytes:
        """Copy a direct Byte String or Big Integer field into Python-owned bytes."""
        length = self._native_field_read(raw_tag, 5)[4]
        if length > 0x7FFF_FFFF:
            errors.raise_for_status(6)
        output = _client_ffi.new("uint8_t[]", length) if length else _client_ffi.NULL
        copied_length = self._native_field_read(raw_tag, 5, output, length)[4]
        if copied_length != length:
            errors.raise_for_status(4)
        return bytes(_client_ffi.buffer(output, length)) if length else b""

    def field_read(self, raw_tag: int, read_kind: int) -> bool | ttlv.ItemType | int | bytes:
        """Read a direct child by tag: 1=presence, 2=type, 3=Enumeration, 4=Integer, 5=bytes."""
        if read_kind == 1:
            return self.has_field(raw_tag)
        if read_kind == 2:
            return self.field_type(raw_tag)
        if read_kind == 3:
            return self.field_enumeration(raw_tag)
        if read_kind == 4:
            return self.field_integer(raw_tag)
        if read_kind == 5:
            return self.field_bytes(raw_tag)
        errors.raise_for_status(4)

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
            owner_kind=1,
            context_raw=object_type,
        )  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def from_fields(cls, object_type: int, fields: Sequence[ttlv.TtlvItem]) -> Self:
        """Copy fields into a typed managed object in KMIP table order."""
        if not isinstance(object_type, int) or isinstance(object_type, bool) or not 0 <= object_type <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_fields(
            cls, _client_lib.kmipkit_managed_object_from_fields, fields,
            object_type, metadata=object_type, owner_kind=1, context_raw=object_type,
        )  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def certificate(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_certificate_from_fields, fields, metadata=1, owner_kind=1, context_raw=1)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def symmetric_key(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_symmetric_key_from_fields, fields, metadata=2, owner_kind=1, context_raw=2)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def public_key(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_public_key_from_fields, fields, metadata=3, owner_kind=1, context_raw=3)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def private_key(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_private_key_from_fields, fields, metadata=4, owner_kind=1, context_raw=4)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def split_key(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_split_key_from_fields, fields, metadata=5, owner_kind=1, context_raw=5)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def secret_data(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_secret_data_from_fields, fields, metadata=7, owner_kind=1, context_raw=7)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def opaque_object(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_opaque_object_from_fields, fields, metadata=8, owner_kind=1, context_raw=8)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def pgp_key(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_pgp_key_from_fields, fields, metadata=9, owner_kind=1, context_raw=9)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def certificate_request(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_certificate_request_from_fields, fields, metadata=10, owner_kind=1, context_raw=10)  # type: ignore[return-value]

    @property
    def object_type(self) -> int:
        return self._metadata_value()

    @property
    def certificate_type(self) -> int:
        return _read_enumeration(self, 0x0042_001D)

    @property
    def certificate_value(self) -> bytes:
        return _read_bytes(self, 0x0042_001E)

    @property
    def certificate_request_type(self) -> int:
        return _read_enumeration(self, 0x0042_0019)

    @property
    def certificate_request_value(self) -> bytes:
        return _read_bytes(self, 0x0042_0140)

    @property
    def opaque_data_type(self) -> int:
        return _read_enumeration(self, 0x0042_0059)

    @property
    def opaque_data_value(self) -> bytes:
        return _read_bytes(self, 0x0042_005A)

    @property
    def pgp_key_version(self) -> int:
        return _read_integer(self, 0x0042_00BE)

    @property
    def secret_data_type(self) -> int:
        return _read_enumeration(self, 0x0042_0086)

    @property
    def key_part_identifier(self) -> int:
        return _read_integer(self, 0x0042_0044)

    @property
    def split_key_method(self) -> int:
        return _read_enumeration(self, 0x0042_008A)

    @property
    def split_key_parts(self) -> int:
        return _read_integer(self, 0x0042_008B)

    @property
    def split_key_threshold(self) -> int:
        return _read_integer(self, 0x0042_008C)

    @property
    def has_key_block(self) -> bool:
        return self.has_field(0x0042_0040)

    @property
    def prime_field_size(self) -> bytes:
        return self.field_bytes(0x0042_0062)


class KeyBlock(ManagedTtlvItem):
    """Validated KMIP Key Block retaining child order and unknown Items."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, item: ttlv.TtlvItem) -> Self:
        return _from_item(cls, _client_lib.kmipkit_key_block_from_item, item, owner_kind=2)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def from_fields(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        """Copy caller-owned Key Block fields in KMIP table order."""
        return _from_fields(cls, _client_lib.kmipkit_key_block_from_fields, fields, owner_kind=2)  # type: ignore[return-value]

    @property
    def key_format_type(self) -> int:
        return self.field_enumeration(0x0042_0042)

    @property
    def key_compression_type(self) -> int:
        return self.field_enumeration(0x0042_0041)

    @property
    def key_value_type(self) -> ttlv.ItemType:
        return self.field_type(0x0042_0045)

    def key_value_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw Key Value node, retaining wrapped values as ByteString."""
        return self.field_item(0x0042_0045)

    @property
    def cryptographic_algorithm(self) -> int:
        return self.field_enumeration(0x0042_0028)

    @property
    def cryptographic_length(self) -> int:
        return self.field_integer(0x0042_002A)

    @property
    def has_key_wrapping_data(self) -> bool:
        return self.has_field(0x0042_0046)

    def key_wrapping_data_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw Key Wrapping Data Structure when present."""
        return self.field_item(0x0042_0046)


class KeyValue(ManagedTtlvItem):
    """Validated plaintext KMIP Key Value for one explicit key format."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, format_raw: int, item: ttlv.TtlvItem) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_item(cls, _client_lib.kmipkit_key_value_from_item, item, format_raw, metadata=format_raw, owner_kind=3, context_raw=format_raw)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def from_fields(cls, format_raw: int, fields: Sequence[ttlv.TtlvItem]) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_fields(cls, _client_lib.kmipkit_key_value_from_fields, fields, format_raw, metadata=format_raw, owner_kind=3, context_raw=format_raw)  # type: ignore[return-value]

    @property
    def format_raw(self) -> int:
        return self._metadata_value()

    @property
    def key_material_type(self) -> ttlv.ItemType:
        return self.field_type(0x0042_0043)

    def key_material_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw Key Material node when present."""
        return self.field_item(0x0042_0043)

    @property
    def has_attributes(self) -> bool:
        return self.has_field(0x0042_0125)

    def attributes_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw Attributes Structure when present."""
        return self.field_item(0x0042_0125)


class KeyMaterial(ManagedTtlvItem):
    """Validated KMIP Key Material with caller-selected open Key Format Type."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, format_raw: int, item: ttlv.TtlvItem) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_item(cls, _client_lib.kmipkit_key_material_from_item, item, format_raw, metadata=format_raw, owner_kind=4, context_raw=format_raw)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def from_fields(cls, format_raw: int, fields: Sequence[ttlv.TtlvItem]) -> Self:
        if not isinstance(format_raw, int) or isinstance(format_raw, bool) or not 0 <= format_raw <= 0xFFFF_FFFF:
            errors.raise_for_status(4)
        return _from_fields(cls, _client_lib.kmipkit_key_material_from_fields, fields, format_raw, metadata=format_raw, owner_kind=4, context_raw=format_raw)  # type: ignore[return-value]

    @property
    def format_raw(self) -> int:
        return self._metadata_value()

    @property
    def modulus(self) -> bytes:
        return _read_bytes(self, 0x0042_0052)

    @property
    def public_exponent(self) -> bytes:
        return _read_bytes(self, 0x0042_006C)

    @property
    def key_byte_string(self) -> bytes:
        return self.field_bytes(0x0042_003F)

    @property
    def p(self) -> bytes:
        return self.field_bytes(0x0042_005E)

    @property
    def q(self) -> bytes:
        return self.field_bytes(0x0042_0071)

    @property
    def g(self) -> bytes:
        return self.field_bytes(0x0042_0037)

    @property
    def x(self) -> bytes:
        return self.field_bytes(0x0042_009F)

    @property
    def y(self) -> bytes:
        return self.field_bytes(0x0042_00A0)

    @property
    def j(self) -> bytes:
        return self.field_bytes(0x0042_003E)

    @property
    def d(self) -> bytes:
        return self.field_bytes(0x0042_002E)

    @property
    def q_string(self) -> bytes:
        return self.field_bytes(0x0042_0072)

    @property
    def recommended_curve(self) -> int:
        return self.field_enumeration(0x0042_0075)

    @property
    def private_exponent(self) -> bytes:
        return self.field_bytes(0x0042_0063)

    @property
    def prime_exponent_p(self) -> bytes:
        return self.field_bytes(0x0042_0060)

    @property
    def prime_exponent_q(self) -> bytes:
        return self.field_bytes(0x0042_0061)

    @property
    def crt_coefficient(self) -> bytes:
        return self.field_bytes(0x0042_0027)


class KeyWrappingData(ManagedTtlvItem):
    """Validated KMIP Key Wrapping Data with lossless nested TTLV access."""

    @classmethod
    @synchronized_native_call
    def from_item(cls, item: ttlv.TtlvItem) -> Self:
        return _from_item(cls, _client_lib.kmipkit_key_wrapping_data_from_item, item, owner_kind=5)  # type: ignore[return-value]

    @classmethod
    @synchronized_native_call
    def from_fields(cls, fields: Sequence[ttlv.TtlvItem]) -> Self:
        return _from_fields(cls, _client_lib.kmipkit_key_wrapping_data_from_fields, fields, owner_kind=5)  # type: ignore[return-value]

    @property
    def wrapping_method(self) -> int:
        return _read_enumeration(self, 0x0042_009E)

    @property
    def has_encryption_key_information(self) -> bool:
        return self.has_field(0x0042_0036)

    def encryption_key_information_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw Encryption Key Information Structure when present."""
        return self.field_item(0x0042_0036)

    @property
    def has_mac_signature_key_information(self) -> bool:
        return self.has_field(0x0042_004E)

    def mac_signature_key_information_item(self) -> ttlv.TtlvItem | None:
        """Return an owned raw MAC/Signature Key Information Structure when present."""
        return self.field_item(0x0042_004E)

    @property
    def mac_signature(self) -> bytes:
        return self.field_bytes(0x0042_004D)

    @property
    def iv_counter_nonce(self) -> bytes:
        return self.field_bytes(0x0042_003D)

    @property
    def encoding_option(self) -> int:
        return self.field_enumeration(0x0042_00A3)


__all__ = ["KeyBlock", "KeyMaterial", "KeyValue", "KeyWrappingData", "ManagedObject"]
