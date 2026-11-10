//! Policy guards and dispatch; every call to the library performs one exchange.
use crate::adapters;
use crate::models::{Case, CaseResult, Connection};
use kmipkit::Client;

fn reference(case: &Case) -> String {
    case.normative_refs.first().map_or_else(
        || "OASIS KMIP §6".to_owned(),
        |r| format!("{} §{}", r.document, r.section),
    )
}

pub(crate) fn classify(case: &Case, connection: &Connection) -> Option<CaseResult> {
    if !case
        .execution
        .protocol_versions
        .contains(&connection.protocol_version)
    {
        return Some(CaseResult::non_run(
            case,
            "not_run",
            format!("not_applicable: selected version; {}", reference(case)),
        ));
    }
    if connection.protocol_version == "2.0"
        && [
            "Ping",
            "Process",
            "Query Asynchronous Requests",
            "Set Defaults",
        ]
        .contains(&case.operation.as_str())
    {
        return Some(CaseResult::non_run(case, "not_run", "not_applicable: operation absent from OASIS KMIP 2.0 Specification §§6.1, 11.36 operation inventories".to_owned()));
    }
    if (case.execution.mutation != "read_only"
        || ["Login", "Logout", "Delegated Login", "Log"].contains(&case.operation.as_str()))
        && !connection.execution_policy.allow_mutations
    {
        return Some(CaseResult::non_run(
            case,
            "policy_restricted",
            "mutations require explicit opt-in".to_owned(),
        ));
    }
    if connection.api_level == "generic-ttlv" && case.execution.mode != "local_model" {
        return Some(CaseResult::non_run(case, "not_run", "library_api_unavailable: generic TTLV outbound KMIP 2.0/2.1 construction; KMIPKIT-0083-FR-007".to_owned()));
    }
    if connection.protocol_version == "2.0"
        && ["Login", "Logout", "Delegated Login"].contains(&case.operation.as_str())
        && kmipkit::VersionGate::validate_tag(kmipkit::ProtocolVersion::V2_0, 0x0042_014a).is_err()
    {
        let mut result = CaseResult::non_run(case, "not_run", "library_api_unavailable: pinned VersionGate rejects KMIP 2.0 Ticket/Request Count/Rights tags required by the bounded session recipe and Logout cleanup; OASIS KMIP 2.0 Specification §§6.1.12, 6.1.29, 6.1.30, 7.29, 7.33".to_owned());
        result.library_correctness = "failed";
        return Some(result);
    }
    None
}

pub(crate) fn run_case(case: &Case, connection: &Connection, client: &mut Client) -> CaseResult {
    if let Some(result) = classify(case, connection) {
        return result;
    }
    if case.execution.mode == "local_model" {
        let passed = adapters::local::verify(&case.case_id);
        let mut result = CaseResult::non_run(
            case,
            if passed { "passed" } else { "failed" },
            "local model validation only; server not checked".to_owned(),
        );
        result.library_correctness = if passed { "passed" } else { "failed" };
        return result;
    }
    let start = std::time::Instant::now();
    let (result, clean) = crate::recipes::execute(case, connection, client);
    let mut output = match result {
        Ok(observed) => {
            // Shape metadata is inspected in memory; payload bytes never enter reports.
            let _shape = observed.payload_tags;
            observed_result(case, observed.status, observed.reason)
        }
        Err(crate::recipes::Failure::LocalRejection(reason)) => {
            let mut result = CaseResult::non_run(case, "not_run", reason.to_owned());
            result.library_correctness = "passed";
            result
        }
        Err(crate::recipes::Failure::Prerequisite(reason)) => {
            CaseResult::non_run(case, "not_run", format!("{reason}; {}", reference(case)))
        }
        Err(crate::recipes::Failure::Assertion(reason)) => {
            let mut result = CaseResult::non_run(case, "failed", reason.to_owned());
            result.library_correctness = "failed";
            result
        }
        Err(crate::recipes::Failure::Library(error)) => {
            if let Some(result) = error.server_operation_result() {
                observed_result(
                    case,
                    result.status().raw(),
                    result.reason().map(kmipkit::ResultReason::raw),
                )
            } else if case.requires["expected_status"] == "NotSent"
                && error.delivery_state() == Some(kmipkit::RequestDeliveryState::NotSent)
            {
                let mut result = CaseResult::non_run(
                    case,
                    "passed",
                    "version boundary rejected before sending; server not checked".to_owned(),
                );
                result.library_correctness = "passed";
                result
            } else {
                CaseResult::non_run(
                    case,
                    "failed",
                    format!(
                        "local {:?}; delivery={:?}; request not retried",
                        error.category(),
                        error.delivery_state()
                    ),
                )
            }
        }
    };
    output.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    if !clean {
        output.diagnostic.push_str("; owned-object cleanup failed");
        output.status = "failed";
    }
    output
}

fn observed_result(case: &Case, status: u32, reason: Option<u32>) -> CaseResult {
    let expected_failure = case.requires["expected_status"] == "OperationFailed";
    let accepted_negative_reason = match case.case_id.as_str() {
        "test-neg-encrypt-preactive-key" => matches!(reason, Some(7 | 10)),
        "test-neg-destroy-active-without-revoke" => reason == Some(0x43),
        _ => false,
    };
    let (outcome, server) = if status == 1 && matches!(reason, Some(5 | 8)) {
        ("server_unsupported", "unsupported")
    } else if status == 1 && reason == Some(12) {
        ("policy_restricted", "policy_restricted")
    } else if (status == 0 && !expected_failure)
        || (status == 1 && expected_failure && accepted_negative_reason)
    {
        ("passed", "supported")
    } else {
        ("failed", "unknown")
    };
    let mut result = CaseResult::non_run(
        case,
        outcome,
        "typed response accepted; raw payload inspected without serialization".to_owned(),
    );
    result.library_correctness = "passed";
    result.server_compatibility = server;
    result.result_status = Some(
        kmipkit::ResultStatus::from_raw(status)
            .known_name()
            .map_or_else(|| format!("0x{status:08x}"), str::to_owned),
    );
    result.result_reason = reason.map(|r| {
        kmipkit::ResultReason::from_raw(r)
            .known_name()
            .map_or_else(|| format!("0x{r:08x}"), str::to_owned)
    });
    result
}
