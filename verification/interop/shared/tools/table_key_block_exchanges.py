"""Select Get workflows that inspect the returned Key Block and its material."""

from copy import deepcopy


TRANSPARENT_KEY = "test-op-get-transparent-symmetric-key"
WRAPPED_KEY = "test-op-get-wrapped-symmetric-key"


def key_block_cases(version, base):
    for case_id, wrapped in ((TRANSPARENT_KEY, False), (WRAPPED_KEY, True)):
        case = deepcopy(base)
        requested = "a key wrapped with AES" if wrapped else "transparent AES key material"
        case.update(
            case_id=case_id,
            description=f"Get and verify {requested} under KMIP {version}.",
            isolation_recipe={
                **case["isolation_recipe"],
                "unique_name_prefix": f"kmipkit-{case_id}",
                "setup_steps": ["Create an extractable AES-256 fixture"] + (
                    ["Create a separate AES-256 wrapping key", "Activate the wrapping key"] if wrapped else []
                ),
                "target_step": f"Get {requested}",
                "teardown_steps": (["Revoke and destroy the wrapping key"] if wrapped else [])
                + ["Destroy the extractable fixture"],
            },
        )
        case["requires"]["controlled_expected_status"] = "Success"
        case["requires"]["key_block_shape"] = "wrapped" if wrapped else "transparent"
        case["execution"]["assertions"] = [
            "request_selects_wrapping_specification" if wrapped else "request_selects_transparent_symmetric_format",
            "response_contains_matching_key_block_metadata",
            "response_contains_wrapped_key_value_and_wrapping_key_identifier" if wrapped else
            "response_contains_exactly_32_bytes_of_transparent_symmetric_key_material",
        ]
        yield case
