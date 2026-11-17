"""Keep the Rust generic Create builder blocked until binding parity lands."""

import unittest

from tools.api_manifest.tests.fluent_builder_manifest_contract import (
    assert_fluent_builder_manifest_contract,
)


class FluentCreateBuilderManifestTests(unittest.TestCase):
    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        assert_fluent_builder_manifest_contract(
            self,
            builder_name="CreateBuilder",
            operation_name="create",
            methods={
                "attributes",
                "build",
                "execute",
                "object_type",
                "protection_storage_masks",
                "to_batch_item",
                "validate",
                "with_asynchronous_indicator",
                "with_options",
            },
            additional_exports={
                "kmipkit_client::FluentBuildError::MissingCreateObjectType",
                "kmipkit_client::fluent::FluentBuildError::MissingCreateObjectType",
            },
        )


if __name__ == "__main__":
    unittest.main()
