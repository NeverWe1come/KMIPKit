"""High-level, "no KMIP knowledge needed" API (Fase 3).

Curated presets (``KeySpec``, ``CipherSpec``, ``SignatureSpec``, ``MacSpec``,
``HashAlgorithm``, ``KemSpec``, ``KdfSpec``), domain namespaces on ``Client``
(``keys``, ``crypto``, ``certificates``, ``secrets``, ``attributes``,
``sessions``, ``admin``, ``async_ops``) and the object-scoped ``KeyRef``
(``client.key(id)``).

Presets only build ``CryptographicParameters`` through the existing Rust-backed
builder, so every value is still validated natively. Numeric values below match
``named_values_generated.rs`` in ``kmipkit-protocol``.
"""

from __future__ import annotations

from dataclasses import dataclass
from enum import Enum
from typing import Any

from . import extensions, fluent
from .cryptographic_parameters import CryptographicParameters

# --- verified KMIP numeric values -------------------------------------------------
_AES, _RSA, _ECDSA = 3, 4, 6
_HMAC_SHA256, _HMAC_SHA384, _HMAC_SHA512 = 9, 0x0A, 0x0B
_CHACHA20, _CHACHA20_POLY1305 = 0x1C, 0x1E
_HMAC_SHA3_256, _HMAC_SHA3_512 = 0x24, 0x26
_ED25519, _ED448 = 0x37, 0x38
_X25519, _X448 = 0x5A, 0x5B
_X25519_MLKEM768, _SECP256R1_MLKEM768 = 0x5C, 0x5D
_ML_KEM = {512: 0x39, 768: 0x3A, 1024: 0x3B}
_ML_DSA = {44: 0x3C, 65: 0x3D, 87: 0x3E}
_SLH_DSA_SHA2_128F = 0x40
_CURVE_P256, _CURVE_P384, _CURVE_P521 = 7, 0x0A, 0x0D
_CURVE_SECP256K1, _CURVE25519, _CURVE448 = 0x19, 0x45, 0x46
_CBC, _CTR, _CMAC, _GCM = 1, 6, 7, 9
_OAEP, _PKCS5, _PKCS1_V1_5, _PSS = 2, 3, 8, 0x0A
_SHA256, _SHA384, _SHA512 = 6, 7, 8
_SHA3_256, _SHA3_384, _SHA3_512 = 0x0F, 0x10, 0x11
_DSA_RSA = {_SHA256: 5, _SHA384: 6, _SHA512: 7}
_DSA_ECDSA = {_SHA256: 0x0E, _SHA384: 0x0F, _SHA512: 0x10}
_MGF1 = 1
_KEM_RSASVE, _KEM_DHKEM, _KEM_MLKEM = 1, 2, 3
_DERIVE_PBKDF2, _DERIVE_HMAC, _DERIVE_NIST_C, _DERIVE_HKDF = 1, 3, 5, 0x0A


class _Spec(Enum):
    """Base class: each member's value is its metadata tuple."""

    def __repr__(self) -> str:
        return f"{type(self).__name__}.{self.name}"


class KeySpec(_Spec):
    """Key presets: ``(algorithm, bits, curve, symmetric, kmip_major)``."""

    AES_128 = (_AES, 128, None, True, 2)
    AES_192 = (_AES, 192, None, True, 2)
    AES_256 = (_AES, 256, None, True, 2)
    HMAC_SHA256 = (_HMAC_SHA256, 256, None, True, 2)
    HMAC_SHA384 = (_HMAC_SHA384, 384, None, True, 2)
    HMAC_SHA512 = (_HMAC_SHA512, 512, None, True, 2)
    CHACHA20 = (_CHACHA20, 256, None, True, 2)
    CHACHA20_POLY1305 = (_CHACHA20_POLY1305, 256, None, True, 2)
    RSA_2048 = (_RSA, 2048, None, False, 2)
    RSA_3072 = (_RSA, 3072, None, False, 2)
    RSA_4096 = (_RSA, 4096, None, False, 2)
    ECDSA_P256 = (_ECDSA, 256, _CURVE_P256, False, 2)
    ECDSA_P384 = (_ECDSA, 384, _CURVE_P384, False, 2)
    ECDSA_P521 = (_ECDSA, 521, _CURVE_P521, False, 2)
    ECDSA_SECP256K1 = (_ECDSA, 256, _CURVE_SECP256K1, False, 2)
    ED25519 = (_ED25519, 256, _CURVE25519, False, 2)
    ED448 = (_ED448, 448, _CURVE448, False, 2)
    X25519 = (_X25519, 256, _CURVE25519, False, 3)
    X448 = (_X448, 448, _CURVE448, False, 3)
    ML_KEM_512 = (_ML_KEM[512], 512, None, False, 3)
    ML_KEM_768 = (_ML_KEM[768], 768, None, False, 3)
    ML_KEM_1024 = (_ML_KEM[1024], 1024, None, False, 3)
    ML_DSA_44 = (_ML_DSA[44], 1312 * 8, None, False, 3)
    ML_DSA_65 = (_ML_DSA[65], 1952 * 8, None, False, 3)
    ML_DSA_87 = (_ML_DSA[87], 2592 * 8, None, False, 3)
    SLH_DSA_SHA2_128F = (_SLH_DSA_SHA2_128F, 128, None, False, 3)
    X25519_MLKEM768 = (_X25519_MLKEM768, 768, None, False, 3)
    SECP256R1_MLKEM768 = (_SECP256R1_MLKEM768, 768, _CURVE_P256, False, 3)

    @property
    def algorithm(self) -> int:
        return self.value[0]

    @property
    def length(self) -> int:
        return self.value[1]

    @property
    def recommended_curve(self) -> int | None:
        return self.value[2]

    @property
    def is_symmetric(self) -> bool:
        return self.value[3]

    @property
    def is_asymmetric(self) -> bool:
        return not self.value[3]

    @property
    def min_kmip_major(self) -> int:
        return self.value[4]

    @property
    def requires_kmip_30(self) -> bool:
        return self.value[4] == 3


def _params() -> Any:
    return CryptographicParameters.builder()


class CipherSpec(_Spec):
    AES_CBC_PKCS5 = "aes_cbc_pkcs5"
    AES_CBC_NO_PADDING = "aes_cbc_nopad"
    AES_GCM = "aes_gcm"
    AES_CTR = "aes_ctr"
    CHACHA20_POLY1305 = "chacha20_poly1305"
    RSA_OAEP_SHA256 = "rsa_oaep_256"
    RSA_OAEP_SHA384 = "rsa_oaep_384"
    RSA_OAEP_SHA512 = "rsa_oaep_512"
    RSA_PKCS1_V1_5 = "rsa_pkcs1"

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        b = _params()
        if self in (CipherSpec.AES_CBC_PKCS5, CipherSpec.AES_CBC_NO_PADDING):
            b.cryptographic_algorithm(_AES).block_cipher_mode(_CBC)
            if self is CipherSpec.AES_CBC_PKCS5:
                b.padding_method(_PKCS5)
        elif self is CipherSpec.AES_GCM:
            b.cryptographic_algorithm(_AES).block_cipher_mode(_GCM)
            b.random_iv(True).iv_length(96).tag_length(128)
        elif self is CipherSpec.AES_CTR:
            b.cryptographic_algorithm(_AES).block_cipher_mode(_CTR)
            b.random_iv(True).iv_length(128)
        elif self is CipherSpec.CHACHA20_POLY1305:
            b.cryptographic_algorithm(_CHACHA20_POLY1305)
        elif self is CipherSpec.RSA_PKCS1_V1_5:
            b.cryptographic_algorithm(_RSA).padding_method(_PKCS1_V1_5)
        else:
            h = {"rsa_oaep_256": _SHA256, "rsa_oaep_384": _SHA384, "rsa_oaep_512": _SHA512}[self.value]
            b.cryptographic_algorithm(_RSA).padding_method(_OAEP).hashing_algorithm(h)
            b.mask_generator(_MGF1).mask_generator_hashing_algorithm(h)
        return b.build()


class HashAlgorithm(_Spec):
    SHA256 = _SHA256
    SHA384 = _SHA384
    SHA512 = _SHA512
    SHA3_256 = _SHA3_256
    SHA3_384 = _SHA3_384
    SHA3_512 = _SHA3_512

    @property
    def raw(self) -> int:
        return self.value

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        return _params().hashing_algorithm(self.value).build()


class SignatureSpec(_Spec):
    ECDSA_SHA256 = ("ecdsa", _SHA256, 2)
    ECDSA_SHA384 = ("ecdsa", _SHA384, 2)
    ECDSA_SHA512 = ("ecdsa", _SHA512, 2)
    RSA_PSS_SHA256 = ("pss", _SHA256, 2)
    RSA_PSS_SHA384 = ("pss", _SHA384, 2)
    RSA_PSS_SHA512 = ("pss", _SHA512, 2)
    RSA_PKCS1_SHA256 = ("pkcs1", _SHA256, 2)
    RSA_PKCS1_SHA384 = ("pkcs1", _SHA384, 2)
    RSA_PKCS1_SHA512 = ("pkcs1", _SHA512, 2)
    ED25519 = ("ed25519", 0, 2)
    ED448 = ("ed448", 0, 2)
    ML_DSA_44 = ("mldsa44", 0, 3)
    ML_DSA_65 = ("mldsa65", 0, 3)
    ML_DSA_87 = ("mldsa87", 0, 3)
    ML_DSA_65_DETERMINISTIC = ("mldsa65d", 0, 3)
    SLH_DSA_SHA2_128F = ("slh128f", 0, 3)

    @property
    def min_kmip_major(self) -> int:
        return self.value[2]

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        kind, h, _ = self.value
        b = _params()
        if kind == "ecdsa":
            b.cryptographic_algorithm(_ECDSA).digital_signature_algorithm(_DSA_ECDSA[h]).hashing_algorithm(h)
        elif kind == "pss":
            b.cryptographic_algorithm(_RSA).padding_method(_PSS).hashing_algorithm(h)
            b.mask_generator(_MGF1).mask_generator_hashing_algorithm(h)
            b.salt_length({_SHA256: 32, _SHA384: 48, _SHA512: 64}[h])
        elif kind == "pkcs1":
            b.cryptographic_algorithm(_RSA).digital_signature_algorithm(_DSA_RSA[h])
            b.padding_method(_PKCS1_V1_5).hashing_algorithm(h)
        elif kind == "ed25519":
            b.cryptographic_algorithm(_ED25519)
        elif kind == "ed448":
            b.cryptographic_algorithm(_ED448)
        elif kind == "slh128f":
            b.cryptographic_algorithm(_SLH_DSA_SHA2_128F)
        else:
            level = {"mldsa44": 44, "mldsa65": 65, "mldsa65d": 65, "mldsa87": 87}[kind]
            b.cryptographic_algorithm(_ML_DSA[level])
            if kind == "mldsa65d":
                b.deterministic(True)
        return b.build()


class MacSpec(_Spec):
    HMAC_SHA256 = (_HMAC_SHA256, _SHA256)
    HMAC_SHA384 = (_HMAC_SHA384, _SHA384)
    HMAC_SHA512 = (_HMAC_SHA512, _SHA512)
    HMAC_SHA3_256 = (_HMAC_SHA3_256, _SHA3_256)
    HMAC_SHA3_512 = (_HMAC_SHA3_512, _SHA3_512)
    AES_CMAC = (_AES, None)

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        alg, h = self.value
        b = _params().cryptographic_algorithm(alg)
        if h is None:
            b.block_cipher_mode(_CMAC)
        else:
            b.hashing_algorithm(h)
        return b.build()


class KemSpec(_Spec):
    ML_KEM_512 = (_KEM_MLKEM, _ML_KEM[512])
    ML_KEM_768 = (_KEM_MLKEM, _ML_KEM[768])
    ML_KEM_1024 = (_KEM_MLKEM, _ML_KEM[1024])
    DHKEM = (_KEM_DHKEM, None)
    RSASVE = (_KEM_RSASVE, None)
    X25519_MLKEM768 = (_KEM_MLKEM, _X25519_MLKEM768)
    SECP256R1_MLKEM768 = (_KEM_MLKEM, _SECP256R1_MLKEM768)

    @property
    def kem_algorithm(self) -> int:
        return self.value[0]

    @property
    def min_kmip_major(self) -> int:
        return 3

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        kem, alg = self.value
        b = _params().kem_algorithm(kem)
        if alg is not None:
            b.cryptographic_algorithm(alg)
        return b.build()


class KdfSpec(_Spec):
    HKDF_SHA256 = (_DERIVE_HKDF, _SHA256)
    HKDF_SHA384 = (_DERIVE_HKDF, _SHA384)
    HKDF_SHA512 = (_DERIVE_HKDF, _SHA512)
    PBKDF2_SHA256 = (_DERIVE_PBKDF2, _SHA256)
    NIST800_108_COUNTER_HMAC_SHA256 = (_DERIVE_NIST_C, _SHA256)
    HMAC_SHA256 = (_DERIVE_HMAC, _SHA256)

    @property
    def derivation_method(self) -> int:
        return self.value[0]

    @property
    def hash_algorithm(self) -> HashAlgorithm:
        return HashAlgorithm(self.value[1])

    def to_cryptographic_parameters(self) -> CryptographicParameters:
        return _params().hashing_algorithm(self.value[1]).build()


# --- builder integration -----------------------------------------------------------
def _require_symmetric(spec: KeySpec) -> None:
    if not isinstance(spec, KeySpec) or not spec.is_symmetric:
        raise ValueError("a symmetric KeySpec is required for this builder")


def _require_asymmetric(spec: KeySpec) -> None:
    if not isinstance(spec, KeySpec) or not spec.is_asymmetric:
        raise ValueError("an asymmetric KeySpec is required for this builder")


def _sym_with_key_spec(self: Any, spec: KeySpec) -> Any:
    _require_symmetric(spec)
    return self.algorithm(spec.algorithm).length(spec.length)


def _pair_with_key_spec(self: Any, spec: KeySpec) -> Any:
    _require_asymmetric(spec)
    return self.algorithm(spec.algorithm).length(spec.length)


def _make_spec_setter(kind: type, label: str) -> Any:
    def setter(self: Any, spec: Any) -> Any:
        if not isinstance(spec, kind):
            raise TypeError(f"{label} requires a {kind.__name__}")
        params = spec.to_cryptographic_parameters()
        try:
            return self.cryptographic_parameters(params)
        finally:
            params.close()

    setter.__name__ = label
    return setter


def _install() -> None:
    fluent.SymmetricKeyBuilder.with_key_spec = _sym_with_key_spec
    fluent.CreateKeyPairBuilder.with_key_spec = _pair_with_key_spec
    for cls in (fluent.EncryptBuilder, fluent.DecryptBuilder):
        cls.with_cipher_spec = _make_spec_setter(CipherSpec, "with_cipher_spec")
    for cls in (fluent.SignBuilder, fluent.SignatureVerifyBuilder):
        cls.with_signature_spec = _make_spec_setter(SignatureSpec, "with_signature_spec")
    for cls in (fluent.MacBuilder, fluent.MacVerifyBuilder):
        cls.with_mac_spec = _make_spec_setter(MacSpec, "with_mac_spec")
    fluent.HashBuilder.with_hash_algorithm = _make_spec_setter(HashAlgorithm, "with_hash_algorithm")
    for cls in (fluent.EncapsulateBuilder, fluent.DecapsulateBuilder):
        cls.with_kem_spec = _make_spec_setter(KemSpec, "with_kem_spec")


_install()


def _uid(value: str | extensions.UniqueIdentifier) -> extensions.UniqueIdentifier:
    if isinstance(value, extensions.UniqueIdentifier):
        return value
    return extensions.UniqueIdentifier.text_string(value)


# --- domain namespaces -------------------------------------------------------------
class _Namespace:
    __slots__ = ("_request",)

    def __init__(self, client: Any) -> None:
        self._request = fluent.Request(client)

    def raw(self) -> fluent.Request:
        """Escape hatch to the full 64-operation ``Request`` namespace."""
        return self._request


def _forward(name: str, doc: str, target: str | None = None) -> Any:
    target = target or name

    def method(self: Any, *args: Any, **kwargs: Any) -> Any:
        return getattr(self._request, target)(*args, **kwargs)

    method.__name__, method.__doc__ = name, doc
    return method


def _namespace(name: str, doc: str, methods: dict[str, str]) -> type:
    body = {n: _forward(n, f"Begin ``{t}``.", t) for n, t in methods.items()}
    return type(name, (_Namespace,), {"__slots__": (), "__doc__": doc, **body})


KeysClient = _namespace("KeysClient", "Key generation, import/export, derivation, lookup.", {
    "create_symmetric": "create_symmetric_key", "create_key_pair": "create_key_pair",
    "create": "create", "create_split_key": "create_split_key", "join_split_key": "join_split_key",
    "derive": "derive_key", "register": "register", "import_object": "import_object",
    "export": "export", "rekey": "rekey", "rekey_key_pair": "rekey_key_pair",
    "locate": "locate", "get": "get", "activate": "activate", "revoke": "revoke",
    "destroy": "destroy",
})
CryptoClient = _namespace("CryptoClient", "Encrypt/decrypt, sign, MAC, hash, RNG, KEM.", {
    "encrypt": "encrypt", "decrypt": "decrypt", "sign": "sign",
    "signature_verify": "signature_verify", "mac": "mac", "mac_verify": "mac_verify",
    "hash": "hash", "rng_retrieve": "rng_retrieve", "rng_seed": "rng_seed",
    "encapsulate": "encapsulate", "decapsulate": "decapsulate",
})
CertificatesClient = _namespace("CertificatesClient", "Certificate lifecycle and validation.", {
    "certify": "certify", "recertify": "recertify", "validate": "validate",
    "get": "get", "locate": "locate", "revoke": "revoke", "destroy": "destroy",
})
SecretsClient = _namespace("SecretsClient", "Secret data, archive and leases.", {
    "register": "register", "get": "get", "archive": "archive", "recover": "recover",
    "obtain_lease": "obtain_lease", "get_usage_allocation": "get_usage_allocation",
    "destroy": "destroy",
})
AttributesClient = _namespace("AttributesClient", "Object attribute and policy operations.", {
    "get": "get_attributes", "get_list": "get_attribute_list", "add": "add_attribute",
    "set": "set_attribute", "modify": "modify_attribute", "adjust": "adjust_attribute",
    "delete": "delete_attribute", "get_constraints": "get_constraints",
    "set_constraints": "set_constraints", "set_defaults": "set_defaults",
})
SessionsClient = _namespace("SessionsClient", "Sessions, endpoint role, KMIP 3.0 identities.", {
    "login": "login", "logout": "logout", "delegated_login": "delegated_login",
    "set_endpoint_role": "set_endpoint_role", "create_user": "create_user",
    "create_group": "create_group", "create_credential": "create_credential",
})
AdminClient = _namespace("AdminClient", "Diagnostics, audit and KMIP 3.0 decommissioning.", {
    "ping": "ping", "query": "query", "discover_versions": "discover_versions",
    "check": "check", "log": "log", "pkcs11": "pkcs11", "interop": "interop",
    "reprovision": "reprovision", "deactivate": "deactivate", "obliterate": "obliterate",
})
AsyncOpsClient = _namespace("AsyncOpsClient", "Asynchronous operation tracking.", {
    "poll": "poll", "cancel": "cancel", "process": "process", "query": "query_async_requests",
})


@dataclass(frozen=True, slots=True)
class KeyRef:
    """Object-scoped handle: ``client.key(id).encrypt()`` pre-binds the identifier."""

    client: Any
    unique_identifier: str

    @property
    def id(self) -> str:
        return self.unique_identifier

    def _bound(self, op: str) -> Any:
        return getattr(fluent.Request(self.client), op)().unique_identifier(_uid(self.unique_identifier))

    def get(self) -> Any:
        return self._bound("get_request")

    def activate(self) -> Any:
        return self._bound("activate")

    def archive(self) -> Any:
        return self._bound("archive")

    def recover(self) -> Any:
        return self._bound("recover")

    def encrypt(self) -> Any:
        return self._bound("encrypt")

    def decrypt(self) -> Any:
        return self._bound("decrypt")

    def sign(self) -> Any:
        return self._bound("sign")

    def verify(self) -> Any:
        return self._bound("signature_verify")

    signature_verify = verify

    def mac(self) -> Any:
        return self._bound("mac")

    def mac_verify(self) -> Any:
        return self._bound("mac_verify")

    def destroy(self) -> Any:
        return fluent.Request(self.client).destroy(_uid(self.unique_identifier))

    def deactivate(self) -> Any:
        return fluent.Request(self.client).deactivate(_uid(self.unique_identifier))

    def obliterate(self) -> Any:
        return fluent.Request(self.client).obliterate(_uid(self.unique_identifier))

    def encapsulate(self) -> Any:
        return fluent.Request(self.client).encapsulate(_uid(self.unique_identifier))

    def decapsulate(self, ciphertext: bytes) -> Any:
        return fluent.Request(self.client).decapsulate(_uid(self.unique_identifier), ciphertext)


def _namespace_property(namespace_cls: type) -> property:
    def getter(self: Any) -> Any:
        return namespace_cls(self)

    return property(getter)


def _key_ref(self: Any, unique_identifier: str) -> KeyRef:
    """Return an object-scoped handle binding ``unique_identifier``."""
    return KeyRef(self, unique_identifier)


def install_client_namespaces(client_cls: type) -> None:
    """Attach the namespace properties and ``key(id)`` to ``Client``."""
    for prop, cls in (
        ("keys", KeysClient), ("crypto", CryptoClient), ("certificates", CertificatesClient),
        ("secrets", SecretsClient), ("attributes", AttributesClient),
        ("sessions", SessionsClient), ("admin", AdminClient), ("async_ops", AsyncOpsClient),
    ):
        setattr(client_cls, prop, _namespace_property(cls))
    client_cls.key = _key_ref


__all__ = [
    "AdminClient", "AsyncOpsClient", "AttributesClient", "CertificatesClient", "CipherSpec",
    "CryptoClient", "HashAlgorithm", "KdfSpec", "KemSpec", "KeyRef", "KeySpec", "KeysClient",
    "MacSpec", "SecretsClient", "SessionsClient", "SignatureSpec", "install_client_namespaces",
]
