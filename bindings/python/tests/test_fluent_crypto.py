"""Consumer contracts for operation-specific Python crypto builders."""

from __future__ import annotations

from kmipkit import fluent


def test_request_namespace_exposes_seven_operation_specific_crypto_builders() -> None:
    expected = {
        "encrypt": "EncryptBuilder",
        "decrypt": "DecryptBuilder",
        "sign": "SignBuilder",
        "signature_verify": "SignatureVerifyBuilder",
        "mac": "MacBuilder",
        "mac_verify": "MacVerifyBuilder",
        "hash": "HashBuilder",
    }

    for factory_name, builder_name in expected.items():
        assert callable(getattr(fluent.Request, factory_name, None))
        assert builder_name in fluent.__all__
        assert getattr(fluent, builder_name, None) is not None


def test_crypto_builders_expose_only_the_fields_of_their_typed_operation() -> None:
    common = {
        "cryptographic_parameters",
        "cryptographic_parameters_structure",
        "data",
        "data_value",
        "correlation_value",
        "init_indicator",
        "final_indicator",
        "with_options",
        "asynchronous_indicator",
        "build",
        "to_batch_item",
        "execute",
        "close",
    }
    operation_fields = {
        "EncryptBuilder": {"unique_identifier", "iv_counter_nonce", "authenticated_encryption_additional_data"},
        "DecryptBuilder": {"unique_identifier", "iv_counter_nonce", "authenticated_encryption_additional_data", "authenticated_encryption_tag"},
        "SignBuilder": {"unique_identifier", "digested_data"},
        "SignatureVerifyBuilder": {"unique_identifier", "digested_data", "signature_data"},
        "MacBuilder": {"unique_identifier"},
        "MacVerifyBuilder": {"unique_identifier", "mac_data"},
        "HashBuilder": set(),
    }

    for builder_name, specific_fields in operation_fields.items():
        builder_type = getattr(fluent, builder_name)
        assert common <= set(dir(builder_type))
        assert specific_fields <= set(dir(builder_type))
        for other_builder, other_fields in operation_fields.items():
            if other_builder == builder_name:
                continue
            forbidden = other_fields - specific_fields
            assert not (forbidden & set(dir(builder_type))), (
                f"{builder_name} exposes fields exclusive to {other_builder}"
            )
