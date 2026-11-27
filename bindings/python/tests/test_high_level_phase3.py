"""Fase 3: presets, domain namespaces and KeyRef (no server needed)."""

import unittest

from kmipkit import (
    CipherSpec, HashAlgorithm, KdfSpec, KemSpec, KeyRef, KeySpec, MacSpec, SignatureSpec,
    extensions, fluent,
)


class HighLevelPhase3Test(unittest.TestCase):
    def test_key_spec_metadata(self) -> None:
        self.assertTrue(KeySpec.AES_256.is_symmetric)
        self.assertEqual(KeySpec.AES_256.length, 256)
        self.assertEqual(KeySpec.ECDSA_P256.recommended_curve, 7)
        self.assertTrue(KeySpec.ML_KEM_768.requires_kmip_30)
        self.assertFalse(KeySpec.RSA_2048.requires_kmip_30)
        self.assertTrue(KeySpec.RSA_2048.is_asymmetric)

    def test_all_presets_build_valid_parameters(self) -> None:
        for enum in (CipherSpec, SignatureSpec, MacSpec, KemSpec, HashAlgorithm, KdfSpec):
            for spec in enum:
                params = spec.to_cryptographic_parameters()
                params.close()

    def test_builders_accept_specs_and_reject_wrong_category(self) -> None:
        sym = fluent.SymmetricKeyBuilder(client=None).with_key_spec(KeySpec.AES_256)
        self.assertIsNotNone(sym)
        sym.close()
        with self.assertRaises(ValueError):
            fluent.SymmetricKeyBuilder(client=None).with_key_spec(KeySpec.RSA_2048)
        with self.assertRaises(ValueError):
            fluent.CreateKeyPairBuilder(client=None).with_key_spec(KeySpec.AES_256)
        pair = fluent.CreateKeyPairBuilder(client=None).with_key_spec(KeySpec.ML_KEM_768)
        pair.close()
        with self.assertRaises(TypeError):
            fluent.HashBuilder(None).with_hash_algorithm(MacSpec.HMAC_SHA256)

    def test_namespaces_and_key_ref_exist_on_detached_request(self) -> None:
        for name in ("keys", "crypto", "certificates", "secrets", "attributes",
                     "sessions", "admin", "async_ops"):
            self.assertTrue(hasattr(__import__("kmipkit").client.Client, name), name)
        self.assertTrue(hasattr(__import__("kmipkit").client.Client, "key"))
        ref = KeyRef(None, "k-1")
        self.assertEqual(ref.id, "k-1")
        self.assertIsInstance(extensions.UniqueIdentifier.text_string("k-1"),
                              extensions.UniqueIdentifier)

    def test_key_ref_delegates_operations(self) -> None:
        from unittest.mock import MagicMock, patch

        req = MagicMock()
        for m in ("get", "activate", "deactivate", "revoke", "destroy", "recover",
                  "archive", "obliterate", "encrypt", "decrypt", "sign",
                  "signature_verify", "mac", "mac_verify", "encapsulate", "decapsulate"):
            getattr(req, m).return_value = req
        req.unique_identifier.return_value = req

        ref = KeyRef(None, "k-1")
        with patch("kmipkit.high_level.fluent.Request", return_value=req):
            ref.get()
            ref.activate()
            ref.deactivate()
            ref.destroy()
            ref.recover()
            ref.archive()
            ref.obliterate()
            ref.encrypt()
            ref.decrypt()
            ref.sign()
            ref.verify()
            ref.signature_verify()
            ref.mac()
            ref.mac_verify()
            ref.encapsulate()
            ref.decapsulate(b"ct")
        self.assertEqual(req.unique_identifier.call_count, 10)
        req.destroy.assert_called_once()
        req.decapsulate.assert_called_once()


if __name__ == "__main__":
    unittest.main()
