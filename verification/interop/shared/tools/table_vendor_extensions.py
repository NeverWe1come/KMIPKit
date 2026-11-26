"""Attach results from the provider-specific HTTPS JSON extension fixture."""

import json
import re

from verification.interop.shared.tools.cosmian_minimum import SUPPORT_TABLE

ROWS = {
    "AttributeIndex": ("Attribute Index", "Integer"),
    "Critical": ("Critical", "Boolean"),
    "ObjectGroupMember": ("Object Group Member", "Enumeration"),
}
GROUP_MEMBER_VALUES = ("Group_Member_Fresh", "Group_Member_Default")
HTTP_CHECKS = {
    "AttributeIndex": {"reject_wrong_type": 422, "set": 200, "read_back": 422, "destroy": 422},
    "Critical": {"reject_wrong_type": 422, "set": 200, "read_back": 422, "destroy": 422},
    "ObjectGroupMember": {"reject_wrong_type": 422, "set": 200, "read_back": 200, "destroy": 200},
}


def attach_evidence(root, inventory, paths):
    """Join provider behavior to table rows without assigning a binding route."""
    version = inventory["protocol_version"]
    rows = {r["name"]: r for r in inventory["rows"] if r["declared_support"] == "vendor_extension"}
    source = json.loads((root / SUPPORT_TABLE).read_text(encoding="utf-8"))
    seen = set()
    for path in paths:
        report = json.loads(path.read_text(encoding="utf-8"))
        if ((report["schema_version"], report["scope"], report["transport"], report["encoding"])
                != ("1.0", "live", "https", "JSON") or report["protocol_version"] != "2.1"
                or report["source_sha256"] != source["source_sha256"]):
            raise ValueError(f"{path.name}: unexpected extension fixture, version, or table source")
        server = report["server"]
        fingerprint = server["configuration_fingerprint"]
        if (server["product"], server["version"]) != ("Cosmian KMS", "5.28.0") or not re.fullmatch(
            r"[0-9a-f]{64}", fingerprint
        ) or not server["id"]:
            raise ValueError(f"{path.name}: unexpected extension provider identity")
        if set(report["observations"]) != ROWS.keys():
            raise ValueError(f"{path.name}: incomplete extension observations")
        for tag, observation in report["observations"].items():
            row_name, representation = ROWS[tag]
            expected = {"representation": representation, "http_checks": HTTP_CHECKS[tag]}
            if tag == "ObjectGroupMember":
                expected["omitted_values"] = list(GROUP_MEMBER_VALUES)
            if observation != expected:
                raise ValueError(f"{path.name}: unexpected {tag} extension evidence")
            if version != report["protocol_version"]:
                continue
            key = row_name, server["id"], fingerprint
            if key in seen:
                raise ValueError(f"{path.name}: duplicate {tag} extension evidence")
            seen.add(key)
            rows[row_name].setdefault("provider_fixture_results", []).append({
                "scope": "live", "transport": "https", "encoding": "JSON",
                "protocol_version": version, "server": server, "tag": tag, **observation,
            })
