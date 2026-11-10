#!/usr/bin/env python3
"""Generate versioned executable interop cases from the normative KMIP catalog."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


KMIP_21_ONLY_OPERATIONS = {
    "Get Constraints": "6.1.22",
    "Set Constraints": "6.1.52",
}


def add_execution_contract(test_case: dict) -> dict:
    """Attach a machine-readable baseline execution recipe to a catalog case."""
    category = test_case["category"]
    req = test_case.get("requires", {})
    operation = test_case.get("operation")
    if category == "version_boundaries":
        versions = ["2.0"]
    elif operation in KMIP_21_ONLY_OPERATIONS:
        versions = ["2.1"]
    else:
        versions = ["2.0", "2.1"]

    if category == "enums_and_bitmasks":
        mode = "local_model"
        steps = ["construct_known_value", "construct_unknown_value", "ttlv_roundtrip"]
        assertions = ["known_value_preserved", "unknown_value_preserved"]
        mutation = "read_only"
    elif category == "negative_and_boundary" and req.get("expected_status"):
        mode = "server_exchange"
        steps = ["prepare_isolated_fixture", "execute_operation", "verify_response"]
        assertions = ["result_status_matches_expected", "delivery_state_recorded"]
        mutation = "mutating"
    elif category in {"attributes", "managed_objects", "object_lifecycle", "cryptography_and_rng", "multipart_crypto", "batch_workflows", "asynchronous_workflows"}:
        mode = "server_exchange"
        steps = ["prepare_isolated_fixture", "execute_operation", "verify_response", "cleanup_fixture"]
        assertions = ["response_matches_operation_contract", "state_effect_matches_expectation"]
        mutation = "mutating"
    else:
        mode = "server_exchange"
        steps = ["execute_operation", "verify_response"]
        assertions = ["response_matches_operation_contract"]
        mutation = "read_only"

    section = str(test_case.get("oasis_section", "6.1")).replace("§", "")
    test_case["normative_refs"] = [{
        "document": "OASIS KMIP 2.1 Specification",
        "section": section,
    }]
    if category == "version_boundaries":
        test_case["normative_refs"].append({
            "document": "OASIS KMIP 2.0 Specification",
            "section": "6",
        })
    test_case["execution"] = {
        "mode": mode,
        "protocol_versions": versions,
        "steps": steps,
        "assertions": assertions,
        "mutation": mutation,
        "status": "planned",
    }
    return test_case


def generate_catalog(normative_catalog_path: Path) -> dict:
    with open(normative_catalog_path, "r", encoding="utf-8") as f:  # NOSONAR
        catalog = json.load(f)

    elements = catalog.get("elements", [])

    # 1. Collect in-scope client-to-server operations.
    c2s_ops = [
        el for el in elements
        if el.get("kind") == "operation"
        and el.get("direction") == "client_to_server"
        and el.get("scope_state") == "client_1_0"
    ]
    c2s_ops.sort(key=lambda x: x["name"])
    s2c_ops = [
        el for el in elements
        if el.get("kind") == "operation"
        and el.get("direction") == "server_to_client"
    ]
    s2c_ops.sort(key=lambda x: x["name"])

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
    enumerations.sort(key=lambda x: x["name"])
    bitmasks = [
        el for el in elements
        if el.get("kind") == "bitmask"
    ]
    bitmasks.sort(key=lambda x: x["name"])

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

    # Category A2: server-to-client operations and endpoint role inversion.
    for op in s2c_ops:
        name = op["name"]
        section = op.get("source_refs", [{}])[0].get("section", "6.2")
        test_cases.append({
            "case_id": f"test-s2c-{name.lower().replace(' ', '-')}",
            "category": "server_initiated_operations",
            "operation": name,
            "oasis_section": section,
            "description": f"Accept, validate, and respond to server-initiated {name} requests.",
            "requires": {"operation": name, "workflow_type": "server_to_client"},
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-s2c-{name.lower().replace(' ', '-')}",
                "setup_steps": ["Start client listener"],
                "target_step": f"Deliver server-initiated {name}",
                "teardown_steps": ["Stop client listener"],
                "normalized_fields": ["ProtocolVersion", "Operation", "ResultStatus", "ResultReason"],
            },
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

    # Category F: 64 Enumerations and 3 Bitmasks
    for enum_el in enumerations:
        enum_name = enum_el["name"]
        case_id = f"test-enum-{enum_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "enums_and_bitmasks",
            "operation": "Query",
            "oasis_section": "§9.1",
            "description": f"Verify roundtrip and custom vendor extension preservation for enumeration {enum_name}.",
            "requires": {
                "operation": "Query",
                "enum_type": enum_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-enum-{enum_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Validate {enum_name} serialization",
                "teardown_steps": [],
                "normalized_fields": ["TimeStamp"]
            }
        })

    for mask_el in bitmasks:
        mask_name = mask_el["name"]
        case_id = f"test-bitmask-{mask_name.lower().replace(' ', '-')}"
        test_cases.append({
            "case_id": case_id,
            "category": "enums_and_bitmasks",
            "operation": "Query",
            "oasis_section": "§9.1",
            "description": f"Verify bitwise composition and unknown bit preservation for bitmask {mask_name}.",
            "requires": {
                "operation": "Query",
                "bitmask_type": mask_name
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-mask-{mask_name.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Validate {mask_name} bits",
                "teardown_steps": [],
                "normalized_fields": ["TimeStamp"]
            }
        })

    # Category G: Multi-item Batch Workflows with IDPlaceholder
    batch_workflows = [
        {
            "id": "test-batch-create-activate-destroy",
            "desc": "Single-message batch workflow: Create -> Activate (IDPlaceholder) -> GetAttributes -> Destroy",
            "recipe_prefix": "kmipkit-batch-cad",
            "steps": ["Create AES-256 SymmetricKey", "Activate with IDPlaceholder", "GetAttributes with IDPlaceholder", "Destroy with IDPlaceholder"]
        },
        {
            "id": "test-batch-register-encrypt-decrypt-destroy",
            "desc": "Single-message batch workflow: Register -> Activate -> Encrypt -> Decrypt -> Destroy",
            "recipe_prefix": "kmipkit-batch-crypto",
            "steps": ["Register AES-128 SymmetricKey", "Activate", "Encrypt", "Decrypt", "Destroy"]
        }
    ]
    for bw in batch_workflows:
        test_cases.append({
            "case_id": bw["id"],
            "category": "batch_workflows",
            "operation": "Create",
            "oasis_section": "§5.1",
            "description": bw["desc"],
            "requires": {
                "operation": "Create",
                "workflow_type": "batch_id_placeholder"
            },
            "isolation_recipe": {
                "unique_name_prefix": bw["recipe_prefix"],
                "setup_steps": [],
                "target_step": "Execute Batch Request",
                "teardown_steps": ["Destroy object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "InitialDate", "ActivationDate", "DestroyDate"]
            }
        })

    # Category H: Multipart Streaming Cryptographic Operations
    multipart_crypto = [
        ("test-multipart-encrypt-aes-cbc", "Encrypt", "AES", 128, "CBC", "Init -> Update(64B) -> Update(64B) -> Final Encrypt streaming with CorrelationValue"),
        ("test-multipart-decrypt-aes-cbc", "Decrypt", "AES", 128, "CBC", "Init -> Update(64B) -> Update(64B) -> Final Decrypt streaming with CorrelationValue"),
        ("test-multipart-sign-rsa", "Sign", "RSA", 2048, "PKCS1_V1_5", "Init -> Update -> Final Sign streaming with CorrelationValue"),
        ("test-multipart-hash-sha256", "Hash", "SHA_256", 0, None, "Init -> Update -> Final Hash streaming with CorrelationValue")
    ]
    for cid, op, alg, klen, mode, desc in multipart_crypto:
        test_cases.append({
            "case_id": cid,
            "category": "multipart_crypto",
            "operation": op,
            "oasis_section": "§6.1",
            "description": desc,
            "requires": {
                "operation": op,
                "algorithm": alg,
                "key_length": klen,
                "workflow_type": "multipart_streaming"
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-stream-{alg.lower()}",
                "setup_steps": ["Create test key"] if op != "Hash" else [],
                "target_step": f"Execute Multipart {op}",
                "teardown_steps": ["Destroy test key"] if op != "Hash" else [],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp", "CorrelationValue", "IVCounterNonce"]
            }
        })

    # Category I: Asynchronous Lifecycle Workflows
    test_cases.append({
        "case_id": "test-async-lifecycle-poll",
        "category": "asynchronous_workflows",
        "operation": "Create",
        "oasis_section": "§6.1.18",
        "description": "Send asynchronous request with AsynchronousIndicator=true, receive Pending status, query and poll until completion.",
        "requires": {
            "operation": "Create",
            "requires_async": True,
            "workflow_type": "async_poll"
        },
        "isolation_recipe": {
            "unique_name_prefix": "kmipkit-async-poll",
            "setup_steps": [],
            "target_step": "Submit Asynchronous Create and Poll",
            "teardown_steps": ["Destroy object"],
            "normalized_fields": ["UniqueIdentifier", "TimeStamp", "AsynchronousCorrelationValue"]
        }
    })

    # Category J: Negative and Boundary Tests
    negative_tests = [
        {
            "id": "test-neg-encrypt-preactive-key",
            "op": "Encrypt",
            "desc": "Verify server rejects Encrypt on Pre-Active SymmetricKey with CryptographicFailure or InvalidField.",
            "exp_status": "OperationFailed"
        },
        {
            "id": "test-neg-modify-readonly-attribute",
            "op": "Modify Attribute",
            "desc": "Verify server rejects mutation of read-only server-generated attribute (InitialDate / UniqueIdentifier).",
            "exp_status": "OperationFailed"
        },
        {
            "id": "test-neg-destroy-active-without-revoke",
            "op": "Destroy",
            "desc": "Verify server enforces KMIP §3 lifecycle rules when attempting Destroy on Active object without prior Revocation.",
            "exp_status": "OperationFailed"
        }
    ]
    for nt in negative_tests:
        test_cases.append({
            "case_id": nt["id"],
            "category": "negative_and_boundary",
            "operation": nt["op"],
            "oasis_section": "§3",
            "description": nt["desc"],
            "requires": {
                "operation": nt["op"],
                "expected_status": nt["exp_status"]
            },
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-neg-{nt['id']}",
                "setup_steps": ["Create test object in initial state"],
                "target_step": f"Attempt illegal {nt['op']}",
                "teardown_steps": ["Revoke and Destroy test object"],
                "normalized_fields": ["UniqueIdentifier", "TimeStamp"]
            }
        })

    # KMIP 2.1-only operations must be rejected locally when the effective
    # request version is 2.0. These cases do not contact a server.
    for operation in sorted(KMIP_21_ONLY_OPERATIONS):
        test_cases.append({
            "case_id": f"test-v20-reject-{operation.lower().replace(' ', '-')}",
            "category": "version_boundaries",
            "operation": operation,
            "oasis_section": KMIP_21_ONLY_OPERATIONS[operation],
            "description": f"Reject KMIP 2.1-only {operation} before sending under KMIP 2.0.",
            "requires": {"operation": operation, "expected_status": "NotSent"},
            "isolation_recipe": {
                "unique_name_prefix": f"kmipkit-v20-{operation.lower().replace(' ', '-')}",
                "setup_steps": [],
                "target_step": f"Attempt {operation} under KMIP 2.0",
                "teardown_steps": [],
                "normalized_fields": ["RequestDeliveryState"],
            },
        })

    test_cases = [add_execution_contract(case) for case in test_cases]

    catalog_output = {
        "version": "2.0.0",
        "catalog_source": "specification/catalog/kmip-2.1.json",
        "target_versions": ["2.0", "2.1"],
        "capability_policy": "probe_all_applicable",
        "execution_policy": {
            "requires_dedicated_test_environment": True,
            "mutating_cases_require_explicit_enablement": True,
            "never_retry_requests": True,
        },
        "statistics": {
            "operations_covered": len(c2s_ops),
            "server_to_client_operations_covered": len(s2c_ops),
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
    parser = argparse.ArgumentParser(description="Generate versioned KMIP 2.0/2.1 interoperability catalog")
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
        with open(output_path, "r", encoding="utf-8") as f:  # NOSONAR
            existing = f.read()
        if existing != content:
            print("Error: Catalog is out of date. Run with --write to regenerate.", file=sys.stderr)
            sys.exit(1)
        print("Catalog check passed: 100% compliant with kmip-2.1.json")
    elif args.write:
        output_path.parent.mkdir(parents=True, exist_ok=True)
        with open(output_path, "w", encoding="utf-8") as f:  # NOSONAR
            f.write(content)
        print(f"Wrote {generated['statistics']['total_cases']} test cases covering "
              f"{generated['statistics']['operations_covered']} operations, "
              f"{generated['statistics']['managed_objects_covered']} objects, "
              f"{generated['statistics']['attributes_covered']} attributes, and "
              f"{generated['statistics']['enumerations_covered']} enumerations to {output_path}")
    else:
        print(content)


if __name__ == "__main__":
    main()
