from __future__ import annotations

import pytest

from kmipkit import errors, fluent, protocol_values


def test_create_key_pair_builder_validates_explicit_choices_and_builds_typed_item() -> None:
    with fluent.CreateKeyPairBuilder() as builder:
        with pytest.raises(errors.KmipKitError):
            builder.build()

        item = (
            builder.algorithm(protocol_values.CryptographicAlgorithm.RSA)
            .length(2048)
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

