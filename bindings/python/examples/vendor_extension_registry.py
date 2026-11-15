"""Run a local vendor-extension registry and Discover Versions example.

This example inspects shared TTLV fixtures and constructs a local batch item.
It does not open a transport or send a request.

Unknown response Message Extensions follow KMIPKIT-0007: an unknown critical
extension is rejected, while an unknown non-critical extension remains
available as generic TTLV. Python-managed string and byte copies cannot be
reliably zeroized by KMIPKit; this example uses only non-secret fixture values.
"""

from __future__ import annotations

import json
from contextlib import ExitStack
from pathlib import Path
from typing import Any

from kmipkit import extensions, ttlv


ROOT = Path(__file__).resolve().parents[3]
FIXTURE_PATH = ROOT / "tests" / "fixtures" / "extensions" / "cases.json"
VENDOR_IDENTIFIER = "example.vendor"


def _tag(raw_tag: str, handles: ExitStack) -> ttlv.Tag:
    raw = handles.enter_context(ttlv.raw_tag_from_raw(int(raw_tag, 16)))
    return handles.enter_context(ttlv.raw_tag_try_checked(raw))


def _fixture_structure(
    children: list[dict[str, Any]], limits: ttlv.CodecLimits, handles: ExitStack
) -> ttlv.Structure:
    structure = handles.enter_context(ttlv.ttlv_structure_create())
    constructors = {
        "Integer": ttlv.ttlv_value_integer,
        "LongInteger": ttlv.ttlv_value_long_integer,
        "Enumeration": ttlv.ttlv_value_enumeration,
    }
    for child in children:
        if child["type"] == "Structure":
            nested = _fixture_structure(child["children"], limits, handles)
            value = ttlv.ttlv_value_structure(nested, limits)
        elif child["type"] == "TextString":
            value = ttlv.ttlv_value_text_string(child["value"].encode("utf-8"), limits)
        else:
            value = constructors[child["type"]](child["value"])
        value = handles.enter_context(value)
        item = handles.enter_context(
            ttlv.ttlv_item_create(_tag(child["tag"], handles), value, limits)
        )
        structure = handles.enter_context(ttlv.ttlv_structure_with_item(structure, item, limits))
    return structure


def _alpha_definition(
    handles: ExitStack, limits: ttlv.CodecLimits
) -> extensions.ExtensionDefinition:
    identity = handles.enter_context(
        extensions.create_extension_identity(VENDOR_IDENTIFIER, "alpha", "1")
    )
    compatibility = handles.enter_context(
        extensions.create_compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
    )
    discriminator_tag = _tag("0x420001", handles)
    path = handles.enter_context(extensions.create_ttlv_path(discriminator_tag))
    scalar = handles.enter_context(ttlv.ttlv_value_text_string(b"alpha-v1", limits))
    discriminator = handles.enter_context(extensions.create_discriminator(path, scalar))
    text_schema = handles.enter_context(extensions.scalar_schema(ttlv.ItemType.TextString))
    number_schema = handles.enter_context(extensions.scalar_schema(ttlv.ItemType.LongInteger))
    text_rule = handles.enter_context(extensions.required_child_rule(discriminator_tag, text_schema))
    number_rule = handles.enter_context(
        extensions.required_child_rule(_tag("0x420004", handles), number_schema)
    )
    schema = handles.enter_context(extensions.structure_schema([text_rule, number_rule], [], True))
    return handles.enter_context(
        extensions.create_extension_definition(identity, compatibility, discriminator, schema)
    )


def _beta_definition(
    handles: ExitStack, limits: ttlv.CodecLimits
) -> extensions.ExtensionDefinition:
    identity = handles.enter_context(
        extensions.create_extension_identity(VENDOR_IDENTIFIER, "ambiguous-beta", "1")
    )
    compatibility = handles.enter_context(
        extensions.create_compatibility(2, 1, 2, 1, "0.0.0", "99.0.0")
    )
    parent_tag = _tag("0x540010", handles)
    discriminator_tag = _tag("0x540012", handles)
    path = handles.enter_context(extensions.create_ttlv_path(parent_tag))
    path = handles.enter_context(extensions.with_child_tag(path, discriminator_tag))
    scalar = handles.enter_context(
        ttlv.ttlv_value_text_string(b"route-beta", limits)
    )
    discriminator = handles.enter_context(extensions.create_discriminator(path, scalar))
    text_schema = handles.enter_context(extensions.scalar_schema(ttlv.ItemType.TextString))
    nested_rule = handles.enter_context(
        extensions.required_child_rule(discriminator_tag, text_schema)
    )
    nested_schema = handles.enter_context(
        extensions.structure_schema([nested_rule], [], True)
    )
    parent_rule = handles.enter_context(
        extensions.required_child_rule(parent_tag, nested_schema)
    )
    schema = handles.enter_context(
        extensions.structure_schema([parent_rule], [], True)
    )
    return handles.enter_context(
        extensions.create_extension_definition(identity, compatibility, discriminator, schema)
    )


def _codec_limits(handles: ExitStack) -> ttlv.CodecLimits:
    return handles.enter_context(ttlv.codec_limits_defaults())


def _top_level_tags(view: ttlv.TtlvStructureView, handles: ExitStack) -> list[str]:
    tags = []
    for index in range(ttlv.ttlv_structure_view_item_count(view)):
        item = handles.enter_context(ttlv.ttlv_structure_view_item_at(view, index))
        tag = handles.enter_context(ttlv.ttlv_item_view_tag(item))
        tags.append(f"0x{ttlv.tag_value(tag):06X}")
    return tags


def main() -> None:
    corpus = json.loads(FIXTURE_PATH.read_text(encoding="utf-8"))
    fixtures = {item["id"]: item for item in corpus["cases"]}

    with ExitStack() as handles:
        codec_limits = _codec_limits(handles)
        alpha = _alpha_definition(handles, codec_limits)
        beta = _beta_definition(handles, codec_limits)
        registry_limits = extensions.default_extension_registry_limits()
        registry = handles.enter_context(
            extensions.create_client_extension_registry([alpha, beta], registry_limits)
        )
        configuration = handles.enter_context(extensions.create_client_configuration(registry))
        configured_registry = handles.enter_context(
            extensions.client_configuration_extension_registry(configuration)
        )

        alpha_case = fixtures["valid-recognized"]["extension"]
        alpha_payload = _fixture_structure(
            alpha_case["payload"]["children"], codec_limits, handles
        )
        recognition = handles.enter_context(
            extensions.inspect_extension(
                configured_registry,
                alpha_case["vendorIdentification"],
                alpha_payload,
                codec_limits,
            )
        )
        recognized = extensions.is_recognized(recognition)
        validated = handles.enter_context(extensions.validated_value(recognition))
        validated_identity = handles.enter_context(
            extensions.validated_extension_identity(validated)
        )
        generic_view = handles.enter_context(
            extensions.extension_recognition_generic_value(recognition)
        )
        preserved_tags = _top_level_tags(generic_view, handles)

        beta_case = fixtures["multiply-matching"]["extension"]
        beta_payload = _fixture_structure(
            beta_case["payload"]["children"], codec_limits, handles
        )
        alpha_outbound = extensions.validate_extension_value(
            configured_registry, alpha.identity, alpha_payload, codec_limits
        )
        beta_outbound = extensions.validate_extension_value(
            configured_registry, beta.identity, beta_payload, codec_limits
        )
        alpha_attachment = extensions.create_client_request_message_extension(
            alpha_outbound, False
        )
        beta_attachment = extensions.create_client_request_message_extension(
            beta_outbound, True
        )
        handles.enter_context(alpha_attachment)
        handles.enter_context(beta_attachment)
        batch_item = handles.enter_context(extensions.client_batch_item_discover_versions())
        batch_item = handles.enter_context(extensions.with_extension(batch_item, alpha_attachment))
        batch_item = handles.enter_context(extensions.with_extension(batch_item, beta_attachment))

        ordered_attachments = []
        for index in range(extensions.client_batch_item_extension_count(batch_item)):
            identity = handles.enter_context(
                extensions.client_batch_item_extension_identity_at(batch_item, index)
            )
            ordered_attachments.append(
                {
                    "identity": "/".join(
                        (
                            identity.vendor_identifier,
                            identity.name,
                            identity.version,
                        )
                    ),
                    "criticality_indicator": (
                        extensions.client_batch_item_extension_criticality_indicator_at(
                            batch_item, index
                        )
                    ),
                }
            )

        summary = {
            "inbound": {
                "identity": "/".join(
                    (
                        validated_identity.vendor_identifier,
                        validated_identity.name,
                        validated_identity.version,
                    )
                ),
                "recognized": recognized,
                "typed_value_available": validated is not None,
                "preserved_tags": preserved_tags,
            },
            "discover_versions": ordered_attachments,
        }
        print(json.dumps(summary, sort_keys=True))


if __name__ == "__main__":
    main()
