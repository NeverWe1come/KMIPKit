"""KMIPKit Python 3.12 bindings.

KMIPKit overwrites its temporary CFFI staging arrays after synchronous native
calls and never overwrites caller-owned input. Immutable Python ``bytes`` and
``str`` values and other runtime copies cannot be deterministically zeroized.
"""

from . import client, errors, extensions, metadata, ttlv
from .extensions import with_extension

__all__ = ["client", "errors", "extensions", "metadata", "ttlv", "with_extension"]
