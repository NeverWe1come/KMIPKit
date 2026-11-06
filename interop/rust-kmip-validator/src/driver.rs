use crate::isolation::AttemptContext;
use crate::models::TestCase;

pub trait KmipDriver: Send + Sync {
    fn execute_case(
        &self,
        test_case: &TestCase,
        attempt: &AttemptContext,
    ) -> Result<ExecutionOutcome, DriverError>;
}

#[derive(Debug, Clone)]
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
