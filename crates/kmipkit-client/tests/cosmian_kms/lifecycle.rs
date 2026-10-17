//! Manifest-parity and case-specific lifecycle tests against Cosmian 5.28.0.

use std::error::Error;

const EXECUTABLE_CASE_IDS: &[&str] = &[
    "cosmian-528-add-attribute-certificate",
    "cosmian-528-add-attribute-opaqueobject",
    "cosmian-528-add-attribute-privatekey",
    "cosmian-528-add-attribute-publickey",
    "cosmian-528-add-attribute-secretdata",
    "cosmian-528-add-attribute-splitkey",
    "cosmian-528-add-attribute-symmetrickey",
    "cosmian-528-adjust-attribute-symmetrickey",
    "cosmian-528-delete-attribute-certificate",
    "cosmian-528-delete-attribute-opaqueobject",
    "cosmian-528-delete-attribute-privatekey",
    "cosmian-528-delete-attribute-publickey",
    "cosmian-528-delete-attribute-secretdata",
    "cosmian-528-delete-attribute-splitkey",
    "cosmian-528-delete-attribute-symmetrickey",
    "cosmian-528-get-attributes-certificate",
    "cosmian-528-get-attributes-opaqueobject",
    "cosmian-528-get-attributes-privatekey",
    "cosmian-528-get-attributes-publickey",
    "cosmian-528-get-attributes-secretdata",
    "cosmian-528-get-attributes-splitkey",
    "cosmian-528-get-attributes-symmetrickey",
    "cosmian-528-get-attribute-list-certificate",
    "cosmian-528-get-attribute-list-opaqueobject",
    "cosmian-528-get-attribute-list-privatekey",
    "cosmian-528-get-attribute-list-publickey",
    "cosmian-528-get-attribute-list-secretdata",
    "cosmian-528-get-attribute-list-splitkey",
    "cosmian-528-get-attribute-list-symmetrickey",
    "cosmian-528-modify-attribute-certificate",
    "cosmian-528-modify-attribute-opaqueobject",
    "cosmian-528-modify-attribute-privatekey",
    "cosmian-528-modify-attribute-publickey",
    "cosmian-528-modify-attribute-secretdata",
    "cosmian-528-modify-attribute-splitkey",
    "cosmian-528-modify-attribute-symmetrickey",
    "cosmian-528-set-attribute-certificate",
    "cosmian-528-set-attribute-opaqueobject",
    "cosmian-528-set-attribute-privatekey",
    "cosmian-528-set-attribute-publickey",
    "cosmian-528-set-attribute-secretdata",
    "cosmian-528-set-attribute-splitkey",
    "cosmian-528-set-attribute-symmetrickey",
    "cosmian-528-locate-certificate",
    "cosmian-528-locate-opaqueobject",
    "cosmian-528-locate-privatekey",
    "cosmian-528-locate-publickey",
    "cosmian-528-locate-secretdata",
    "cosmian-528-locate-splitkey",
    "cosmian-528-locate-symmetrickey",
    "cosmian-528-check-certificate",
    "cosmian-528-check-opaqueobject",
    "cosmian-528-check-privatekey",
    "cosmian-528-check-publickey",
    "cosmian-528-check-secretdata",
    "cosmian-528-check-splitkey",
    "cosmian-528-check-symmetrickey",
    "cosmian-528-destroy-certificate",
    "cosmian-528-destroy-opaqueobject",
    "cosmian-528-destroy-privatekey",
    "cosmian-528-destroy-publickey",
    "cosmian-528-destroy-secretdata",
    "cosmian-528-destroy-splitkey",
    "cosmian-528-destroy-symmetrickey",
    "cosmian-528-revoke-certificate",
    "cosmian-528-revoke-opaqueobject",
    "cosmian-528-revoke-privatekey",
    "cosmian-528-revoke-publickey",
    "cosmian-528-revoke-secretdata",
    "cosmian-528-revoke-splitkey",
    "cosmian-528-revoke-symmetrickey",
    "cosmian-528-re-certify-certificate",
    "cosmian-528-activate-certificate",
    "cosmian-528-activate-privatekey",
    "cosmian-528-activate-publickey",
    "cosmian-528-activate-secretdata",
    "cosmian-528-activate-splitkey",
    "cosmian-528-activate-symmetrickey",
    "cosmian-528-modify-attribute-symmetrickey-activation-date",
    "cosmian-528-set-attribute-symmetrickey-activation-date",
    "cosmian-528-validate-certificate-certificate-values-current",
    "cosmian-528-validate-certificate-certificate-values-explicit-time",
    "cosmian-528-validate-certificate-managed-identifiers-current",
    "cosmian-528-validate-certificate-managed-identifiers-explicit-time",
    "cosmian-528-validate-certificate-mixed-current",
    "cosmian-528-validate-certificate-mixed-explicit-time",
];

#[test]
fn cosmian_lifecycle_manifest_cases_have_typed_executors() {
    let manifest: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../tests/integration/cosmian/capability-matrix.json"
    ))
    .expect("capability manifest must be valid JSON");
    let assigned_operations = [
        "Activate",
        "Add Attribute",
        "Delete Attribute",
        "Get Attribute List",
        "Get Attributes",
        "Locate",
        "Check",
        "Destroy",
        "Revoke",
        "Modify Attribute",
        "Set Attribute",
        "Validate",
        "Adjust Attribute",
        "Re-certify",
    ];
    let manifest_cases: std::collections::BTreeSet<_> = manifest["cases"]
        .as_array()
        .expect("capability manifest contains cases")
        .iter()
        .filter(|row| row["test_family"] == "cosmian_kms::lifecycle")
        .filter(|row| assigned_operations.contains(&row["operation"].as_str().unwrap_or_default()))
        .filter_map(|row| row["case_id"].as_str())
        .collect();
    let executable_cases: std::collections::BTreeSet<_> =
        EXECUTABLE_CASE_IDS.iter().copied().collect();
    assert_eq!(
        executable_cases, manifest_cases,
        "every Task 4 lifecycle matrix case must map to exactly one typed executor"
    );
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.2, Tables 167–169.
fn cosmian_528_add_attribute_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-add-attribute-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.3, Tables 170–172; §4.59, Table 149.
fn cosmian_528_adjust_attribute_probe() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-adjust-attribute-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.13, Tables 202–204.
fn cosmian_528_delete_attribute_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-delete-attribute-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.20, Tables 223–225.
fn cosmian_528_get_attributes_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attributes-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.21, Tables 226–228.
fn cosmian_528_get_attribute_list_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-get-attribute-list-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.28, Tables 247–249.
fn cosmian_528_locate_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-locate-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.7, Tables 183–185.
fn cosmian_528_check_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-check-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.15, Tables 208–210.
fn cosmian_528_destroy_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-destroy-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_opaqueobject() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-opaqueobject")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.44, Tables 295–297.
fn cosmian_528_revoke_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-revoke-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.45, Tables 300–302.
fn cosmian_528_recertify_probe() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-re-certify-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_certificate() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-certificate")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_privatekey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-privatekey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_publickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-publickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_secretdata() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-secretdata")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_splitkey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-splitkey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.1, Tables 164–166.
fn cosmian_528_activate_symmetrickey() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-activate-symmetrickey")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.34, Tables 265–267.
fn cosmian_528_modify_attribute_symmetrickey_activation_date() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-modify-attribute-symmetrickey-activation-date")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.51, Tables 322–324.
fn cosmian_528_set_attribute_symmetrickey_activation_date() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-set-attribute-symmetrickey-activation-date")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_certificate_values_current() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-validate-certificate-certificate-values-current")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_certificate_values_explicit_time() -> Result<(), Box<dyn Error>>
{
    super::common::run_lifecycle_case(
        "cosmian-528-validate-certificate-certificate-values-explicit-time",
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_managed_identifiers_current() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case(
        "cosmian-528-validate-certificate-managed-identifiers-current",
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_managed_identifiers_explicit_time() -> Result<(), Box<dyn Error>>
{
    super::common::run_lifecycle_case(
        "cosmian-528-validate-certificate-managed-identifiers-explicit-time",
    )
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_mixed_current() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-validate-certificate-mixed-current")
}

#[test]
#[ignore = "requires the local Cosmian KMS deployment; see tests/integration/cosmian/README.md"]
// OASIS KMIP v2.1 §6.1.57, Tables 340–342.
fn cosmian_528_validate_certificate_mixed_explicit_time() -> Result<(), Box<dyn Error>> {
    super::common::run_lifecycle_case("cosmian-528-validate-certificate-mixed-explicit-time")
}
