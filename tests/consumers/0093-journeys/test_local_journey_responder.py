"""KMIPKIT-0093-FR-013; KMIP 2.1 sections 6.1.2, 4.14 and 11.45."""

import struct
import unittest

from local_journey_responder import _item, _structure, _validate_operation


class KeyPairCurveContractTests(unittest.TestCase):
    def validate(self, domain: bytes) -> None:
        common = _structure(
            0x420126,
            _item(0x420028, 5, struct.pack(">I", 6)),
            _item(0x42002A, 2, struct.pack(">i", 256)),
            domain,
        )
        payload = common + _structure(0x420127, _item(0x42002C, 2, struct.pack(">i", 1)))
        payload += _structure(0x420128, _item(0x42002C, 2, struct.pack(">i", 2)))
        _validate_operation(2, [(0x420079, 1, payload)], [], b"")

    def test_requires_exact_explicit_p256_domain_parameters(self) -> None:
        self.validate(_structure(0x420029, _item(0x420075, 5, struct.pack(">I", 7))))

    def test_rejects_absent_wrong_type_wrong_curve_and_empty_domain(self) -> None:
        for domain in (
            b"",
            _structure(0x420029),
            _structure(0x420029, _item(0x420075, 2, struct.pack(">i", 7))),
            _structure(0x420029, _item(0x420075, 5, struct.pack(">I", 25))),
        ):
            with self.subTest(domain=domain.hex()), self.assertRaisesRegex(RuntimeError, "P-256"):
                self.validate(domain)
