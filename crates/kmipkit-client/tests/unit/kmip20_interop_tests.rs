//! Commercial and independent KMS interoperability tests for OASIS KMIP 2.0 (KMIPKIT-0070).
//!
//! Validates client behavior against the published conformance profiles for:
//! 1. Cosmian KMS
//! 2. Thales `CipherTrust` Manager
//! 3. Utimaco Enterprise Secure Key Manager (ESKM)

use kmipkit_protocol::ProtocolVersion;
use kmipkit_test_support::ExchangeScript;
use kmipkit_transport::{RequestDeliveryState, RequestOptions};
use kmipkit_ttlv::codec::CodecLimits;
use serde_json::Value as JsonValue;

use crate::asynchronous_execution_test_support::client_for;

const COSMIAN_PROFILE: &str = include_str!("../../../../interop/shared/profiles/cosmian-kms.json");
const THALES_PROFILE: &str =
    include_str!("../../../../interop/shared/profiles/thales-ciphertrust.json");
const UTIMACO_PROFILE: &str = include_str!("../../../../interop/shared/profiles/utimaco-eskm.json");

#[test]
fn test_cosmian_kms_profile_characteristics() {
    let profile: JsonValue =
        serde_json::from_str(COSMIAN_PROFILE).expect("valid Cosmian JSON profile");
    assert_eq!(profile["server"]["vendor"], "Cosmian");
    assert_eq!(profile["server"]["product"], "Cosmian KMS");
    assert_eq!(profile["transports"]["raw_tls"]["supported"], true);
    assert_eq!(profile["encodings"]["TTLV"]["supported"], true);
}

#[test]
fn test_thales_ciphertrust_profile_characteristics_and_20_support() {
    let profile: JsonValue =
        serde_json::from_str(THALES_PROFILE).expect("valid Thales JSON profile");
    assert_eq!(profile["server"]["vendor"], "Thales");
    assert_eq!(profile["server"]["product"], "CipherTrust Manager");

    let versions = profile["server"]["kmip_versions"]
        .as_array()
        .expect("kmip_versions is an array");
    assert!(
        versions.iter().any(|v| v == "2.0"),
        "Thales CipherTrust profile must advertise KMIP 2.0 support"
    );
    assert!(
        versions.iter().any(|v| v == "2.1"),
        "Thales CipherTrust profile must advertise KMIP 2.1 support"
    );
}

#[test]
fn test_utimaco_eskm_profile_characteristics_and_20_support() {
    let profile: JsonValue =
        serde_json::from_str(UTIMACO_PROFILE).expect("valid Utimaco JSON profile");
    assert_eq!(profile["server"]["vendor"], "Utimaco");
    assert_eq!(
        profile["server"]["product"],
        "Enterprise Secure Key Manager (ESKM)"
    );

    let versions = profile["server"]["kmip_versions"]
        .as_array()
        .expect("kmip_versions is an array");
    assert!(
        versions.iter().any(|v| v == "2.0"),
        "Utimaco ESKM profile must advertise KMIP 2.0 support"
    );

    assert_eq!(profile["transports"]["raw_tls"]["supported"], true);
    assert_eq!(profile["transports"]["https"]["supported"], false);
}

#[test]
fn test_interop_presend_gate_enforces_kmip20_for_commercial_profiles() {
    let (mut client, transport, _) = client_for(ExchangeScript::FailBeforeWrite);
    let opt_2_0 = RequestOptions::default().force_kmip_version(ProtocolVersion::V2_0);

    // Ping is a KMIP 2.1 operation (0x3B). When client interacts with a 2.0 endpoint,
    // it must reject locally without sending any transport frames.
    let err = client
        .ping_with_options(&CodecLimits::defaults(), &opt_2_0)
        .expect_err("Ping must be blocked locally under KMIP 2.0");

    assert_eq!(err.delivery_state(), Some(RequestDeliveryState::NotSent));
    assert_eq!(transport.borrow().exchange_count(), 0);
}
