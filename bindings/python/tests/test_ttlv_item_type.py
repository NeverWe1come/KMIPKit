import pytest

from kmipkit.ttlv import ItemType


def test_item_type_rejects_unassigned_codes() -> None:
    assert len(ItemType) == 14
    for i in range(1, 15):
        assert ItemType(i) is not None
    with pytest.raises(ValueError):
        ItemType(15)
