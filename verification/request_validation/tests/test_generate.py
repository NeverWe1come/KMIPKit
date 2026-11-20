"""Unit tests for deterministic request-schema code generation."""

from pathlib import Path
import unittest

from verification.request_validation.generate import field_descriptor, split_row


ROOT = Path(__file__).resolve().parents[3]


class NestedSchemaNameTests(unittest.TestCase):
    def test_interface_schema_name_keeps_digits_and_outer_type(self) -> None:
        matrix = ROOT / "verification/request_validation/operation-schema-matrix.md"
        row = next(
            line
            for line in matrix.read_text(encoding="utf-8").splitlines()
            if line.startswith("| `KMIPKIT-0051-R127` |")
        )

        descriptor = field_descriptor(split_row(row))

        self.assertIn(
            'field!(R127 "PKCS#11 Interface" 0x0042_0159 0x001 0 1 Pkcs11Interface)',
            descriptor,
        )
        self.assertNotIn("FieldDescriptor {", descriptor)

    def test_input_parameters_schema_name_keeps_digits_and_outer_type(self) -> None:
        matrix = ROOT / "verification/request_validation/operation-schema-matrix.md"
        row = next(
            line
            for line in matrix.read_text(encoding="utf-8").splitlines()
            if line.startswith("| `KMIPKIT-0051-R130` |")
        )

        descriptor = field_descriptor(split_row(row))

        self.assertIn(
            'field!(R130 "PKCS#11 Input Parameters" 0x0042_015b 0x001 0 1 Pkcs11InputParameters)',
            descriptor,
        )
        self.assertNotIn("FieldDescriptor {", descriptor)

    def test_managed_object_uses_compact_tag_marker(self) -> None:
        matrix = ROOT / "verification/request_validation/operation-schema-matrix.md"
        row = next(
            line
            for line in matrix.read_text(encoding="utf-8").splitlines()
            if line.startswith("| `KMIPKIT-0051-R140` |")
        )

        descriptor = field_descriptor(split_row(row))

        self.assertEqual(
            '    field!(R140 "Any Object (Section 2)" managed 0x001 1 1 none),',
            descriptor,
        )


if __name__ == "__main__":
    unittest.main()
