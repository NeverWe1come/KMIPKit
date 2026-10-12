import pytest

from kmipkit.ttlv import ItemType


def test_item_type_rejects_unassigned_kmip_21_codes() -> None:
    with pytest.raises(ValueError):
        ItemType(12)
