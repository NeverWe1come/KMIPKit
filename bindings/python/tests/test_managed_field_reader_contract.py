"""Contract test for the typed managed-field C ABI used by all bindings."""

from kmipkit._client_ffi import lib


def test_typed_managed_field_reader_is_available_through_cffi():
    assert hasattr(lib, "kmipkit_managed_value_field_read")
