"""Register a disposable key with a KMIP 2.x Attribute structure and read it back."""

from copy import deepcopy
from xml.etree.ElementTree import fromstring

from verification.interop.shared.tools.oasis_ttlv import WireValues, encode
from verification.interop.shared.tools.table_ttlv import (
    KEY_FORMATS, transparent_object_fixture,
)


def attribute_exchange(root, version, base):
    vectors = root / "verification/interop/shared/vectors/kmip2x-table-attributes.xml"
    fixtures = fromstring(vectors.read_bytes())
    fixture, = fixtures.findall("./Fixture[@row='Vendor Attribute']/Attribute")
    key, attributes = transparent_object_fixture(*KEY_FORMATS[0])
    attributes.append(deepcopy(fixture))
    names = WireValues(root, version, [])
    case = deepcopy(base)
    case.update(
        case_id="test-register-attribute-structure",
        description=f"Register and read back the vendor-qualified Attribute structure under KMIP {version}.",
        requires={
            "base_object": "Attribute",
            "attribute": "Vendor Attribute",
            "attribute_vendor_identification": fixture.find("VendorIdentification").attrib["value"],
            "attribute_reference_name": fixture.find("AttributeName").attrib["value"],
            "register_attribute_value": fixture.find("AttributeValue").attrib["value"],
            "register_object_type": "2",
            "register_object_ttlv": encode(key, names)[0].hex(),
            "register_attributes_ttlv": encode(attributes, names)[0].hex(),
            "controlled_expected_status": "Success",
        },
        isolation_recipe={
            "unique_name_prefix": "kmipkit-register-attribute-structure",
            "setup_steps": [],
            "target_step": "Register the key with Attribute and verify Get Attributes on the returned identifier",
            "teardown_steps": ["Destroy the registered fixture"],
        },
    )
    case["execution"]["assertions"] = [
        "register_request_contains_the_vendor_qualified_attribute_structure",
        "get_attributes_returns_the_vendor_name_and_original_value_for_the_registered_identifier",
        "the_registered_object_is_destroyed_even_if_the_readback_fails",
    ]
    return case
