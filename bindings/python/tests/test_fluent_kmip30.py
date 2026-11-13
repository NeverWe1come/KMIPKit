"""Public Python fluent surface for the assigned KMIP 3.0 operations."""

from kmipkit.fluent import Request


def test_request_exposes_every_kmip30_operation_builder() -> None:
    expected = (
        "create_credential",
        "create_group",
        "create_user",
        "deactivate",
        "decapsulate",
        "encapsulate",
        "obliterate",
    )
    for name in expected:
        assert callable(getattr(Request, name, None)), name
