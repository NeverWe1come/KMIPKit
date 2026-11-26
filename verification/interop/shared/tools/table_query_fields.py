"""Select Query functions that exercise individual base-object rows."""

from copy import deepcopy

QUERY_FIELDS = {
    "Server Information": (3, 0x420088),
    "Extension Information": (5, 0x4200A4),
    "RNG Parameters": (8, 0x4200D9),
    "Validation Information": (9, 0x4200DF),
    "Profile Information": (10, 0x4200EB),
    "Capability Information": (11, 0x4200F7),
}


def query_cases(version, base):
    for name, (function, tag) in QUERY_FIELDS.items():
        case = deepcopy(base)
        case_id = f"test-query-{'-'.join(name.lower().split())}"
        case.update(
            case_id=case_id,
            description=f"Query the provider for {name} and read the returned structure under KMIP {version}.",
            requires={"base_object": name, "query_function": str(function), "query_result_tag": str(tag),
                      "controlled_expected_status": "Success"},
            isolation_recipe={
                "unique_name_prefix": f"kmipkit-{case_id}", "setup_steps": [],
                "target_step": f"Execute Query for {name}", "teardown_steps": [],
            },
            execution={
                "mode": "server_exchange", "protocol_versions": [version], "mutation": "read_only",
                "status": "planned", "steps": ["execute_query", "verify_response"],
                "assertions": ["request_selects_the_query_function", "response_contains_the_requested_nonempty_structure"],
            },
        )
        yield case


def nonce_case(version, base):
    case = deepcopy(base)
    case.update(
        case_id="test-query-attestation-nonce",
        description=f"Request attestation capabilities and validate the server's Nonce under KMIP {version}.",
        requires={"base_object": "Nonce", "controlled_expected_status": "Success"},
        isolation_recipe={
            "unique_name_prefix": "kmipkit-query-attestation-nonce", "setup_steps": [],
            "target_step": "Execute Query Attestation Types with an attestation-capable header", "teardown_steps": [],
        },
        execution={
            "mode": "server_exchange", "protocol_versions": [version], "mutation": "read_only",
            "status": "planned", "steps": ["execute_query", "verify_response"],
            "assertions": ["request_indicates_attestation_capability", "request_queries_attestation_types",
                           "response_contains_nonempty_nonce_id_and_value"],
        },
    )
    return case
