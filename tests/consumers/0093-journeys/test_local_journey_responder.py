"""KMIPKIT-0093-FR-013; KMIP 2.1 §§4.35, 6.1.2, 11.45 and 11.56."""

import struct
import unittest

from local_journey_responder import _item, _structure, _validate_operation
import local_journey_responder as responder


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


class BatchIdPlaceholderJourneyContractTests(unittest.TestCase):
    """KMIP 2.1 §6.1; batch order and ID Placeholder semantics."""

    @staticmethod
    def _batch_item(
        operation: int, payload: bytes = b"", unique_id: bytes | None = None
    ) -> bytes:
        fields = []
        if unique_id is not None:
            fields.append(_item(0x420093, 8, unique_id))
        fields.append(_item(0x42005C, 5, struct.pack(">I", operation)))
        if payload:
            fields.append(_structure(0x420079, payload))
        return _structure(0x42000F, *fields)

    @staticmethod
    def _create_payload() -> bytes:
        return b"".join(
            (
                _item(0x420057, 5, struct.pack(">I", 2)),
                _item(0x420098, 7, b"fluent-batch-placeholder"),
                _item(0x420028, 5, struct.pack(">I", 3)),
                _item(0x42002A, 2, struct.pack(">i", 256)),
            )
        )

    @classmethod
    def _request_frame(
        cls,
        items: tuple[bytes, ...],
        *,
        ordered: bool = True,
        declared_count: int = 2,
    ) -> bytes:
        version = _structure(
            0x420069,
            _item(0x42006A, 2, struct.pack(">i", 2)),
            _item(0x42006B, 2, struct.pack(">i", 1)),
        )
        header_fields = [
            version,
            _item(0x420092, 9, struct.pack(">q", 1)),
            _item(0x42000D, 2, struct.pack(">i", declared_count)),
        ]
        if ordered:
            header_fields.append(_item(0x420010, 6, struct.pack(">Q", 1)))
        header = _structure(0x420077, *header_fields)
        body = header + b"".join(items)
        return _item(0x420078, 1, body)

    @classmethod
    def _valid_request(cls) -> bytes:
        create_payload = cls._create_payload()
        get_payload = _item(0x420094, 5, struct.pack(">I", 1))
        return cls._request_frame(
            (
                cls._batch_item(1, create_payload, b"create-placeholder-source"),
                cls._batch_item(10, get_payload, b"get-placeholder-target"),
            )
        )

    def test_batch_parser_preserves_create_then_placeholder_get(self) -> None:
        parser = getattr(responder, "_request_batch", None)
        self.assertTrue(callable(parser), "fixture must parse every ordered batch item")
        items, header = parser(self._valid_request())
        self.assertEqual([operation for operation, _fields in items], [1, 10])
        self.assertEqual(
            [
                next(value for tag, kind, value in fields if tag == 0x420093 and kind == 8)
                for _operation, fields in items
            ],
            [b"create-placeholder-source", b"get-placeholder-target"],
        )
        self.assertIn((0x42000D, 2, struct.pack(">i", 2)), header)
        self.assertIn((0x420010, 6, struct.pack(">Q", 1)), header)

    def test_batch_parser_rejects_declared_count_that_disagrees_with_items(self) -> None:
        parser = getattr(responder, "_request_batch", None)
        self.assertTrue(callable(parser), "fixture must parse every ordered batch item")
        malformed = self._request_frame(
            (
                self._batch_item(1, _item(0x420057, 5, struct.pack(">I", 2))),
                self._batch_item(10, _item(0x420094, 5, struct.pack(">I", 1))),
            ),
            declared_count=1,
        )
        with self.assertRaisesRegex(RuntimeError, "Batch Count"):
            parser(malformed)

    def test_placeholder_batch_rejects_unordered_or_non_placeholder_get(self) -> None:
        validator = getattr(responder, "_validate_placeholder_batch", None)
        self.assertTrue(callable(validator), "fixture must validate placeholder resolution")
        items, header = responder._request_batch(self._valid_request())
        validator(items, header)

        unordered, unordered_header = responder._request_batch(
            self._request_frame(
                (
                    self._batch_item(1, self._create_payload(), b"create-placeholder-source"),
                    self._batch_item(10, _item(0x420094, 5, struct.pack(">I", 1)), b"get-placeholder-target"),
                ),
                ordered=False,
            )
        )
        with self.assertRaisesRegex(RuntimeError, "Batch Order Option"):
            validator(unordered, unordered_header)

        missing_ids, missing_ids_header = responder._request_batch(
            self._request_frame(
                (
                    self._batch_item(1, self._create_payload()),
                    self._batch_item(10, _item(0x420094, 5, struct.pack(">I", 1))),
                )
            )
        )
        with self.assertRaisesRegex(RuntimeError, "Unique Batch Item ID"):
            validator(missing_ids, missing_ids_header)

        wrong_identifier, wrong_header = responder._request_batch(
            self._request_frame(
                (
                    self._batch_item(1, self._create_payload(), b"create-placeholder-source"),
                    self._batch_item(10, _item(0x420094, 7, b"guessed-key-id"), b"get-placeholder-target"),
                )
            )
        )
        with self.assertRaisesRegex(RuntimeError, "ID Placeholder"):
            validator(wrong_identifier, wrong_header)

        reversed_items, reversed_header = responder._request_batch(
            self._request_frame(
                (
                    self._batch_item(10, _item(0x420094, 5, struct.pack(">I", 1)), b"get-placeholder-target"),
                    self._batch_item(1, self._create_payload(), b"create-placeholder-source"),
                )
            )
        )
        with self.assertRaisesRegex(RuntimeError, "Create.*Get"):
            validator(reversed_items, reversed_header)

    def test_batch_response_returns_created_identifier_for_both_items(self) -> None:
        response_builder = getattr(responder, "_response_batch", None)
        self.assertTrue(callable(response_builder), "fixture must encode a full batch response")
        actual_identifier = b"server-created-batch-key"
        frame = response_builder(
            (
                (1, b"create-placeholder-source"),
                (10, b"get-placeholder-target"),
            ),
            actual_identifier,
        )
        top_level = responder._children(frame[8:])
        response_header = next(
            value for tag, kind, value in top_level if tag == 0x42007A and kind == 1
        )
        self.assertIn(
            (0x42000D, 2, struct.pack(">i", 2)),
            responder._children(response_header),
        )
        response_items = [
            value
            for tag, kind, value in top_level
            if tag == 0x42000F and kind == 1
        ]
        self.assertEqual(len(response_items), 2)
        decoded = [responder._children(item) for item in response_items]
        self.assertEqual(
            [(tag, kind) for tag, kind, _value in decoded[0][:3]],
            [(0x42005C, 5), (0x420093, 8), (0x42007F, 5)],
        )
        self.assertEqual(
            [
                next(value for tag, kind, value in fields if tag == 0x42005C and kind == 5)
                for fields in decoded
            ],
            [struct.pack(">I", 1), struct.pack(">I", 10)],
        )
        self.assertEqual(
            [
                next(value for tag, kind, value in fields if tag == 0x420093 and kind == 8)
                for fields in decoded
            ],
            [b"create-placeholder-source", b"get-placeholder-target"],
        )
        for fields in decoded:
            payload = next(value for tag, kind, value in fields if tag == 0x42007C and kind == 1)
            identifiers = [
                value
                for tag, kind, value in responder._flatten(responder._children(payload))
                if tag == 0x420094 and kind == 7
            ]
            self.assertEqual(identifiers, [actual_identifier])
