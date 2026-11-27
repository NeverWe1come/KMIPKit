"""Register isolated DSA and DH fixtures so the target receives the key material."""

from copy import deepcopy
from hashlib import sha256
from xml.etree.ElementTree import fromstring

from verification.interop.shared.tools.oasis_ttlv import WireValues, encode
from verification.interop.shared.tools.table_ttlv import OASIS_FIXTURES, field, transparent_object_fixture


# RFC 7919 Appendix A.1. Private components in these fixtures are public test data.
FFDHE2048 = int(
    "FFFFFFFFFFFFFFFFADF85458A2BB4A9AAFDC5620273D3CF1D8B9C583CE2D3695"
    "A9E13641146433FBCC939DCE249B3EF97D2FE363630C75D8F681B202AEC4617AD3"
    "DF1ED5D5FD65612433F51F5F066ED0856365553DED1AF3B557135E7F57C935984"
    "F0C70E0E68B77E2A689DAF3EFE8721DF158A136ADE73530ACCA4F483A797ABC0AB"
    "182B324FB61D108A94BB2C8E3FBB96ADAB760D7F4681D4F42A3DE394DF4AE56ED"
    "E76372BB190B07A7C8EE0A6D709E02FCE1CDF7E2ECC03404CD28342F619172FE9"
    "CE98583FF8E4F1232EEF28183C3FE3B1B4C6FAD733BB5FCBC2EC22005C58EF183"
    "7D1683B2C6F34A26C1B2EFFA886B423861285C97FFFFFFFFFFFFFFFF", 16,
)


def dsa_fixture(root, version, visibility):
    directory = root / OASIS_FIXTURES / f"kmip-{version}"
    scenario = 12 if visibility == "Public" else 13
    name = f"mandatory/BL-M-{scenario}-{version.replace('.', '')}.xml"
    contents = (directory / name).read_bytes()
    manifest = {filename: digest for digest, filename in
                (line.split("  ", 1) for line in (directory / "CHECKSUMS.sha256").read_text().splitlines())}
    if sha256(contents).hexdigest() != manifest[name]:
        raise ValueError(f"DSA fixture checksum mismatch: {name}")
    tag = visibility + "Key"
    for payload in fromstring(contents).iter("RequestPayload"):
        obj = payload.find(tag)
        if obj is not None:
            attributes = deepcopy(payload.find("Attributes"))
            # The OASIS usage mask is preserved. Specify the algorithm and length
            # explicitly for the Register attribute requirements of the SDKs.
            attributes.append(field("CryptographicAlgorithm", "Enumeration", "DSA"))
            attributes.append(field("CryptographicLength", "Integer", 512))
            return obj, attributes
    raise ValueError(f"DSA key not found in {name}")


def dh_fixture(visibility):
    component = ("X", 5) if visibility == "Private" else ("Y", pow(2, 5, FFDHE2048))
    fields = (("P", FFDHE2048), ("Q", (FFDHE2048 - 1) // 2), ("G", 2), ("J", 2), component)
    return transparent_object_fixture(
        f"DH {visibility} Key", visibility + "Key", "DH", 2048, "KeyAgreement",
        tuple((name, "BigInteger", value.to_bytes((value.bit_length() + 8) // 8, "big").hex()) for name, value in fields),
    )


def transparent_registration_cases(root, version, base):
    names = WireValues(root, version, [])
    for family in ("DSA", "DH"):
        for visibility, object_type in (("Private", 4), ("Public", 3)):
            key = f"{family} {visibility} Key"
            case_id = f"test-register-transparent-{family.lower()}-{visibility.lower()}-key"
            obj, attributes = dsa_fixture(root, version, visibility) if family == "DSA" else dh_fixture(visibility)
            case = deepcopy(base)
            case.update(
                case_id=case_id,
                description=f"Register transparent {key} material under KMIP {version}.",
                requires={
                    "transparent_key": key,
                    "register_object_type": str(object_type),
                    "register_object_ttlv": encode(obj, names)[0].hex(),
                    "register_attributes_ttlv": encode(attributes, names)[0].hex(),
                    "controlled_expected_status": "Success",
                },
                isolation_recipe={
                    "unique_name_prefix": f"kmipkit-{case_id}",
                    "setup_steps": [],
                    "target_step": f"Register the disposable {key} material",
                    "teardown_steps": ["Destroy the object identified by a successful Register"],
                },
            )
            case["execution"]["assertions"] = [
                "register_request_contains_the_transparent_key_format_and_exact_key_components",
                "successful_register_returns_an_identifier_used_for_cleanup",
                "target_operation_status_and_reason_are_preserved",
            ]
            yield case
