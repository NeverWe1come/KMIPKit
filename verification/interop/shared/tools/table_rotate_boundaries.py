"""Build paired version-boundary recipes for the seven direct Rotate attributes."""

from copy import deepcopy
from xml.etree.ElementTree import fromstring

from verification.interop.shared.tools.oasis_ttlv import WireValues, encode


def rotate_cases(root, version, base):
    document = fromstring((root / "verification/interop/shared/vectors/kmip2x-table-attributes.xml").read_bytes())
    names = WireValues(root, "2.1", [])
    absent = version == "2.0"
    sections = {
        "Rotate Automatic": "4.48",
        "Rotate Date": "4.49",
        "Rotate Generation": "4.50",
        "Rotate Interval": "4.51",
        "Rotate Latest": "4.52",
        "Rotate Name": "4.53",
        "Rotate Offset": "4.54",
    }
    for entry in document:
        row = entry.attrib["row"]
        if row not in sections:
            continue
        attribute, = entry
        wire, _ = encode(attribute, names)
        case = deepcopy(base)
        case["case_id"] = f"test-v{version.replace('.', '')}-locate-{'-'.join(row.lower().split())}"
        case["category"] = "attributes"
        case["description"] = f"Locate with the direct {row} attribute under KMIP {version}."
        case["requires"] = {"attribute": row, "rotate_attribute_ttlv": wire.hex()}
        if absent:
            case["requires"]["expected_status"] = "NotSent"
        case["normative_refs"] = [
            {"document": "OASIS KMIP 2.0 Specification", "section": "11.54"},
            {"document": "OASIS KMIP 2.1 Specification", "section": sections[row]},
        ]
        case["isolation_recipe"] = {
            "unique_name_prefix": f"kmipkit-{case['case_id']}",
            "setup_steps": [],
            "target_step": f"Build and execute Locate with {row}",
            "teardown_steps": [],
        }
        case["execution"] = {
            "mode": "server_exchange", "protocol_versions": [version],
            "steps": ["construct_locate_with_rotate_attribute", "execute_target", "verify_delivery_or_success"],
            "assertions": ["version_gate_rejects_attribute_before_transport"] if absent else [
                "request_preserves_direct_rotate_attribute", "locate_returns_a_bounded_result",
            ],
            "mutation": "read_only", "status": "planned",
        }
        yield row, case
