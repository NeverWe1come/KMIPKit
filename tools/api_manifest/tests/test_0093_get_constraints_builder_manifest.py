"""Keep the Rust Get Constraints builder blocked until binding parity lands."""

import unittest

from tools.api_manifest.tests.fluent_builder_manifest_contract import (
    assert_fluent_builder_manifest_contract,
)


class FluentGetConstraintsBuilderManifestTests(unittest.TestCase):
    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        assert_fluent_builder_manifest_contract(
            self,
            builder_name="GetConstraintsBuilder",
            operation_name="get_constraints",
            methods={
                "build",
                "execute",
                "to_batch_item",
                "validate",
                "with_asynchronous_indicator",
                "with_options",
            },
        )


if __name__ == "__main__":
    unittest.main()
