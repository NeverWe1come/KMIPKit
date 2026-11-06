use crate::isolation::AttemptContext;
use crate::models::TestCase;
use serde::{Deserialize, Serialize};

pub trait KmipDriver: Send + Sync {
    fn execute_case(
        &self,
        test_case: &TestCase,
        attempt: &AttemptContext,
    ) -> Result<ExecutionOutcome, DriverError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionOutcome {
    pub success: bool,
    pub raw_response_received: bool,
    pub result_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DriverError {
    pub message: String,
    pub result_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NormalizedResponse {
    pub status: String,
    pub result_reason: Option<String>,
    pub object_type: Option<String>,
    pub sanitized_payload_hex: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EquivalenceResult {
    pub is_equivalent: bool,
    pub discrepancies: Vec<String>,
}

pub struct EquivalenceVerifier;

impl EquivalenceVerifier {
    pub fn verify_equivalence(responses: &[(&str, &NormalizedResponse)]) -> EquivalenceResult {
        let mut discrepancies = Vec::new();
        if responses.is_empty() {
            return EquivalenceResult {
                is_equivalent: true,
                discrepancies,
            };
        }
        let (base_label, baseline) = responses[0];
        for (label, resp) in &responses[1..] {
            if resp.status != baseline.status {
                discrepancies.push(format!(
                    "[{label}] status '{}' does not match [{base_label}] '{}'",
                    resp.status, baseline.status
                ));
            }
            if resp.result_reason != baseline.result_reason {
                discrepancies.push(format!(
                    "[{label}] result_reason '{:?}' does not match [{base_label}] '{:?}'",
                    resp.result_reason, baseline.result_reason
                ));
            }
            if resp.object_type != baseline.object_type {
                discrepancies.push(format!(
                    "[{label}] object_type '{:?}' does not match [{base_label}] '{:?}'",
                    resp.object_type, baseline.object_type
                ));
            }
            if resp.sanitized_payload_hex != baseline.sanitized_payload_hex {
                discrepancies.push(format!(
                    "[{label}] sanitized payload differs from [{base_label}]"
                ));
            }
        }
        EquivalenceResult {
            is_equivalent: discrepancies.is_empty(),
            discrepancies,
        }
    }
}

pub struct RecipeRunner;

impl RecipeRunner {
    pub fn execute_with_guaranteed_teardown<FSetup, FTarget, FTeardown, T>(
        setup: FSetup,
        target: FTarget,
        teardown: FTeardown,
    ) -> Result<T, String>
    where
        FSetup: FnOnce() -> Result<(), String>,
        FTarget: FnOnce() -> Result<T, String>,
        FTeardown: FnOnce() -> Result<(), String>,
    {
        setup()?;

        struct TeardownGuard<F: FnOnce() -> Result<(), String>>(Option<F>);
        impl<F: FnOnce() -> Result<(), String>> Drop for TeardownGuard<F> {
            fn drop(&mut self) {
                if let Some(td) = self.0.take() {
                    let _ = td();
                }
            }
        }

        let guard = TeardownGuard(Some(teardown));
        let target_res = target();
        drop(guard);

        target_res
    }
}

/// Simulated mock driver for independent testing without requiring live KMIPKit bindings
pub struct MockKmipDriver;

impl KmipDriver for MockKmipDriver {
    fn execute_case(
        &self,
        _test_case: &TestCase,
        _attempt: &AttemptContext,
    ) -> Result<ExecutionOutcome, DriverError> {
        Ok(ExecutionOutcome {
            success: true,
            raw_response_received: true,
            result_reason: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_equivalence_verifier_detects_matches_and_mismatches() {
        let r_high = NormalizedResponse {
            status: "Success".to_string(),
            result_reason: None,
            object_type: Some("SymmetricKey".to_string()),
            sanitized_payload_hex: Some("deadbeef".to_string()),
        };
        let r_typed = NormalizedResponse {
            status: "Success".to_string(),
            result_reason: None,
            object_type: Some("SymmetricKey".to_string()),
            sanitized_payload_hex: Some("deadbeef".to_string()),
        };
        let r_diff = NormalizedResponse {
            status: "OperationFailed".to_string(),
            result_reason: Some("GeneralFailure".to_string()),
            object_type: Some("SymmetricKey".to_string()),
            sanitized_payload_hex: Some("deadbeef".to_string()),
        };

        let ok_match =
            EquivalenceVerifier::verify_equivalence(&[("HighLevel", &r_high), ("Typed", &r_typed)]);
        assert!(ok_match.is_equivalent);
        assert!(ok_match.discrepancies.is_empty());

        let mismatch = EquivalenceVerifier::verify_equivalence(&[
            ("HighLevel", &r_high),
            ("LowLevel", &r_diff),
        ]);
        assert!(!mismatch.is_equivalent);
        assert_eq!(mismatch.discrepancies.len(), 2);
    }

    #[test]
    fn test_recipe_runner_guarantees_teardown_on_failure() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc;

        let teardown_called = Arc::new(AtomicBool::new(false));
        let td_clone = teardown_called.clone();

        let res: Result<(), String> = RecipeRunner::execute_with_guaranteed_teardown(
            || Ok(()),
            || Err("target failed".to_string()),
            move || {
                td_clone.store(true, Ordering::SeqCst);
                Ok(())
            },
        );

        assert!(res.is_err());
        assert!(teardown_called.load(Ordering::SeqCst));
    }
}
