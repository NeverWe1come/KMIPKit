from __future__ import annotations

import pytest

from kmipkit import errors, fluent, protocol_values, ttlv
from kmipkit.client import RequestOptions
from kmipkit.typed_attributes import StandardAttribute


def test_create_key_pair_builder_validates_explicit_choices_and_builds_typed_item() -> None:
    with fluent.CreateKeyPairBuilder() as builder:
        with pytest.raises(errors.KmipKitError):
            builder.build()

        item = (
            builder.algorithm(protocol_values.CryptographicAlgorithm.RSA)
            .length(2048)
            .seed(b"test seed")
            .private_usage_mask(protocol_values.CryptographicUsageMask.SIGN)
            .public_usage_mask(protocol_values.CryptographicUsageMask.VERIFY)
            .build()
        )
        try:
            assert item.operation == protocol_values.Operation.CREATE_KEY_PAIR
        finally:
            item.close()


def test_request_namespace_exposes_create_key_pair_builder() -> None:
    assert fluent.Request.create_key_pair.__annotations__["return"] == "CreateKeyPairBuilder"
    with fluent.Request(None).create_key_pair() as builder:
        assert isinstance(builder, fluent.CreateKeyPairBuilder)


def test_create_key_pair_builder_exercises_attributes_options_and_input_validation() -> None:
    with ttlv.codec_limits_defaults() as limits:
        with ttlv.raw_tag_from_raw(0x54F001) as raw_tag:
            with ttlv.raw_tag_try_checked(raw_tag) as expert_tag:
                with ttlv.ttlv_value_integer(17) as expert_value:
                    with ttlv.ttlv_item_create(expert_tag, expert_value, limits) as expert_item:
                        with ttlv.ttlv_value_text_string(b"key pair description", limits) as description:
                            with ttlv.ttlv_structure_create() as nested:
                                with ttlv.ttlv_value_structure(nested, limits) as nested_value:
                                    with ttlv.ttlv_item_create(
                                        expert_tag, nested_value, limits
                                    ) as nested_item:
                                        with fluent.CreateKeyPairBuilder() as builder:
                                            assert builder.private_name("signing key") is builder
                                            assert builder.public_name("verification key") is builder
                                            assert builder.add_common_attribute_item(expert_item) is builder
                                            assert builder.add_private_attribute_item(expert_item) is builder
                                            assert builder.add_public_attribute_item(expert_item) is builder
                                            assert builder.add_common_catalogued_attribute(
                                                StandardAttribute.DESCRIPTION, description
                                            ) is builder
                                            assert builder.add_private_catalogued_attribute(
                                                StandardAttribute.DESCRIPTION, description
                                            ) is builder
                                            assert builder.add_public_catalogued_attribute(
                                                StandardAttribute.DESCRIPTION, description
                                            ) is builder
                                            assert builder.add_common_vendor_attribute(
                                                "Vendor_Name.1", "profile", nested_item
                                            ) is builder
                                            assert builder.add_private_vendor_attribute(
                                                "Vendor_Name.1", "policy", expert_item
                                            ) is builder
                                            assert builder.add_public_vendor_attribute(
                                                "Vendor_Name.1", "region", expert_item
                                            ) is builder
                                            assert builder.with_options(RequestOptions()) is builder
                                            assert builder.asynchronous_indicator(0xFFFF_FFFF) is builder
                                            builder.algorithm(protocol_values.CryptographicAlgorithm.ECDSA)
                                            builder.length(256)
                                            builder.private_usage_mask(
                                                protocol_values.CryptographicUsageMask.SIGN
                                            )
                                            builder.public_usage_mask(
                                                protocol_values.CryptographicUsageMask.VERIFY
                                            )
                                            builder.validate()
                                            item = builder.to_batch_item()
                                            try:
                                                assert item.operation == protocol_values.Operation.CREATE_KEY_PAIR
                                            finally:
                                                item.close()

                                            with pytest.raises(errors.KmipKitError):
                                                builder.private_name(1)  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.public_name(None)  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.length(True)
                                            with pytest.raises(errors.KmipKitError):
                                                builder.private_usage_mask(True)
                                            with pytest.raises(errors.KmipKitError):
                                                builder.public_usage_mask(1 << 32)
                                            with pytest.raises(errors.KmipKitError):
                                                builder.add_common_attribute_item(None)  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.add_common_catalogued_attribute(
                                                    StandardAttribute.DESCRIPTION, expert_item
                                                )  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.add_private_catalogued_attribute(
                                                    None, description
                                                )  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.add_common_vendor_attribute(
                                                    None, "profile", expert_item
                                                )  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.add_private_vendor_attribute(
                                                    "Vendor_Name.1", None, expert_item
                                                )  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.with_options(None)  # type: ignore[arg-type]
                                            with pytest.raises(errors.KmipKitError):
                                                builder.asynchronous_indicator(-1)
                                            with pytest.raises(errors.KmipKitError):
                                                builder.asynchronous_indicator(1 << 32)
                                            with pytest.raises(errors.KmipKitError):
                                                builder.execute(limits)
