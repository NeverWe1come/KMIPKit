"""Keep the Rust RNG Seed builder blocked until binding parity lands."""

import unittest

from tools.api_manifest.tests.fluent_builder_manifest_contract import (
    assert_fluent_builder_manifest_contract,
)


class FluentRngSeedBuilderManifestTests(unittest.TestCase):
    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        assert_fluent_builder_manifest_contract(
            self,
            builder_name="RngSeedBuilder",
            operation_name="rng_seed",
            methods={
                "build",
                "data",
                "execute",
                "to_batch_item",
                "validate",
                "with_asynchronous_indicator",
                "with_options",
            },
            additional_exports={
                "kmipkit_client::FluentBuildError::MissingRngSeedData",
                "kmipkit_client::fluent::FluentBuildError::MissingRngSeedData",
            },
        )


if __name__ == "__main__":
    unittest.main()
