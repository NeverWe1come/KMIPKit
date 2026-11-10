"""In-memory redacted request/result signatures; never emit KMIP bodies."""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
ELEMENTS = json.loads((ROOT / "specification/catalog/kmip-2.1.json").read_text())["elements"]


def name_key(value):
    return "".join(c.lower() for c in value if c.isalnum())


CODE_NAMES = {
    group: {name_key(e["name"]): int(e["wire_value"], 16) for e in ELEMENTS
            if e["kind"] == "enumeration_value"
            and e["parent_element_ids"] == ["KMIPKIT-ELEM-ENUMERATION-" + group]
            and e.get("allocation") == "assigned"}
    for group in ("RESULT-STATUS", "RESULT-REASON")
}


def result_code(value, group):
    if value is None:
        return None
    if isinstance(value, int) and not isinstance(value, bool):
        code = value
    elif isinstance(value, str) and value.lower().startswith("0x"):
        code = int(value, 16)
    elif isinstance(value, str) and value.isdecimal():
        code = int(value)
    elif isinstance(value, str) and name_key(value) in CODE_NAMES[group]:
        code = CODE_NAMES[group][name_key(value)]
    else:
        raise ValueError("unrecognized KMIP result code")
    if not 0 <= code <= 0xffffffff:
        raise ValueError("KMIP result code outside unsigned 32-bit range")
    return code


def parity_signature(report, request_trees=()):
    outcomes = tuple((c["case_id"], c["status"], c["library_correctness"],
                      c["server_compatibility"],
                      result_code(c.get("result_status"), "RESULT-STATUS"),
                      result_code(c.get("result_reason"), "RESULT-REASON"))
                     for c in report["cases"])
    return outcomes, tuple(request_trees)


class RequestNormalizer:
    """Preserve ordered tags/types and public scalars, with stable ID aliases.

    Response IDs seed aliases before subsequent requests, preserving resource
    relationships across setup/target/teardown. Raw alias keys stay in memory.
    """
    IDENTIFIERS = {0x420094, 0x420066, 0x42006f, 0x420076}
    SENSITIVE_CONTAINERS = {0x420025, 0x420043, 0x42000b}

    def __init__(self):
        self.aliases = {}

    def alias(self, namespace, kind, value):
        values = self.aliases.setdefault(namespace, {})
        identity = kind, value
        if identity not in values:
            values[identity] = f"{namespace}-{len(values)}"
        return values[identity]

    def observe_response(self, tree):
        for tag, kind, value in tree:
            if kind == 1:
                self.observe_response(value)
            elif tag in self.IDENTIFIERS and kind in (2, 3, 7):
                self.alias("identifier", kind, value)

    def normalize(self, tree):
        normalized = []
        for tag, kind, value in tree:
            if tag == 0x420092:  # OASIS §9: per-exchange Time Stamp.
                continue
            if tag in self.SENSITIVE_CONTAINERS:
                safe_value = "<redacted>"
            elif kind == 1:
                safe_value = self.normalize(value)
            elif tag in self.IDENTIFIERS and kind in (2, 3, 7):
                safe_value = self.alias("identifier", kind, value)
            elif tag == 0x420055 and kind == 7:
                safe_value = self.alias("name", kind, value)
            elif tag == 0x420093 and kind == 8:
                safe_value = self.alias("batch", kind, value)
            elif kind in (2, 3, 5, 6, 9, 10, 11):
                safe_value = value
            else:
                safe_value = "<redacted>"
            normalized.append((tag, kind, safe_value))
        return tuple(normalized)
