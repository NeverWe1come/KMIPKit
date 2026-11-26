"""Select live Get probes for the active transparent RSA and EC formats."""

from copy import deepcopy


FORMATS = {
    "RSA Private Key": 10,
    "RSA Public Key": 11,
    "EC Private Key": 20,
    "EC Public Key": 21,
}


def asymmetric_key_cases(version, base):
    for name, format_code in FORMATS.items():
        case = deepcopy(base)
        case_id = f"test-op-get-transparent-{'-'.join(name.lower().split())}"
        family = name.split()[0]
        case.update(
            case_id=case_id,
            description=f"Read transparent {name} material under KMIP {version}.",
            isolation_recipe={
                **case["isolation_recipe"],
                "unique_name_prefix": f"kmipkit-{case_id}",
                "setup_steps": [f"Create an isolated extractable {family} key pair"],
                "target_step": f"Get the {name} in its transparent key format",
                "teardown_steps": ["Destroy both keys created by the fixture"],
            },
        )
        case["requires"].update(
            transparent_key=name,
            key_material_format=str(format_code),
            controlled_expected_status="Success",
        )
        case["execution"]["assertions"] = [
            "request_selects_the_transparent_key_format",
            "response_names_the_owned_key_and_correct_object_type",
            "response_contains_the_expected_key_material_components",
        ]
        yield case
