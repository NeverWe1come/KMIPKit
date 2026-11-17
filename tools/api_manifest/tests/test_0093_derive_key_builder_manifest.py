"""Keep the Rust Derive Key builder blocked until binding parity lands."""

import unittest

from tools.api_manifest.tests.fluent_builder_manifest_contract import (
    assert_fluent_builder_manifest_contract,
)


class FluentDeriveKeyBuilderManifestTests(unittest.TestCase):
    def test_builder_exports_are_traced_and_blocked_for_four_language_parity(self) -> None:
        assert_fluent_builder_manifest_contract(
            self,
            builder_name="DeriveKeyBuilder",
            operation_name="derive_key",
            methods={
                "attributes",
                "build",
                "derivation_method",
                "derivation_parameters",
                "execute",
                "object_type",
                "to_batch_item",
                "unique_identifier",
                "validate",
                "with_asynchronous_indicator",
                "with_options",
            },
            additional_exports={
                "kmipkit_client::FluentBuildError::MissingDeriveKeyAttributes",
                "kmipkit_client::FluentBuildError::MissingDeriveKeyMethod",
                "kmipkit_client::FluentBuildError::MissingDeriveKeyObjectType",
                "kmipkit_client::FluentBuildError::MissingDeriveKeyParameters",
                "kmipkit_client::FluentBuildError::MissingDeriveKeyUniqueIdentifier",
                "kmipkit_client::fluent::FluentBuildError::MissingDeriveKeyAttributes",
                "kmipkit_client::fluent::FluentBuildError::MissingDeriveKeyMethod",
                "kmipkit_client::fluent::FluentBuildError::MissingDeriveKeyObjectType",
                "kmipkit_client::fluent::FluentBuildError::MissingDeriveKeyParameters",
                "kmipkit_client::fluent::FluentBuildError::MissingDeriveKeyUniqueIdentifier",
            },
        )


if __name__ == "__main__":
    unittest.main()
