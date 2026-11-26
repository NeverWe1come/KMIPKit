"""Select AES-GCM recipes for authenticated encryption fields in the table."""

from copy import deepcopy


AAD_CASES = (
    "test-crypto-encrypt-aes-256-gcm",
    "test-crypto-decrypt-aes-256-gcm",
    "test-crypto-decrypt-aes-256-gcm-invalid-tag",
)


def authenticated_encryption_cases(version, candidates):
    for operation, invalid in (("Encrypt", False), ("Decrypt", False), ("Decrypt", True)):
        base = candidates[f"test-crypto-{operation.lower()}-aes-256-cbc"]
        case = deepcopy(base)
        case_id = base["case_id"].removesuffix("cbc") + "gcm" + ("-invalid-tag" if invalid else "")
        step = ("Reject Decrypt with a changed authentication tag" if invalid
                else f"Execute {operation} with authenticated additional data")
        case.update(
            case_id=case_id,
            description=f"{step} under KMIP {version}.",
            isolation_recipe={
                **case["isolation_recipe"],
                "unique_name_prefix": f"kmipkit-gcm-{operation.lower()}",
                "setup_steps": ["Create an isolated AES-256 key", "Activate it"]
                + (["Encrypt fixture data with additional authenticated data"] if operation == "Decrypt" else []),
                "target_step": step,
                "teardown_steps": ["Revoke and destroy the fixture key"],
            },
        )
        case["requires"]["block_cipher_mode"] = "GCM"
        case["execution"]["assertions"] = [
            "request_contains_aad_and_gcm_parameters",
            "ciphertext_and_16_byte_tag_returned" if operation == "Encrypt" else
            "tampered_authentication_tag_rejected" if invalid else "decrypt_recovers_original_plaintext",
        ]
        if invalid:
            case["requires"].update(expected_status="OperationFailed", expected_result_reasons=[10])
        yield case
