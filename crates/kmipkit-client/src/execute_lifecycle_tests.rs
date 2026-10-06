//! Private candidate tests for the `Client::execute` lifecycle boundary.
//!
//! These are derived project-policy tests, not official OASIS vectors. The
//! Pending Asynchronous Correlation Value is required immediately by OASIS
//! KMIP v2.1 §8.6, Table 399, catalog element
//! `KMIPKIT-ELEM-MESSAGE-FIELD-8-6-ASYNCHRONOUS-CORRELATION-VALUE`; §9.19 and
//! `KMIPKIT-REQ-SPEC-9.19-002` concern its later use by Poll. The Discover
//! Versions fixture has no secret payload and does not resolve ADR-0012 OD-006,
//! which remains a gate for the first secret-bearing operation.
//!
//! Traceability: `KMIPKIT-0007-FR-005`, `-FR-008`, `-FR-011`, `-FR-012`,
//! `-FR-013`, `-FR-017`; `KMIPKIT-0007-SC-002`, `-SC-004`, `-SC-005`,
//! `-SC-008`; ADR-0012; `AGENTS.md` §8.

use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::fmt::Write as _;
use std::ops::Range;
use std::rc::Rc;

use kmipkit_transport::RequestDeliveryState;
use zeroize::Zeroize;

const CORRELATION_VALUE: &[u8] = b"KMIP_ASYNC_CORRELATION_SENTINEL_73";
const CORRELATION_PREFIX: &str = "KMIP_ASYNC_CORRELATION";
const NON_SECRET_DISCOVER_VERSIONS_REQUEST: &[u8] = b"DiscoverVersions KMIP/2.1";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FailurePoint {
    BeforeWrite,
    AfterPartialWrite,
    AfterResponseStarted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Script {
    SuccessAfterPartialWrites,
    Failure(FailurePoint),
}

#[derive(Clone, Default)]
struct CleanupObservation {
    dropped: Rc<Cell<bool>>,
    initialized_range_was_zero: Rc<Cell<bool>>,
}

impl CleanupObservation {
    /// Records only whether initialized bytes are zero; it never retains or
    /// copies the slice and cannot panic while called from `Drop`.
    fn record_initialized_range(&self, initialized: &[u8]) {
        let all_zero = initialized.iter().all(|byte| *byte == 0);
        self.initialized_range_was_zero.set(all_zero);
        self.dropped.set(true);
    }
}

struct RequestOwner {
    encoded: Vec<u8>,
    observation: CleanupObservation,
}

impl RequestOwner {
    fn discover_versions(observation: CleanupObservation) -> Self {
        Self {
            encoded: NON_SECRET_DISCOVER_VERSIONS_REQUEST.to_vec(),
            observation,
        }
    }
}

impl Drop for RequestOwner {
    fn drop(&mut self) {
        self.encoded.as_mut_slice().zeroize();
        self.observation
            .record_initialized_range(self.encoded.as_slice());
    }
}

struct ResponseOwner {
    bytes: Vec<u8>,
    observation: CleanupObservation,
}

impl ResponseOwner {
    fn new(bytes: Vec<u8>, observation: CleanupObservation) -> Self {
        Self { bytes, observation }
    }
}

impl Drop for ResponseOwner {
    fn drop(&mut self) {
        // Deliberately incomplete Red candidate: Green must zeroize this
        // allocation before invoking the boolean-only observer.
        self.observation.record_initialized_range(&self.bytes);
    }
}

struct FakeObservations {
    exchange_calls: Cell<usize>,
    write_calls: Cell<usize>,
    bytes_written: Cell<usize>,
    owner_was_dropped_at_return: Cell<bool>,
    returned_response_address: Cell<Option<usize>>,
    log_saw_correlation_prefix: Cell<bool>,
}

struct CapturedLog<'a> {
    saw_correlation_prefix: &'a Cell<bool>,
}

impl fmt::Write for CapturedLog<'_> {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        if text.contains(CORRELATION_PREFIX) {
            self.saw_correlation_prefix.set(true);
        }
        Ok(())
    }
}

impl Default for FakeObservations {
    fn default() -> Self {
        Self {
            exchange_calls: Cell::new(0),
            write_calls: Cell::new(0),
            bytes_written: Cell::new(0),
            owner_was_dropped_at_return: Cell::new(false),
            returned_response_address: Cell::new(None),
            log_saw_correlation_prefix: Cell::new(false),
        }
    }
}

struct FakeExchange {
    delivery_state: RequestDeliveryState,
    response: Option<Vec<u8>>,
    failed: bool,
}

struct FakeLocal {
    script: Script,
    observations: Rc<FakeObservations>,
}

impl FakeLocal {
    fn write_chunk(
        &self,
        request: &[u8],
        offset: &mut usize,
        requested: usize,
        delivery_state: &mut RequestDeliveryState,
    ) -> usize {
        let end = offset.saturating_add(requested).min(request.len());
        let written_chunk = &request[*offset..end];
        *offset = end;
        self.observations
            .write_calls
            .set(self.observations.write_calls.get() + 1);
        self.observations.bytes_written.set(*offset);
        if !written_chunk.is_empty() {
            *delivery_state = delivery_state.write_started();
        }
        written_chunk.len()
    }

    fn exchange(
        &mut self,
        request: &[u8],
        request_observation: &CleanupObservation,
    ) -> FakeExchange {
        self.observations
            .exchange_calls
            .set(self.observations.exchange_calls.get() + 1);
        assert!(!request.is_empty(), "the fake receives the encoded request");

        let mut delivery_state = RequestDeliveryState::not_sent();
        let mut offset = 0;
        let (failed, response) = match self.script {
            Script::SuccessAfterPartialWrites => {
                for requested in [2, request.len().saturating_sub(2)] {
                    self.write_chunk(request, &mut offset, requested, &mut delivery_state);
                }
                delivery_state = delivery_state.response_bytes_received(1);
                let mut response = b"Pending:".to_vec();
                response.extend_from_slice(CORRELATION_VALUE);
                self.observations
                    .returned_response_address
                    .set(Some(response.as_ptr() as usize));
                (false, Some(response))
            }
            Script::Failure(FailurePoint::BeforeWrite) => (true, None),
            Script::Failure(FailurePoint::AfterPartialWrite) => {
                self.write_chunk(request, &mut offset, 2, &mut delivery_state);
                (true, None)
            }
            Script::Failure(FailurePoint::AfterResponseStarted) => {
                self.write_chunk(request, &mut offset, request.len(), &mut delivery_state);
                delivery_state = delivery_state.response_bytes_received(1);
                (true, None)
            }
        };
        self.observations
            .owner_was_dropped_at_return
            .set(request_observation.dropped.get());
        FakeExchange {
            delivery_state,
            response,
            failed,
        }
    }
}

struct Client {
    fake: FakeLocal,
}

fn client_factory(script: Script) -> Client {
    Client {
        fake: FakeLocal {
            script,
            observations: Rc::new(FakeObservations::default()),
        },
    }
}

struct PendingResult {
    response: ResponseOwner,
    correlation_range: Range<usize>,
    delivery_state: RequestDeliveryState,
}

impl PendingResult {
    fn asynchronous_correlation_value(&self) -> &[u8] {
        let end = self.correlation_range.end.saturating_sub(1);
        &self.response.bytes[self.correlation_range.start..end]
    }

    fn delivery_state(&self) -> RequestDeliveryState {
        self.delivery_state
    }
}

impl fmt::Debug for PendingResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PendingResult")
            .field(
                "asynchronous_correlation_value",
                &String::from_utf8_lossy(self.asynchronous_correlation_value()),
            )
            .finish()
    }
}

impl fmt::Display for PendingResult {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Pending result with correlation {}",
            String::from_utf8_lossy(self.asynchronous_correlation_value())
        )
    }
}

struct CandidateSourceError {
    diagnostic: &'static str,
}

impl fmt::Debug for CandidateSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("CandidateSourceError")
            .field(&self.diagnostic)
            .finish()
    }
}

impl fmt::Display for CandidateSourceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.diagnostic)
    }
}

impl Error for CandidateSourceError {}

struct CandidateError {
    delivery_state: RequestDeliveryState,
    source: CandidateSourceError,
}

impl CandidateError {
    fn delivery_state(&self) -> RequestDeliveryState {
        self.delivery_state
    }
}

impl fmt::Debug for CandidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CandidateError")
            .field("delivery_state", &self.delivery_state)
            .field("diagnostic", &self.source.diagnostic)
            .finish()
    }
}

impl fmt::Display for CandidateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "exchange failed: {}", self.source.diagnostic)
    }
}

impl Error for CandidateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.source)
    }
}

impl Client {
    /// Test-only, deliberately incorrect execute candidate. Green must retain
    /// the single request owner through exchange, propagate the fake's delivery
    /// evidence, preserve the complete borrowed correlation range, redact it,
    /// and zeroize the response allocation on drop.
    fn execute(
        &mut self,
        request_owner: RequestOwner,
        request_observation: CleanupObservation,
        response_observation: CleanupObservation,
    ) -> Result<PendingResult, CandidateError> {
        // Deliberate Red defect: this non-secret fixture is copied and its owner
        // is dropped before exchange. Green must pass the owner-backed slice.
        let request_copy = request_owner.encoded.clone();
        drop(request_owner);
        let exchange = self.fake.exchange(&request_copy, &request_observation);

        if exchange.failed {
            let reported_state = match exchange.delivery_state {
                RequestDeliveryState::NotSent | RequestDeliveryState::PossiblySent => {
                    RequestDeliveryState::NotSent
                }
                RequestDeliveryState::ResponseStarted => RequestDeliveryState::PossiblySent,
                _ => RequestDeliveryState::NotSent,
            };
            return Err(CandidateError {
                // Deliberate Red defect: ignore the evidence from the fake.
                delivery_state: reported_state,
                source: CandidateSourceError {
                    diagnostic: "KMIP_ASYNC_CORRELATION_SENTINEL_73",
                },
            });
        }

        let response = exchange.response.unwrap_or_default();
        let correlation_start = b"Pending:".len();
        let correlation_range = correlation_start..response.len();
        let correlation = &response[correlation_range.clone()];
        let mut captured_log = CapturedLog {
            saw_correlation_prefix: &self.fake.observations.log_saw_correlation_prefix,
        };
        let _ = write!(
            captured_log,
            "received Pending correlation {}",
            String::from_utf8_lossy(correlation)
        );

        Ok(PendingResult {
            response: ResponseOwner::new(response, response_observation),
            correlation_range,
            // Deliberate Red defect: response bytes arrived, but the candidate
            // reports only that request transmission may have started.
            delivery_state: match exchange.delivery_state {
                RequestDeliveryState::ResponseStarted => RequestDeliveryState::PossiblySent,
                RequestDeliveryState::NotSent | RequestDeliveryState::PossiblySent => {
                    RequestDeliveryState::PossiblySent
                }
                _ => RequestDeliveryState::PossiblySent,
            },
        })
    }
}

fn observations(client: &Client) -> &FakeObservations {
    &client.fake.observations
}

fn request_owner(observation: CleanupObservation) -> RequestOwner {
    RequestOwner::discover_versions(observation)
}

#[test]
fn success_after_partial_writes_reports_response_started_and_one_exchange() {
    let mut client = client_factory(Script::SuccessAfterPartialWrites);
    let result = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            CleanupObservation::default(),
        )
        .expect("the scripted transport succeeds");

    assert_eq!(observations(&client).write_calls.get(), 2);
    assert_eq!(
        observations(&client).bytes_written.get(),
        NON_SECRET_DISCOVER_VERSIONS_REQUEST.len()
    );
    assert_eq!(observations(&client).exchange_calls.get(), 1);
    assert_eq!(
        result.delivery_state(),
        RequestDeliveryState::ResponseStarted
    );
}

#[test]
fn failure_before_write_reports_not_sent_and_does_not_retry() {
    let mut client = client_factory(Script::Failure(FailurePoint::BeforeWrite));
    let error = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            CleanupObservation::default(),
        )
        .expect_err("the fake fails before writing");

    assert_eq!(error.delivery_state(), RequestDeliveryState::NotSent);
    assert_eq!(observations(&client).exchange_calls.get(), 1);
    assert_eq!(observations(&client).write_calls.get(), 0);
}

#[test]
fn failure_after_partial_write_reports_possibly_sent_and_does_not_retry() {
    let mut client = client_factory(Script::Failure(FailurePoint::AfterPartialWrite));
    let error = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            CleanupObservation::default(),
        )
        .expect_err("the fake fails after a partial write");

    assert_eq!(error.delivery_state(), RequestDeliveryState::PossiblySent);
    assert_eq!(observations(&client).exchange_calls.get(), 1);
    assert_eq!(observations(&client).write_calls.get(), 1);
    assert!(observations(&client).bytes_written.get() > 0);
    assert!(observations(&client).bytes_written.get() < NON_SECRET_DISCOVER_VERSIONS_REQUEST.len());
}

#[test]
fn failure_after_response_started_preserves_delivery_state_and_redacts_error_chain() {
    let mut client = client_factory(Script::Failure(FailurePoint::AfterResponseStarted));
    let error = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            CleanupObservation::default(),
        )
        .expect_err("the fake fails after response reception starts");
    let forbidden_prefix = CORRELATION_PREFIX;
    let mut source = error.source();
    let mut source_chain_redacts = true;
    while let Some(error_source) = source {
        source_chain_redacts &= !format!("{error_source:?}").contains(forbidden_prefix)
            && !error_source.to_string().contains(forbidden_prefix);
        source = error_source.source();
    }

    let delivery_state_preserved = error.delivery_state() == RequestDeliveryState::ResponseStarted;
    let debug_redacts = !format!("{error:?}").contains(forbidden_prefix);
    let display_redacts = !error.to_string().contains(forbidden_prefix);
    assert!(
        delivery_state_preserved && debug_redacts && display_redacts && source_chain_redacts,
        "error checks: state={delivery_state_preserved}, debug_redacted={debug_redacts}, display_redacted={display_redacts}, sources_redacted={source_chain_redacts}"
    );
    assert_eq!(observations(&client).exchange_calls.get(), 1);
}

#[test]
fn pending_correlation_is_byte_exact_borrowed_and_absent_from_text_and_logs() {
    let mut client = client_factory(Script::SuccessAfterPartialWrites);
    let result = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            CleanupObservation::default(),
        )
        .expect("the scripted transport succeeds");
    let observed = result.asynchronous_correlation_value();
    let debug = format!("{result:?}");
    let display = result.to_string();
    let exact_value_preserved = observed == CORRELATION_VALUE;
    let debug_redacts = !debug.contains(CORRELATION_PREFIX);
    let display_redacts = !display.contains(CORRELATION_PREFIX);
    let captured_log_redacts = !observations(&client).log_saw_correlation_prefix.get();

    assert!(
        exact_value_preserved && debug_redacts && display_redacts && captured_log_redacts,
        "correlation checks: exact={exact_value_preserved}, debug_redacted={debug_redacts}, display_redacted={display_redacts}, log_redacted={captured_log_redacts}"
    );
}

#[test]
fn response_owner_moves_the_fake_allocation_and_zeroizes_before_drop() {
    let mut client = client_factory(Script::SuccessAfterPartialWrites);
    let response_observation = CleanupObservation::default();
    let result = client
        .execute(
            request_owner(CleanupObservation::default()),
            CleanupObservation::default(),
            response_observation.clone(),
        )
        .expect("the scripted transport succeeds");
    let returned_address = observations(&client)
        .returned_response_address
        .get()
        .expect("fake recorded its returned allocation address");

    assert_eq!(result.response.bytes.as_ptr() as usize, returned_address);
    drop(result);

    assert!(response_observation.dropped.get());
    assert!(response_observation.initialized_range_was_zero.get());
}

#[test]
fn request_owner_remains_live_until_exchange_returns_then_zeroizes_before_free() {
    let mut client = client_factory(Script::SuccessAfterPartialWrites);
    let request_observation = CleanupObservation::default();
    let result = client.execute(
        request_owner(request_observation.clone()),
        request_observation.clone(),
        CleanupObservation::default(),
    );

    let execute_succeeded = result.is_ok();
    let owner_remained_live = !observations(&client).owner_was_dropped_at_return.get();
    let cleanup_was_observed = request_observation.dropped.get();
    let initialized_request_bytes_were_zero = request_observation.initialized_range_was_zero.get();
    assert!(
        execute_succeeded
            && owner_remained_live
            && cleanup_was_observed
            && initialized_request_bytes_were_zero,
        "request-owner checks: execute={execute_succeeded}, live_through_exchange={owner_remained_live}, cleanup_observed={cleanup_was_observed}, initialized_range_zero={initialized_request_bytes_were_zero}"
    );
}
