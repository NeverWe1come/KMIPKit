import pytest

from kmipkit.ttlv import ItemType


def test_item_type_rejects_unassigned_kmip_21_codes() -> None:
    # KMIP Specification v2.1 §11.23 assigns only the eleven modeled item types.
    with pytest.raises(ValueError):
        ItemType(12)
