"""Typed Python access to the stable TTLV portion of the KMIPKit C ABI."""

from __future__ import annotations

from enum import IntEnum

from . import errors
from ._ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _invoke,
    _new_handle,
    _scalar,
    synchronize_public_functions,
)


class ItemType(IntEnum):
    """The eleven KMIP 2.1 TTLV Item Types represented by the model."""

    Structure = 1
    Integer = 2
    LongInteger = 3
    BigInteger = 4
    Enumeration = 5
    Boolean = 6
    TextString = 7
    ByteString = 8
    DateTime = 9
    Interval = 10
    DateTimeExtended = 11

    @classmethod
    def _missing_(cls, value: object) -> ItemType | None:
        if (
            not isinstance(value, int)
            or isinstance(value, bool)
            or not 0 <= value <= 0xFF
        ):
            return None
        member = int.__new__(cls, value)
        member._name_ = f"Unknown_0x{value:02X}"
        member._value_ = value
        cls._value2member_map_[value] = member
        return member


class RawTag(NativeHandle):
    _release_name = "kmipkit_raw_tag_release"


class Tag(NativeHandle):
    _release_name = "kmipkit_tag_release"


class CodecLimits(NativeHandle):
    _release_name = "kmipkit_codec_limits_release"


class Value(NativeHandle):
    _release_name = "kmipkit_ttlv_value_release"


class Structure(NativeHandle):
    _release_name = "kmipkit_ttlv_structure_release"


class TtlvItem(NativeHandle):
    _release_name = "kmipkit_ttlv_item_release"


class TtlvStructureView(NativeHandle):
    _release_name = "kmipkit_ttlv_structure_view_release"


class TtlvItemView(NativeHandle):
    _release_name = "kmipkit_ttlv_item_view_release"


class TtlvValueView(NativeHandle):
    _release_name = "kmipkit_ttlv_value_view_release"


def raw_tag_from_raw(raw: int) -> RawTag:
    """Create a raw, not yet validated TTLV tag."""
    handle = _new_handle("kmipkit_raw_tag_t", lib.kmipkit_ttlv_raw_tag_create, raw)
    return RawTag(handle)


def raw_tag_value(raw_tag: RawTag) -> int:
    """Return the numeric contents of a raw tag."""
    return _scalar("uint32_t", lib.kmipkit_ttlv_raw_tag_value, raw_tag._pointer())


def raw_tag_try_checked(raw_tag: RawTag) -> Tag:
    """Validate a raw tag and return its checked tag handle."""
    handle = _new_handle(
        "kmipkit_tag_t", lib.kmipkit_ttlv_raw_tag_try_checked, raw_tag._pointer()
    )
    return Tag(handle)


def tag_value(tag: Tag) -> int:
    """Return the numeric value of a checked TTLV tag."""
    return _scalar("uint32_t", lib.kmipkit_ttlv_tag_value, tag._pointer())


def codec_limits_defaults() -> CodecLimits:
    """Create the default bounded TTLV codec configuration."""
    handle = _new_handle("kmipkit_codec_limits_t", lib.kmipkit_codec_limits_defaults)
    return CodecLimits(handle)


def codec_limits_create(
    max_message_bytes: int, max_structure_depth: int, max_elements: int
) -> CodecLimits:
    """Create custom TTLV codec limits, subject to native hard bounds."""
    handle = _new_handle(
        "kmipkit_codec_limits_t",
        lib.kmipkit_codec_limits_create,
        max_message_bytes,
        max_structure_depth,
        max_elements,
    )
    return CodecLimits(handle)


def codec_limits_max_message_bytes(limits: CodecLimits) -> int:
    return _scalar(
        "uint64_t", lib.kmipkit_codec_limits_max_message_bytes, limits._pointer()
    )


def codec_limits_max_structure_depth(limits: CodecLimits) -> int:
    return _scalar(
        "uint64_t", lib.kmipkit_codec_limits_max_structure_depth, limits._pointer()
    )


def codec_limits_max_elements(limits: CodecLimits) -> int:
    return _scalar("uint64_t", lib.kmipkit_codec_limits_max_elements, limits._pointer())


def _new_value(symbol: str, c_type: str, *args: object) -> Value:
    function = getattr(lib, symbol)
    return Value(_new_handle(c_type, function, *args))


def ttlv_value_integer(value: int) -> Value:
    return _new_value("kmipkit_ttlv_value_integer", "kmipkit_ttlv_value_t", value)


def ttlv_value_long_integer(value: int) -> Value:
    return _new_value("kmipkit_ttlv_value_long_integer", "kmipkit_ttlv_value_t", value)


def ttlv_value_enumeration(value: int) -> Value:
    return _new_value("kmipkit_ttlv_value_enumeration", "kmipkit_ttlv_value_t", value)


def ttlv_value_boolean(value: bool) -> Value:
    if not isinstance(value, bool):
        errors.raise_for_status(4)
    return _new_value("kmipkit_ttlv_value_boolean", "kmipkit_ttlv_value_t", int(value))


def ttlv_value_date_time(value: int) -> Value:
    return _new_value("kmipkit_ttlv_value_date_time", "kmipkit_ttlv_value_t", value)


def ttlv_value_interval(value: int) -> Value:
    return _new_value("kmipkit_ttlv_value_interval", "kmipkit_ttlv_value_t", value)


def ttlv_value_date_time_extended(value: int) -> Value:
    return _new_value(
        "kmipkit_ttlv_value_date_time_extended", "kmipkit_ttlv_value_t", value
    )


def _variable_value(symbol: str, value: bytes, limits: CodecLimits) -> Value:
    if not isinstance(value, (bytes, bytearray, memoryview)):
        errors.raise_for_status(4)
    try:
        view = memoryview(value)
    except (TypeError, ValueError):
        errors.raise_for_status(4)
    if view.nbytes > codec_limits_max_message_bytes(limits):
        errors.raise_for_status(6)
    raw = view.tobytes()
    data = ffi.new("uint8_t[]", raw)
    handle = _new_handle(
        "kmipkit_ttlv_value_t", getattr(lib, symbol), limits._pointer(), data, len(raw)
    )
    return Value(handle)


def ttlv_value_big_integer(value: bytes, limits: CodecLimits) -> Value:
    return _variable_value("kmipkit_ttlv_value_big_integer", value, limits)


def ttlv_value_text_string(value: bytes, limits: CodecLimits) -> Value:
    return _variable_value("kmipkit_ttlv_value_text_string", value, limits)


def ttlv_value_byte_string(value: bytes, limits: CodecLimits) -> Value:
    return _variable_value("kmipkit_ttlv_value_byte_string", value, limits)


def ttlv_value_structure(structure: Structure, limits: CodecLimits) -> Value:
    handle = _new_handle(
        "kmipkit_ttlv_value_t",
        lib.kmipkit_ttlv_value_structure,
        structure,
        limits._pointer(),
        consumed=(0,),
    )
    return Value(handle)


def ttlv_structure_create() -> Structure:
    return Structure(
        _new_handle("kmipkit_ttlv_structure_t", lib.kmipkit_ttlv_structure_create)
    )


def ttlv_item_create(tag: Tag, value: Value, limits: CodecLimits) -> TtlvItem:
    handle = _new_handle(
        "kmipkit_ttlv_item_t",
        lib.kmipkit_ttlv_item_create,
        tag._pointer(),
        value,
        limits._pointer(),
        consumed=(1,),
    )
    return TtlvItem(handle)


def ttlv_structure_with_item(
    structure: Structure, item: TtlvItem, limits: CodecLimits
) -> Structure:
    handle = _new_handle(
        "kmipkit_ttlv_structure_t",
        lib.kmipkit_ttlv_structure_with_item,
        structure,
        item,
        limits._pointer(),
        consumed=(0, 1),
    )
    return Structure(handle)


def ttlv_structure_view(structure: Structure) -> TtlvStructureView:
    handle = _new_handle(
        "kmipkit_ttlv_structure_view_t",
        lib.kmipkit_ttlv_structure_view,
        structure._pointer(),
    )
    return TtlvStructureView(handle, owner=structure)


def ttlv_structure_view_item_count(view: TtlvStructureView) -> int:
    return _scalar(
        "uint64_t", lib.kmipkit_ttlv_structure_view_item_count, view._pointer()
    )


def ttlv_structure_view_item_at(view: TtlvStructureView, index: int) -> TtlvItemView:
    if index < 0:
        errors.raise_for_status(4)
    handle = _new_handle(
        "kmipkit_ttlv_item_view_t",
        lib.kmipkit_ttlv_structure_view_item_at,
        view._pointer(),
        index,
    )
    return TtlvItemView(handle, owner=view)


def ttlv_item_view_tag(view: TtlvItemView) -> Tag:
    handle = _new_handle(
        "kmipkit_tag_t", lib.kmipkit_ttlv_item_view_tag, view._pointer()
    )
    return Tag(handle, owner=view)


def ttlv_item_view_type(view: TtlvItemView) -> ItemType:
    value = _scalar("uint8_t", lib.kmipkit_ttlv_item_view_type, view._pointer())
    return ItemType(value)


def ttlv_item_view_value(view: TtlvItemView) -> TtlvValueView:
    handle = _new_handle(
        "kmipkit_ttlv_value_view_t", lib.kmipkit_ttlv_item_view_value, view._pointer()
    )
    return TtlvValueView(handle, owner=view)


def ttlv_value_view(value: Value) -> TtlvValueView:
    handle = _new_handle(
        "kmipkit_ttlv_value_view_t", lib.kmipkit_ttlv_value_view, value._pointer()
    )
    return TtlvValueView(handle, owner=value)


def ttlv_value_view_type(view: TtlvValueView) -> ItemType:
    value = _scalar("uint8_t", lib.kmipkit_ttlv_value_view_type, view._pointer())
    return ItemType(value)


def ttlv_value_view_structure(view: TtlvValueView) -> TtlvStructureView:
    handle = _new_handle(
        "kmipkit_ttlv_structure_view_t",
        lib.kmipkit_ttlv_value_view_structure,
        view._pointer(),
    )
    return TtlvStructureView(handle, owner=view)


def _value_scalar(symbol: str, c_type: str, view: TtlvValueView) -> int:
    return _scalar(c_type, getattr(lib, symbol), view._pointer())


def ttlv_value_view_integer(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_integer", "int32_t", view)


def ttlv_value_view_long_integer(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_long_integer", "int64_t", view)


def ttlv_value_view_enumeration(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_enumeration", "uint32_t", view)


def ttlv_value_view_boolean(view: TtlvValueView) -> bool:
    return bool(_value_scalar("kmipkit_ttlv_value_view_boolean", "uint8_t", view))


def ttlv_value_view_date_time(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_date_time", "int64_t", view)


def ttlv_value_view_interval(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_interval", "uint32_t", view)


def ttlv_value_view_date_time_extended(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_date_time_extended", "int64_t", view)


def ttlv_value_view_byte_length(view: TtlvValueView) -> int:
    return _value_scalar("kmipkit_ttlv_value_view_byte_length", "uint64_t", view)


def ttlv_value_view_byte_at(view: TtlvValueView, index: int) -> int:
    if index < 0:
        errors.raise_for_status(4)
    output = ffi.new("uint8_t *")
    errors.raise_for_status(
        _invoke(lib.kmipkit_ttlv_value_view_byte_at, view._pointer(), index, output)
    )
    return int(output[0])


__all__ = [
    name
    for name in globals()
    if name.startswith(("ttlv_", "codec_limits_", "raw_tag_", "tag_"))
] + [
    "ItemType",
    "RawTag",
    "Tag",
    "CodecLimits",
    "Value",
    "Structure",
    "TtlvItem",
    "TtlvStructureView",
    "TtlvItemView",
    "TtlvValueView",
]

synchronize_public_functions(globals(), __all__)
