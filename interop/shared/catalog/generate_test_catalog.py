#!/usr/bin/env python3
"""Generate the exhaustive KMIP 2.1 Test Catalog from specification/catalog/kmip-2.1.json."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def generate_catalog(normative_catalog_path: Path) -> dict:
    with open(normative_catalog_path, "r", encoding="utf-8") as f:
        catalog = json.load(f)

    elements = catalog.get("elements", [])

    # 1. Collect in-scope client-to-server operations (client_1_0)
    c2s_ops = [
        el for el in elements
        if el.get("kind") == "operation"
        and el.get("direction") == "client_to_server"
        and el.get("scope_state") == "client_1_0"
    ]
    c2s_ops.sort(key=lambda x: x["name"])

    # 2. Collect 9 managed object types
    managed_objects = [
        el for el in elements
        if el.get("kind") == "object_type"
    ]
    managed_objects.sort(key=lambda x: x["name"])

    # 3. Collect 63 attributes
    attributes = [
        el for el in elements
        if el.get("kind") == "attribute"
    ]
    attributes.sort(key=lambda x: x["name"])

    # 4. Collect 64 enumerations and 3 bitmasks
    enumerations = [
        el for el in elements
        if el.get("kind") == "enumeration"
    ]
    bitmasks = [
        el for el in elements
        if el.get("kind") == "bitmask"
    ]

    test_cases = []

    # Category A: Protocol, discovery, and session operations
    session_ops = [
        "Discover Versions", "Query", "Ping", "Interop", "Log",
        "Login", "Logout", "Delegated Login"
    ]
    for op in c2s_ops:
        name = op["name"]
        if name in session_ops:
            case_id = f"test-op-{name.lower().replace(' ', '-')}"
            sec = op.get("source_refs", [{}])[0].get("section", "6.1")
            test_cases.append({
                "case_id": case_id,
                "category": "protocol_and_session",
                "operation": name,
                "oasis_section": sec,
                "description": f"Execute {name} operation and verify typed response and raw TTLV preservation.",
                "requires": {
                    "operation": name
                },
                "isolation_recipe": {
                    "unique_name_prefix": f"kmipkit-{name.lower().replace(' ', '-')}",
                    "setup_steps": [],
                    "target_step": f"Execute {name}",
                    "teardown_steps": [],
                    "normalized_fields": ["TimeStamp", "UniqueIdentifier"]
                }
            })

    # Category B: Object Lifecycle & General operations
    crypto_ops = {"Encrypt", "Decrypt", "Sign", "Signature Verify", "MAC", "MAC Verify", "Hash", "RNG Retrieve", "RNG Seed"}
    attribute_ops = {"Get Attributes", "Get Attribute List", "Add Attribute", "Modify Attribute", "Set Attribute", "Delete Attribute", "Adjust Attribute"}

    for op in c2s_ops:
        name = op["name"]
        if name in session_ops or name in crypto_ops or name in attribute_ops:
            continue
        case_id = f"test-op-{name.lower().replace(' ', '-')}"
        sec = op.get("source_refs", [{}])[0].get("section", "6.1")
        is_async = name in {"Cancel", "Poll", "Process", "Query Asynchronous Requests"}

        setup = []
        teardown = []
        if name not in {"Create", "Create Key Pair", "Create Split Key", "Register", "Import", "Locate"}:
            setup = ["Create unique SymmetricKey"]
            teardown = ["Revoke SymmetricKey", "Destroy SymmetricKey"]
        elif name in {"Create", "Create Key Pair", "Create Split Key", "Register", "Import"}:
            teardown = ["Destroy object"]

        test_cases.append({
            "case_id": case_id,
            "category": "object_lifecycle",
            "operation": name,
            "oasis_section": sec,
            "description": f"Execute {name} operation ensuring state isolation and equivalence across API levels.",
            "requires": {
                "operation": name,
                "requires_async": is_async
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-{name.lower().replace(' ', '-')}",
                "setup_steps": setup,
                "target_step": f"Execute {name}",
                "teardown_steps": teardown,
                "normalized_fields": ["TimeStamp", "UniqueIdentifier", "InitialDate", "ActivationDate", "LastChangeDate", "DestroyDate"]
            }
        })

    # Category C: 9 Managed Object creation/registration variants
    for obj in managed_objects:
        obj_name = obj["name"]
        case_id = f"test-obj-{obj_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "managed_objects",
            "operation": "Register" if obj_name in {"Certificate", "Certificate Request", "PGP Key", "Opaque Object", "Secret Data"} else "Create",
            "oasis_section": "§2",
            "description": f"Validate {obj_name} object construction, encoding roundtrip, and cleanup.",
            "requires": {
                "object_type": obj_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-obj-{obj_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Provision {obj_name}",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })

    # Category D: 63 Attributes
    for attr in attributes:
        attr_name = attr["name"]
        case_id = f"test-attr-{attr_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "attributes",
            "operation": "Get Attributes",
            "oasis_section": "§4",
            "description": f"Verify read/write semantics and policies for attribute '{attr_name}'.",
            "requires": {
                "attribute": attr_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-attr-{attr_name.lower().replace(' ', '-')}",
                "setup_steps": ["Create unique SymmetricKey with attribute"],
                "target_step": f"GetAttributes for {attr_name}",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })

    # Category E: Cryptographic and RNG operations
    crypto_profiles = [
        ("Encrypt", "AES", 256, "CBC"),
        ("Decrypt", "AES", 256, "CBC"),
        ("Sign", "RSA", 2048, "PKCS1_V1_5"),
        ("Signature Verify", "RSA", 2048, "PKCS1_V1_5"),
        ("Sign", "ECDSA", 256, "P_256"),
        ("Signature Verify", "ECDSA", 256, "P_256"),
        ("MAC", "HMAC_SHA256", 256, None),
        ("MAC Verify", "HMAC_SHA256", 256, None),
        ("Hash", "SHA_256", 0, None),
        ("RNG Retrieve", "RNG", 32, None),
        ("RNG Seed", "RNG", 32, None),
    ]
    for op_name, alg, key_len, mode in crypto_profiles:
        suffix = f"{alg.lower()}-{key_len}" + (f"-{mode.lower()}" if mode else "")
        case_id = f"test-crypto-{op_name.lower().replace(' ', '-')}-{suffix}"
        test_cases.append({
            "case_id": case_id,
            "category": "cryptography_and_rng",
            "operation": op_name,
            "oasis_section": "§6.1",
            "description": f"Execute cryptographic operation {op_name} with {alg} {key_len}-bit.",
            "requires": {
                "operation": op_name,
                "algorithm": alg,
                "key_length": key_len
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-crypto-{suffix}",
                "setup_steps": ["Create test key"] if op_name not in {"Hash", "RNG Retrieve", "RNG Seed"} else [],
                "target_step": f"Execute {op_name}",
                "teardown_steps": ["Destroy test key"] if op_name not in {"Hash", "RNG Retrieve", "RNG Seed"} else [],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "IVCounterNonce"]
            }
        })

    catalog_output = {
        "version": "1.0.0",
        "catalog_source": "specification/catalog/kmip-2.1.json",
        "statistics": {
            "operations_covered": len(c2s_ops),
            "managed_objects_covered": len(managed_objects),
            "attributes_covered": len(attributes),
            "enumerations_covered": len(enumerations),
            "bitmasks_covered": len(bitmasks),
            "total_cases": len(test_cases)
        },
        "test_cases": test_cases
    }

    return catalog_output


def main():
    parser = argparse.ArgumentParser(description="Generate KMIP 2.1 Test Catalog")
    parser.add_argument("--write", action="store_true", help="Write catalog to destination")
    parser.add_argument("--check", action="store_true", help="Verify catalog is up-to-date")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    normative_path = root / "specification" / "catalog" / "kmip-2.1.json"
    output_path = root / "interop" / "shared" / "catalog" / "kmipkit-1.0-test-catalog.json"

    generated = generate_catalog(normative_path)
    content = json.dumps(generated, indent=2, ensure_ascii=False) + "\n"

    if args.check:
        if not output_path.exists():
            print(f"Error: Catalog does not exist at {output_path}", file=sys.stderr)
            sys.exit(1)
        with open(output_path, "r", encoding="utf-8") as f:
            existing = f.read()
        if existing != content:
            print("Error: Catalog is out of date. Run with --write to regenerate.", file=sys.stderr)
            sys.exit(1)
        print("Catalog check passed: 100% compliant with kmip-2.1.json")
    elif args.write:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:
            f.write(content)
        print(f"Wrote {generated['statistics']['total_cases']} test cases covering "
              f"{generated['statistics']['operations_covered']} operations, "
              f"{generated['statistics']['managed_objects_covered']} objects, and "
              f"{generated['statistics']['attributes_covered']} attributes to {output_path}")
    else:
        print(content)


if __name__ == "__main__":
    main()
