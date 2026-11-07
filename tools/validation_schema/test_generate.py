"""Unit tests for deterministic request-schema code generation."""

from pathlib import Path
import unittest

from tools.validation_schema.generate import field_descriptor, split_row


ROOT = Path(__file__).resolve().parents[2]


class NestedSchemaNameTests(unittest.TestCase):
    def test_interface_schema_name_keeps_digits_and_outer_type(self) -> None:
        matrix = ROOT / "specs/051-pre-send-validation/gap-matrix.md"
        row = next(
            line
            for line in matrix.read_text(encoding="utf-8").splitlines()
            if line.startswith("| `KMIPKIT-0051-R127` |")
        )

        descriptor = field_descriptor(split_row(row))

        self.assertIn(
            "nested_schema: Some(NestedStructureSchema::Pkcs11Interface)",
            descriptor,
        )
        self.assertIn("types: TYPE_STRUCTURE,", descriptor)

    def test_input_parameters_schema_name_keeps_digits_and_outer_type(self) -> None:
        matrix = ROOT / "specs/051-pre-send-validation/gap-matrix.md"
        row = next(
            line
            for line in matrix.read_text(encoding="utf-8").splitlines()
            if line.startswith("| `KMIPKIT-0051-R130` |")
        )

        descriptor = field_descriptor(split_row(row))

        self.assertIn(
            "nested_schema: Some(NestedStructureSchema::Pkcs11InputParameters)",
            descriptor,
        )
        self.assertIn("types: TYPE_STRUCTURE,", descriptor)


if __name__ == "__main__":
    unittest.main()
