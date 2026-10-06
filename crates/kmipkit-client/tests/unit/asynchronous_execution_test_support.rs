//! Deterministic one-exchange fixtures for asynchronous client execution.

use std::cell::RefCell;
use std::rc::Rc;

use kmipkit_test_support::{ExchangeScript, ScriptedTransport};
use kmipkit_transport::{Transport, TransportError, TransportResponse};
use zeroize::Zeroizing;

use crate::execute::Client;

pub(crate) struct CapturingTransport {
    fake: Rc<RefCell<ScriptedTransport>>,
    request: Rc<RefCell<Option<Zeroizing<Vec<u8>>>>>,
}

impl Transport for CapturingTransport {
    fn exchange(
        &mut self,
        request: &[u8],
        max_response_bytes: usize,
    ) -> Result<TransportResponse, TransportError> {
        *self.request.borrow_mut() = Some(Zeroizing::new(request.to_vec()));
        self.fake.borrow_mut().exchange(request, max_response_bytes)
    }
}

pub(crate) fn client_for(
    script: ExchangeScript,
) -> (
    Client,
    Rc<RefCell<ScriptedTransport>>,
    Rc<RefCell<Option<Zeroizing<Vec<u8>>>>>,
) {
    let fake = Rc::new(RefCell::new(ScriptedTransport::new(script)));
    let request = Rc::new(RefCell::new(None));
    let client = Client::for_test(CapturingTransport {
        fake: Rc::clone(&fake),
        request: Rc::clone(&request),
    });
    (client, fake, request)
}

pub(crate) fn client_for_with_response_observer(
    script: ExchangeScript,
    observer: kmipkit_test_support::ResponseDropObserver,
) -> (
    Client,
    Rc<RefCell<ScriptedTransport>>,
    Rc<RefCell<Option<Zeroizing<Vec<u8>>>>>,
) {
    let fake = Rc::new(RefCell::new(
        ScriptedTransport::new(script).with_response_drop_observer(observer),
    ));
    let request = Rc::new(RefCell::new(None));
    let client = Client::for_test(CapturingTransport {
        fake: Rc::clone(&fake),
        request: Rc::clone(&request),
    });
    (client, fake, request)
}

pub(crate) fn request_contains(
    request: &Rc<RefCell<Option<Zeroizing<Vec<u8>>>>>,
    value: &[u8],
) -> bool {
    request
        .borrow()
        .as_ref()
        .is_some_and(|bytes| bytes.windows(value.len()).any(|window| window == value))
}
