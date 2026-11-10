"""Shared immutable representation of a KMIP protocol version."""

from __future__ import annotations

from dataclasses import dataclass

_I32_MIN = -(2**31)
_I32_MAX = 2**31 - 1


@dataclass(frozen=True, slots=True)
class ProtocolVersion:
    """A KMIP protocol version pair using raw signed TTLV Integer values."""

    major: int
    minor: int

    def __post_init__(self) -> None:
        for name, value in (("major", self.major), ("minor", self.minor)):
            if isinstance(value, bool) or not isinstance(value, int):
                raise TypeError(f"{name} must be an integer")
            if not _I32_MIN <= value <= _I32_MAX:
                raise ValueError(f"{name} must fit a signed 32-bit integer")
