"""KMIPKit Python 3.12 bindings.

Python strings, input buffers, and byte values returned from the native model
are Python-owned copies. KMIPKit zeroizes buffers it owns, but Python runtime
copies are subject to Python's memory-management limitations.
"""

from . import errors, extensions, metadata, ttlv
from .extensions import with_extension

__all__ = ["errors", "extensions", "metadata", "ttlv", "with_extension"]
