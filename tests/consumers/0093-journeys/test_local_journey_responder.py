"""KMIPKIT-0093-FR-013; KMIP 2.1 §§4.35, 6.1.2, 11.45 and 11.56."""

import struct
import unittest

from local_journey_responder import _item, _structure, _validate_operation


class ExpertAttributeJourneyContractTests(unittest.TestCase):
    def _create_fields(self, attribute: bytes | None) -> list[tuple[int, int, bytes]]:
        payload_items = [
            _item(0x420057, 5, struct.pack(">I", 2)),
            _item(0x420098, 7, b"fluent-expert-attributes"),
            _item(0x420028, 5, struct.pack(">I", 3)),
            _item(0x42002A, 2, struct.pack(">i", 256)),
            _item(0x42002C, 2, struct.pack(">i", 12)),
        ]
        if attribute is not None:
            payload_items.append(attribute)
        return [(0x420079, 1, b"".join(payload_items))]

    def test_expert_attribute_create_requires_catalogued_object_group_text(self) -> None:
        invalid_attributes = (
            None,
            _item(0x420056, 2, struct.pack(">i", 1)),
            _item(0x420056, 7, b"wrong-expert-group"),
        )
        for attribute in invalid_attributes:
            with self.subTest(attribute=attribute), self.assertRaisesRegex(
                RuntimeError, "Object Group"
            ):
                _validate_operation(
                    1,
                    self._create_fields(attribute),
                    [],
                    b"fluent-expert-attributes",
                )

        _validate_operation(
            1,
            self._create_fields(_item(0x420056, 7, b"fluent-expert-group")),
            [],
            b"fluent-expert-attributes",
        )


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
