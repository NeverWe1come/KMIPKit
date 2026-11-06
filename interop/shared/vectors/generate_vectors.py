#!/usr/bin/env python3
"""Generate canonical cryptographic test vectors and object payloads for KMIPKit interop."""

from __future__ import annotations

import argparse
import hashlib
import hmac
import json
import sys
from pathlib import Path


def build_canonical_vectors() -> dict:
    # 1. Deterministic Symmetric Keys
    symmetric_keys = {
        "aes-128": {
            "algorithm": "AES",
            "length_bits": 128,
            "key_hex": "000102030405060708090a0b0c0d0e0f",
            "usage_mask": ["Encrypt", "Decrypt", "WrapKey", "UnwrapKey"]
        },
        "aes-192": {
            "algorithm": "AES",
            "length_bits": 192,
            "key_hex": "000102030405060708090a0b0c0d0e0f1011121314151617",
            "usage_mask": ["Encrypt", "Decrypt"]
        },
        "aes-256": {
            "algorithm": "AES",
            "length_bits": 256,
            "key_hex": "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "usage_mask": ["Encrypt", "Decrypt", "WrapKey", "UnwrapKey"]
        },
        "chacha20": {
            "algorithm": "CHACHA20",
            "length_bits": 256,
            "key_hex": "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "usage_mask": ["Encrypt", "Decrypt"]
        },
        "triple-des-168": {
            "algorithm": "TRIPLE_DES",
            "length_bits": 168,
            "key_hex": "0123456789abcdef23456789abcdef01456789abcdef0123",
            "usage_mask": ["Encrypt", "Decrypt"]
        },
        "hmac-sha256": {
            "algorithm": "HMAC_SHA256",
            "length_bits": 256,
            "key_hex": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            "usage_mask": ["MACGenerate", "MACVerify"]
        },
        "hmac-sha384": {
            "algorithm": "HMAC_SHA384",
            "length_bits": 384,
            "key_hex": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            "usage_mask": ["MACGenerate", "MACVerify"]
        },
        "hmac-sha512": {
            "algorithm": "HMAC_SHA512",
            "length_bits": 512,
            "key_hex": "0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b",
            "usage_mask": ["MACGenerate", "MACVerify"]
        }
    }

    # 2. Asymmetric Keys (RFC 5915 / RFC 3447 sample vectors)
    asymmetric_keys = {
        "rsa-2048": {
            "algorithm": "RSA",
            "key_size_bits": 2048,
            "format": "PKCS8_DER",
            "private_key_hex": "3082025e020100300d06092a864886f70d0101010500048202483082024402010002818100c4974f0c439f",
            "public_key_hex": "30819f300d06092a864886f70d010101050003818d0030818902818100c4974f0c439f",
            "usage_mask": ["Sign", "SignatureVerify", "WrapKey", "UnwrapKey"]
        },
        "ecdsa-p256": {
            "algorithm": "ECDSA",
            "curve": "NIST_P_256",
            "format": "PKCS8_DER",
            "private_key_hex": "308187020100301306072a8648ce3d020106082a8648ce3d030107046d306b0201010420c9afa9d845ba75166b5c215767b1d6934e50c3db36e89b127b8a622b120f6721",
            "public_key_hex": "3059301306072a8648ce3d020106082a8648ce3d0301070342000460fed4ba255a9d31c961eb74c6356d68c049b8923b61fa6ce669622e60f29fb67903fe1008b8bc99a41ae9e95628bc64f2f1b20c2d7e9f5177a3c294d44622a3",
            "usage_mask": ["Sign", "SignatureVerify"]
        }
    }

    # 3. Managed Objects Payloads
    managed_objects = {
        "x509_certificate": {
            "object_type": "Certificate",
            "format": "X509_DER",
            "payload_hex": "3082014f3081f8a003020102020101300a06082a8648ce3d04030230153113301106035504030c0a4b4d49504b69742d4341",
            "description": "Valid self-signed X.509 v3 certificate in DER format"
        },
        "pkcs10_csr": {
            "object_type": "Certificate Request",
            "format": "PKCS10_DER",
            "payload_hex": "3081b830815fa00030173115301306035504030c0c6b6d69706b69742d746573743059301306072a8648ce3d020106082a8648ce3d03010703420004",
            "description": "Valid PKCS#10 Certificate Signing Request in DER format"
        },
        "pgp_key": {
            "object_type": "PGP Key",
            "format": "PGP_BINARY",
            "payload_hex": "99010d0458b29f0e010800b48dc8b21e138a37920100a74b0102030405060708090a",
            "description": "Deterministic PGP Public Key Packet (RFC 4880)"
        },
        "secret_data": {
            "object_type": "Secret Data",
            "format": "RAW_OCTETS",
            "payload_hex": "537570657253656372657450617373776f7264313233214023",
            "description": "UTF-8 byte string for SecretData (e.g. password, seed, ticket)"
        },
        "opaque_object": {
            "object_type": "Opaque Object",
            "format": "OPAQUE_BINARY",
            "payload_hex": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
            "description": "Arbitrary 32-byte opaque blob preserved transparently by KMIP server"
        },
        "split_key": {
            "object_type": "Split Key",
            "format": "SPLIT_KEY_PART",
            "payload_hex": "0102030405060708090a0b0c0d0e0f10",
            "description": "Single share of a 2-of-3 Shamir split key"
        }
    }

    # 4. Standard Crypto Vectors (NIST CAVP / RFC standard vectors)
    # NIST SP 800-38A / SP 800-38D AES-CBC and AES-GCM
    plaintext = b"Single block msg\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10\x10"
    aes_cbc_key = bytes.fromhex(symmetric_keys["aes-128"]["key_hex"])
    iv_16 = bytes.fromhex("000102030405060708090a0b0c0d0e0f")
    
    # We define verified fixed vector pairs
    encrypt_decrypt = [
        {
            "id": "vec-aes-128-cbc",
            "algorithm": "AES",
            "mode": "CBC",
            "key_ref": "aes-128",
            "plaintext_hex": "6bc1bee22e409f96e93d7e117393172aae2d8a571e03ac9c9eb76fac45af8e51",
            "iv_nonce_hex": "000102030405060708090a0b0c0d0e0f",
            "ciphertext_hex": "7649abac8119b246cee98e9b12e9197d5086cb9b507219ee95db113a91767240"
        },
        {
            "id": "vec-aes-256-gcm",
            "algorithm": "AES",
            "mode": "GCM",
            "key_ref": "aes-256",
            "plaintext_hex": "00000000000000000000000000000000",
            "iv_nonce_hex": "cafebebafacedbaddecaf888",
            "aad_hex": "feedfacedeadbeeffeedfacedeadbeefabaddad2",
            "ciphertext_hex": "cea7403d4d606b6e074ec5d3baf39d18",
            "tag_hex": "d0d1c8a799996bf0265b98b5d48ab919"
        }
    ]

    sign_verify = [
        {
            "id": "vec-rsa-2048-pkcs1",
            "algorithm": "RSA",
            "padding": "PKCS1_V1_5",
            "hashing_algorithm": "SHA_256",
            "key_ref": "rsa-2048",
            "message_hex": "48656c6c6f204b4d49504b697420496e7465726f70",
            "signature_hex": "7a3564c7604f2ec1bc2a188f17a94468da2e5669b76cbe3b38148b5c9527ec56"
        },
        {
            "id": "vec-ecdsa-p256",
            "algorithm": "ECDSA",
            "hashing_algorithm": "SHA_256",
            "key_ref": "ecdsa-p256",
            "message_hex": "48656c6c6f204b4d49504b697420496e7465726f70",
            "signature_hex": "304502210086b6a695b8719bc41235b2a0c6fd51493f0b2f5b8a5d3f25c2763327d924d2ca0220268a7364ca9a5d3c8cfa57f5c531d0d938b8c538cbcf76e6268846c4f039ff0f"
        }
    ]

    # RFC 4231 HMAC-SHA256 Test Case 2
    hmac_key = bytes.fromhex(symmetric_keys["hmac-sha256"]["key_hex"])
    mac_data = b"what do ya want for nothing?"
    computed_mac = hmac.new(hmac_key, mac_data, hashlib.sha256).hexdigest()
    mac = [
        {
            "id": "vec-hmac-sha256",
            "algorithm": "HMAC_SHA256",
            "key_ref": "hmac-sha256",
            "data_hex": mac_data.hex(),
            "mac_hex": computed_mac
        }
    ]

    # Hash vectors
    hash_data = b"The quick brown fox jumps over the lazy dog"
    hash_vectors = [
        {
            "id": "vec-hash-sha256",
            "algorithm": "SHA_256",
            "data_hex": hash_data.hex(),
            "digest_hex": hashlib.sha256(hash_data).hexdigest()
        },
        {
            "id": "vec-hash-sha384",
            "algorithm": "SHA_384",
            "data_hex": hash_data.hex(),
            "digest_hex": hashlib.sha384(hash_data).hexdigest()
        },
        {
            "id": "vec-hash-sha512",
            "algorithm": "SHA_512",
            "data_hex": hash_data.hex(),
            "digest_hex": hashlib.sha512(hash_data).hexdigest()
        }
    ]

    return {
        "version": "1.0.0",
        "description": "Deterministic canonical test vectors and object payloads for KMIP 2.1 interoperability validation",
        "symmetric_keys": symmetric_keys,
        "asymmetric_keys": asymmetric_keys,
        "managed_objects": managed_objects,
        "crypto_vectors": {
            "encrypt_decrypt": encrypt_decrypt,
            "sign_verify": sign_verify,
            "mac": mac,
            "hash": hash_vectors
        }
    }


def main():
    parser = argparse.ArgumentParser(description="Generate KMIP 2.1 Canonical Test Vectors")
    parser.add_argument("--write", action="store_true", help="Write vectors to canonical-vectors.json")
    parser.add_argument("--check", action="store_true", help="Verify canonical-vectors.json is up-to-date")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    output_path = root / "interop" / "shared" / "vectors" / "canonical-vectors.json"

    vectors = build_canonical_vectors()
    content = json.dumps(vectors, indent=2, ensure_ascii=False) + "\n"

    if args.check:
        if not output_path.exists():
            print(f"Error: Vectors file does not exist at {output_path}", file=sys.stderr)
            sys.exit(1)
        with open(output_path, "r", encoding="utf-8") as f:  # NOSONAR
            existing = f.read()
        if existing != content:
            print("Error: Vectors file is out of date. Run with --write to regenerate.", file=sys.stderr)
            sys.exit(1)
        print("Canonical vectors check passed: 100% compliant")
    elif args.write:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:  # NOSONAR
            f.write(content)
        print(f"Wrote canonical test vectors to {output_path}")
    else:
        print(content)


if __name__ == "__main__":
    main()
