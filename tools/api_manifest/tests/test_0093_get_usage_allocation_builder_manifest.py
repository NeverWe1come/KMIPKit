"""Keep the Rust Usage Allocation builder blocked until binding parity lands."""

import unittest

from tools.api_manifest.tests.fluent_builder_manifest_contract import (
    assert_fluent_builder_manifest_contract,
)


class FluentGetUsageAllocationBuilderManifestTests(unittest.TestCase):
    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        assert_fluent_builder_manifest_contract(
            self,
            builder_name="GetUsageAllocationBuilder",
            operation_name="get_usage_allocation",
            methods={
                "build",
                "execute",
                "to_batch_item",
                "unique_identifier",
                "usage_limits_count",
                "validate",
                "with_asynchronous_indicator",
                "with_options",
            },
            additional_exports={
                "kmipkit_client::FluentBuildError::MissingUsageLimitsCount",
                "kmipkit_client::fluent::FluentBuildError::MissingUsageLimitsCount",
            },
        )


if __name__ == "__main__":
    unittest.main()
