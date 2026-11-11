"""KMIPKIT-0095: explicit response renderers return owned zeroizing bytes."""
from types import SimpleNamespace

import pytest
from kmipkit import client, credentials, errors, ttlv
from kmipkit._client_ffi import ffi


@pytest.mark.parametrize("name", ["ttlv_hex", "json", "xml"])
def test_render_is_inspection_only_and_transfers_native_owner(monkeypatch, name):
    calls = []
    result = client.BatchItemResult(client.BatchOutcome.COMPLETED, 0, None, None, None)
    class Item:
        def _pointer(self):
            return ffi.cast("kmipkit_client_batch_item_response_t *", 41)
    object.__setattr__(result, "_native_response_item", Item())
    def render(item, limits, output):
        calls.append((item, limits))
        output[0] = ffi.cast("kmipkit_secret_bytes_t *", 42)
        return 0
    monkeypatch.setattr(client, "lib", SimpleNamespace(**{
        "kmipkit_client_batch_item_response_" + name: render,
        "kmipkit_secret_bytes_release": lambda pointer: None,
    }))
    owner = object()
    monkeypatch.setattr(client, "_wrap_owned_handle", lambda cls, pointer: owner if cls is credentials.SecretBytes else None)
    with ttlv.CodecLimits.defaults() as limits:
        assert getattr(result, name)(limits) is owner
    assert len(calls) == 1
    assert "41" not in repr(result)


@pytest.mark.parametrize("name", ["ttlv_hex", "json", "xml"])
def test_render_requires_live_response_and_explicit_limits(name):
    result = client.BatchItemResult(client.BatchOutcome.COMPLETED, 0, None, None, None)
    with ttlv.CodecLimits.defaults() as limits:
        with pytest.raises(errors.InvalidInputError):
            getattr(result, name)(limits)
    with pytest.raises(errors.InvalidInputError):
        getattr(result, name)(None)
