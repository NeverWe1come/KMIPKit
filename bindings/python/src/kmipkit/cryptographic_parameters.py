"""Typed KMIP 2.1 Cryptographic Parameters built and validated by Rust."""

from __future__ import annotations

from typing import Self

from . import errors, ttlv
from ._client_ffi import ffi, lib
from ._handles import (
    NativeHandle,
    _integer_range,
    _wrap_owned_handle,
    synchronized_native_call,
)

_U32_MAX = 0xFFFF_FFFF
_I32_MIN = -(1 << 31)
_I32_MAX = (1 << 31) - 1
_MAX_P_SOURCE_BYTES = 16 * 1024 * 1024
_ENUM_FIELDS = (
    "block_cipher_mode",
    "padding_method",
    "hashing_algorithm",
    "key_role_type",
    "digital_signature_algorithm",
    "cryptographic_algorithm",
    "mask_generator",
    "mask_generator_hashing_algorithm",
)
_INTEGER_FIELDS = (
    "iv_length",
    "tag_length",
    "fixed_field_length",
    "invocation_field_length",
    "counter_length",
    "initial_counter_value",
    "salt_length",
    "trailer_field",
)


class CryptographicParameters(NativeHandle):
    """Owned typed parameter model; raw enum values remain forward-compatible."""

    _release_name = "kmipkit_cryptographic_parameters_release"

    @classmethod
    def builder(cls) -> CryptographicParametersBuilder:
        """Create a builder whose Rust implementation emits Table 59 order."""
        return CryptographicParametersBuilder(cls)

    @classmethod
    @synchronized_native_call
    def from_ttlv(cls, structure: ttlv.Structure) -> Self:
        """Copy and validate a generic TTLV Structure using the Rust schema."""
        if not isinstance(structure, ttlv.Structure):
            errors.raise_for_status(4)
        output = ffi.new("kmipkit_cryptographic_parameters_t **")
        address = int(ttlv.ffi.cast("uintptr_t", structure._pointer()))
        source = ffi.cast("kmipkit_ttlv_structure_t *", address)
        errors.raise_for_status(lib.kmipkit_cryptographic_parameters_from_ttlv(source, output))
        address = int(ffi.cast("uintptr_t", output[0]))
        handle = ttlv.ffi.cast("kmipkit_cryptographic_parameters_t *", address)
        return _wrap_owned_handle(cls, handle)

    @synchronized_native_call
    def into_ttlv(self) -> ttlv.Structure:
        """Consume the typed model into an owned generic TTLV Structure."""
        output = ffi.new("kmipkit_ttlv_structure_t **")
        handle, owner = self._take()
        address = int(ttlv.ffi.cast("uintptr_t", handle))
        client_handle = ffi.cast("kmipkit_cryptographic_parameters_t *", address)
        status = int(lib.kmipkit_cryptographic_parameters_into_ttlv(client_handle, output))
        del owner
        errors.raise_for_status(status)
        address = int(ffi.cast("uintptr_t", output[0]))
        return ttlv._structure_from_native_address(address)

    def block_cipher_mode(self) -> int | None:
        """Return the raw Block Cipher Mode Enumeration, including future values."""
        return self._enumeration(0)

    def padding_method(self) -> int | None:
        """Return the raw Padding Method Enumeration, including future values."""
        return self._enumeration(1)

    def hashing_algorithm(self) -> int | None:
        """Return the raw Hashing Algorithm Enumeration, including future values."""
        return self._enumeration(2)

    def key_role_type(self) -> int | None:
        """Return the raw Key Role Type Enumeration, including future values."""
        return self._enumeration(3)

    def digital_signature_algorithm(self) -> int | None:
        """Return the raw Digital Signature Algorithm Enumeration, including future values."""
        return self._enumeration(4)

    def cryptographic_algorithm(self) -> int | None:
        """Return the raw Cryptographic Algorithm Enumeration, including future values."""
        return self._enumeration(5)

    def random_iv(self) -> bool | None:
        """Return Random IV when present."""
        present, values, _p_source = self._read_fields()
        return bool(values[6]) if present[6] else None

    def iv_length(self) -> int | None:
        """Return IV Length when present."""
        return self._integer(7)

    def tag_length(self) -> int | None:
        """Return Tag Length when present."""
        return self._integer(8)

    def fixed_field_length(self) -> int | None:
        """Return Fixed Field Length when present."""
        return self._integer(9)

    def invocation_field_length(self) -> int | None:
        """Return Invocation Field Length when present."""
        return self._integer(10)

    def counter_length(self) -> int | None:
        """Return Counter Length when present."""
        return self._integer(11)

    def initial_counter_value(self) -> int | None:
        """Return Initial Counter Value when present."""
        return self._integer(12)

    def salt_length(self) -> int | None:
        """Return Salt Length when present."""
        return self._integer(13)

    def mask_generator(self) -> int | None:
        """Return the raw Mask Generator Enumeration, including future values."""
        return self._enumeration(14)

    def mask_generator_hashing_algorithm(self) -> int | None:
        """Return the raw Mask Generator Hashing Algorithm Enumeration."""
        return self._enumeration(15)

    def p_source(self) -> bytes | None:
        """Return a caller-owned P Source copy when present.

        The returned Python bytes object is outside KMIPKit's zeroization guarantee.
        """
        present, _values, p_source = self._read_fields()
        return p_source if present[16] else None

    def trailer_field(self) -> int | None:
        """Return Trailer Field when present."""
        return self._integer(17)

    def _enumeration(self, index: int) -> int | None:
        present, values, _p_source = self._read_fields()
        return values[index] if present[index] else None

    def _integer(self, index: int) -> int | None:
        present, values, _p_source = self._read_fields()
        return values[index] if present[index] else None

    @synchronized_native_call
    def _read_fields(self) -> tuple[tuple[bool, ...], tuple[int, ...], bytes]:
        fields = ffi.new("kmipkit_cryptographic_parameters_fields_v1_t *")
        fields.struct_size = ffi.sizeof("kmipkit_cryptographic_parameters_fields_v1_t")
        fields.abi_version = 1
        address = int(ttlv.ffi.cast("uintptr_t", self._pointer()))
        client_handle = ffi.cast("kmipkit_cryptographic_parameters_t *", address)
        errors.raise_for_status(
            lib.kmipkit_cryptographic_parameters_read_fields_v1(client_handle, fields)
        )
        present = (
            bool(fields.block_cipher_mode_present),
            bool(fields.padding_method_present),
            bool(fields.hashing_algorithm_present),
            bool(fields.key_role_type_present),
            bool(fields.digital_signature_algorithm_present),
            bool(fields.cryptographic_algorithm_present),
            bool(fields.random_iv_present),
            bool(fields.iv_length_present),
            bool(fields.tag_length_present),
            bool(fields.fixed_field_length_present),
            bool(fields.invocation_field_length_present),
            bool(fields.counter_length_present),
            bool(fields.initial_counter_value_present),
            bool(fields.salt_length_present),
            bool(fields.mask_generator_present),
            bool(fields.mask_generator_hashing_algorithm_present),
            bool(fields.p_source_present),
            bool(fields.trailer_field_present),
        )
        values = (
            int(fields.block_cipher_mode),
            int(fields.padding_method),
            int(fields.hashing_algorithm),
            int(fields.key_role_type),
            int(fields.digital_signature_algorithm),
            int(fields.cryptographic_algorithm),
            int(fields.random_iv),
            int(fields.iv_length),
            int(fields.tag_length),
            int(fields.fixed_field_length),
            int(fields.invocation_field_length),
            int(fields.counter_length),
            int(fields.initial_counter_value),
            int(fields.salt_length),
            int(fields.mask_generator),
            int(fields.mask_generator_hashing_algorithm),
            int(fields.p_source_length),
            int(fields.trailer_field),
        )
        p_source = (
            bytes(ffi.buffer(fields.p_source, int(fields.p_source_length)))
            if fields.p_source_present
            else b""
        )
        return present, values, p_source


class CryptographicParametersBuilder:
    """Fluent field input whose build step delegates schema rules to Rust."""

    __slots__ = ("_values", "_closed", "_model_type")

    def __init__(self, model_type: type[CryptographicParameters]) -> None:
        self._model_type = model_type
        self._values: dict[str, int | bool | bytearray] = {}
        self._closed = False

    def block_cipher_mode(self, raw: int) -> Self:
        return self._enum("block_cipher_mode", raw)

    def padding_method(self, raw: int) -> Self:
        return self._enum("padding_method", raw)

    def hashing_algorithm(self, raw: int) -> Self:
        return self._enum("hashing_algorithm", raw)

    def key_role_type(self, raw: int) -> Self:
        return self._enum("key_role_type", raw)

    def digital_signature_algorithm(self, raw: int) -> Self:
        return self._enum("digital_signature_algorithm", raw)

    def cryptographic_algorithm(self, raw: int) -> Self:
        return self._enum("cryptographic_algorithm", raw)

    def random_iv(self, value: bool) -> Self:
        self._ensure_open()
        if not isinstance(value, bool):
            errors.raise_for_status(4)
        self._values["random_iv"] = value
        return self

    def iv_length(self, value: int) -> Self:
        return self._integer("iv_length", value)

    def tag_length(self, value: int) -> Self:
        return self._integer("tag_length", value)

    def fixed_field_length(self, value: int) -> Self:
        return self._integer("fixed_field_length", value)

    def invocation_field_length(self, value: int) -> Self:
        return self._integer("invocation_field_length", value)

    def counter_length(self, value: int) -> Self:
        return self._integer("counter_length", value)

    def initial_counter_value(self, value: int) -> Self:
        return self._integer("initial_counter_value", value)

    def salt_length(self, value: int) -> Self:
        return self._integer("salt_length", value)

    def mask_generator(self, raw: int) -> Self:
        return self._enum("mask_generator", raw)

    def mask_generator_hashing_algorithm(self, raw: int) -> Self:
        return self._enum("mask_generator_hashing_algorithm", raw)

    def p_source(self, value: bytes | bytearray | memoryview) -> Self:
        self._ensure_open()
        if not isinstance(value, (bytes, bytearray, memoryview)):
            errors.raise_for_status(4)
        length = value.nbytes if isinstance(value, memoryview) else len(value)
        if length > _MAX_P_SOURCE_BYTES:
            errors.raise_for_status(6)
        previous = self._values.get("p_source")
        if isinstance(previous, bytearray):
            previous[:] = b"\x00" * len(previous)
        self._values["p_source"] = bytearray(value)
        return self

    def trailer_field(self, value: int) -> Self:
        return self._integer("trailer_field", value)

    def build(self) -> CryptographicParameters:
        """Build, validate, and return the typed value through the Rust ABI."""
        self._ensure_open()
        fields = ffi.new("kmipkit_cryptographic_parameters_fields_v1_t *")
        fields.struct_size = ffi.sizeof("kmipkit_cryptographic_parameters_fields_v1_t")
        fields.abi_version = 1
        for name in _ENUM_FIELDS:
            present_name = f"{name}_present"
            setattr(fields, present_name, int(name in self._values))
            if name in self._values:
                setattr(fields, name, int(self._values[name]))
        random_present = "random_iv" in self._values
        fields.random_iv_present = int(random_present)
        fields.random_iv = int(bool(self._values.get("random_iv", False)))
        for name in _INTEGER_FIELDS:
            present_name = f"{name}_present"
            setattr(fields, present_name, int(name in self._values))
            if name in self._values:
                setattr(fields, name, int(self._values[name]))
        stored_source = self._values.get("p_source")
        source = stored_source if isinstance(stored_source, bytearray) else bytearray()
        fields.p_source_present = int("p_source" in self._values)
        fields.p_source_length = len(source)
        fields.p_source = ffi.from_buffer("const uint8_t[]", source) if source else ffi.NULL
        output = ffi.new("kmipkit_cryptographic_parameters_t **")
        try:
            status = int(lib.kmipkit_cryptographic_parameters_from_fields_v1(fields, output))
        finally:
            if source:
                source[:] = b"\x00" * len(source)
            self.close()
        errors.raise_for_status(status)
        address = int(ffi.cast("uintptr_t", output[0]))
        handle = ttlv.ffi.cast("kmipkit_cryptographic_parameters_t *", address)
        return _wrap_owned_handle(self._model_type, handle)

    def close(self) -> None:
        self._closed = True
        for value in self._values.values():
            if isinstance(value, bytearray):
                value[:] = b"\x00" * len(value)
        self._values.clear()

    def _enum(self, name: str, raw: int) -> Self:
        self._ensure_open()
        self._values[name] = _integer_range(raw, 0, _U32_MAX)
        return self

    def _integer(self, name: str, value: int) -> Self:
        self._ensure_open()
        self._values[name] = _integer_range(value, _I32_MIN, _I32_MAX)
        return self

    def _ensure_open(self) -> None:
        if self._closed:
            errors.raise_for_status(4)

    def __enter__(self) -> Self:
        self._ensure_open()
        return self

    def __exit__(self, exc_type: object, exc_value: object, traceback: object) -> None:
        self.close()


__all__ = ["CryptographicParameters", "CryptographicParametersBuilder"]
