"""Behavior checks for generated public API signature metadata."""

import kmipkit
from kmipkit._generated import extension_registry, public_api


def test_generated_registry_manifest_has_unique_traceable_api_entries() -> None:
    assert extension_registry.FORMAT_VERSION == 1

    functions = extension_registry.FUNCTIONS
    identifiers = [function["id"] for function in functions]

    assert len(functions) > 40
    assert len(identifiers) == len(set(identifiers))
    assert {
        "extension_identity_create",
        "client_extension_registry_inspect",
        "ttlv_value_view_byte_at",
    } <= set(identifiers)

    for function in functions:
        assert function["name"]
        assert isinstance(function["parameters"], list)
        assert function["returnType"]
        assert function["requirementIds"]
        assert all(requirement.startswith("KMIPKIT-0012-FR-") for requirement in function["requirementIds"])


def test_generated_public_api_manifest_has_unique_python_declarations() -> None:
    assert public_api.FORMAT_VERSION == 2

    declarations = public_api.DECLARATIONS
    identifiers = [declaration["declarationId"] for declaration in declarations]

    assert len(declarations) > 40
    assert len(identifiers) == len(set(identifiers))
    assert {declaration["packageId"] for declaration in declarations} == {"python-kmipkit"}

    for declaration in declarations:
        assert declaration["declarationId"]
        assert declaration["kind"]
        assert declaration["name"]


def test_managed_objects_module_is_exported_from_the_package() -> None:
    assert "managed_objects" in kmipkit.__all__
    assert kmipkit.managed_objects.__name__ == "kmipkit.managed_objects"
