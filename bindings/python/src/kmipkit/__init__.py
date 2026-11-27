"""KMIPKit Python 3.12 bindings.

Sensitive operation-input builders and the variable TTLV BigInteger,
TextString, and ByteString constructors overwrite the CFFI staging arrays they
own after their synchronous native calls. These paths do not overwrite
caller-owned input. Immutable Python ``bytes`` and ``str`` values and other
runtime copies cannot be deterministically zeroized.
"""

from . import (
    async_operations,
    attribute_operations,
    client,
    credentials,
    cryptographic_parameters,
    errors,
    extensions,
    fluent,
    high_level,
    messages,
    managed_objects,
    metadata,
    profile_catalog,
    protocol,
    protocol_values,
    role_reversal,
    server_to_client,
    ttlv,
    typed_attributes,
    validation,
)
from .extensions import with_extension
from .credentials import CredentialType, CredentialValue
from .errors import ProfileValidationDetails, ProfileValidationError
from .typed_attributes import StandardAttribute, TypedAttribute
from .high_level import (
    CipherSpec,
    HashAlgorithm,
    KdfSpec,
    KemSpec,
    KeyRef,
    KeySpec,
    MacSpec,
    SignatureSpec,
    install_client_namespaces as _install_client_namespaces,
)

_install_client_namespaces(client.Client)

__all__ = [
    "client",
    "async_operations",
    "attribute_operations",
    "credentials",
    "cryptographic_parameters",
    "CredentialType",
    "CredentialValue",
    "ProfileValidationDetails",
    "ProfileValidationError",
    "errors",
    "extensions",
    "fluent",
    "high_level",
    "messages",
    "managed_objects",
    "metadata",
    "profile_catalog",
    "protocol",
    "protocol_values",
    "role_reversal",
    "server_to_client",
    "ttlv",
    "typed_attributes",
    "validation",
    "CipherSpec",
    "HashAlgorithm",
    "KdfSpec",
    "KemSpec",
    "KeyRef",
    "KeySpec",
    "MacSpec",
    "SignatureSpec",
    "StandardAttribute",
    "TypedAttribute",
    "with_extension",
]
