//! Policy guards and dispatch; every call to the library performs one exchange.
use crate::{adapters, driver};
use crate::models::{Case, CaseResult, Connection};
use kmipkit::protocol::{DestroyRequest, UniqueIdentifier};
use kmipkit::{Client, ClientRequest};

fn reference(case: &Case) -> String {
    case.normative_refs.first().map_or_else(|| "OASIS KMIP §6".to_owned(), |r| format!("{} §{}", r.document, r.section))
}

pub(crate) fn classify(case: &Case, connection: &Connection) -> Option<CaseResult> {
    if !case.execution.protocol_versions.contains(&connection.protocol_version) {
        return Some(CaseResult::non_run(case, "not_run", format!("not_applicable: selected version; {}", reference(case))));
    }
    if case.execution.mutation != "read_only" && !connection.execution_policy.allow_mutations {
        return Some(CaseResult::non_run(case, "policy_restricted", "mutations require explicit opt-in".to_owned()));
    }
    if connection.api_level == "generic-ttlv" && case.execution.mode != "local_model" {
        return Some(CaseResult::non_run(case, "not_run", "library_api_unavailable: generic TTLV outbound KMIP 2.0/2.1 construction; KMIPKIT-0083-FR-007".to_owned()));
    }
    if case.requires["workflow_type"] == "server_to_client" {
        return Some(CaseResult::non_run(case, "not_run", format!("incoming server channel required; {}", reference(case))));
    }
    None
}

pub(crate) fn run_case(case: &Case, connection: &Connection, client: &mut Client) -> CaseResult {
    if let Some(result) = classify(case, connection) { return result; }
    if case.execution.mode == "local_model" {
        let passed = adapters::local::verify(&case.case_id);
        let mut result = CaseResult::non_run(case, if passed { "passed" } else { "failed" }, "local model validation only; server not checked".to_owned());
        result.library_correctness = if passed { "passed" } else { "failed" };
        return result;
    }
    let name = format!("kmipkit-rust-{}", uuid::Uuid::new_v4());
    let mut owned_identifier = None;
    if case.execution.mutation == "mutating" && !["Create", "Create Key Pair", "Create Split Key", "Register", "Import", "Set Constraints", "Set Defaults", "Hash", "RNG Retrieve", "RNG Seed", "Query Asynchronous Requests", "Poll", "Process", "Cancel", "PKCS#11", "Re-Provision", "Validate"].contains(&case.operation.as_str()) {
        match adapters::create(&name).and_then(|request| driver::execute(client, connection, request).map_err(|_| "object setup failed")) {
            Ok(observed) if observed.status == 0 => owned_identifier = observed.identifier,
            _ => return CaseResult::non_run(case, "not_run", "owned object setup failed; target request not sent".to_owned()),
        }
    }
    let request = match adapters::request(case, owned_identifier.clone(), &name) {
        Ok(request) => request,
        Err(reason) => {
            cleanup(client, connection, owned_identifier.as_ref());
            return CaseResult::non_run(case, "not_run", format!("{reason}; {}", reference(case)));
        }
    };
    let start = std::time::Instant::now();
    let result = driver::execute(client, connection, request);
    let mut output = match result {
        Ok(observed) => {
            // A successful creation is owned by this attempt and must be removed.
            if owned_identifier.is_none() && ["Create", "Register", "Import"].contains(&case.operation.as_str()) { owned_identifier = observed.identifier.clone(); }
            observed_result(case, observed.status, observed.reason)
        }
        Err(error) => {
            if let Some(result) = error.server_operation_result() {
                observed_result(case, result.status().raw(), result.reason().map(kmipkit::ResultReason::raw))
            } else {
                let mut result = CaseResult::non_run(case, "failed", format!("local {:?}; delivery={:?}; request not retried", error.category(), error.delivery_state()));
                result.library_correctness = if error.category() == kmipkit::ClientErrorCategory::Protocol { "failed" } else { "not_checked" };
                result
            }
        }
    };
    output.duration_ms = start.elapsed().as_secs_f64() * 1000.0;
    if case.operation != "Destroy" { cleanup(client, connection, owned_identifier.as_ref()); }
    output
}

fn cleanup(client: &mut Client, connection: &Connection, identifier: Option<&UniqueIdentifier>) {
    if let Some(identifier) = identifier {
        // This is a distinct teardown operation, never a retry of the target.
        let _ = driver::execute(client, connection, ClientRequest::Destroy(DestroyRequest::new(Some(identifier.clone()))));
    }
}

fn observed_result(case: &Case, status: u32, reason: Option<u32>) -> CaseResult {
    let expected_failure = case.requires["expected_status"] == "OperationFailed";
    let (outcome, server) = if status == 0 && !expected_failure {
        ("passed", "supported")
    } else if status == 1 && expected_failure {
        ("passed", "supported")
    } else if status == 1 && reason == Some(5) {
        ("server_unsupported", "unsupported")
    } else if status == 1 && matches!(reason, Some(3 | 12)) {
        ("policy_restricted", "policy_restricted")
    } else { ("failed", "unknown") };
    let mut result = CaseResult::non_run(case, outcome, "typed response accepted; raw payload inspected without serialization".to_owned());
    result.library_correctness = "passed";
    result.server_compatibility = server;
    result.result_status = Some(kmipkit::ResultStatus::from_raw(status).known_name().map_or_else(|| format!("0x{status:08x}"), str::to_owned));
    result.result_reason = reason.map(|r| kmipkit::ResultReason::from_raw(r).known_name().map_or_else(|| format!("0x{r:08x}"), str::to_owned));
    result
}
