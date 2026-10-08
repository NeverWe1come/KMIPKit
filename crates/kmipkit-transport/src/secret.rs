//! Private owners for secret input and temporary key encodings.

use rustls::pki_types::PrivateKeyDer;
use zeroize::Zeroize;

/// Owns key input bytes and clears the initialized range before its allocation is released.
pub(crate) struct SecretBuffer {
    bytes: Vec<u8>,
    #[cfg(test)]
    observer: Option<SecretBufferObserver>,
}

impl SecretBuffer {
    pub(crate) fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            #[cfg(test)]
            observer: None,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)] // The source-including contract target supplies this observer.
    pub(crate) fn with_observer(bytes: Vec<u8>, observer: SecretBufferObserver) -> Self {
        Self {
            bytes,
            observer: Some(observer),
        }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for SecretBuffer {
    fn drop(&mut self) {
        #[cfg(test)]
        let initialized_len = self.bytes.len();
        self.bytes.as_mut_slice().zeroize();

        #[cfg(test)]
        if let Some(observer) = &self.observer {
            let initialized_range_was_zero = self.bytes.iter().all(|byte| *byte == 0);
            observer.record(initialized_len, initialized_range_was_zero);
        }
    }
}

/// Keeps parser-produced key bytes under explicit zeroizing ownership until transfer.
pub(crate) struct SecretPrivateKeyDer(Option<PrivateKeyDer<'static>>);

impl SecretPrivateKeyDer {
    pub(crate) fn new(key: PrivateKeyDer<'static>) -> Self {
        Self(Some(key))
    }

    pub(crate) fn into_inner(mut self) -> Option<PrivateKeyDer<'static>> {
        self.0.take()
    }
}

impl Drop for SecretPrivateKeyDer {
    fn drop(&mut self) {
        if let Some(key) = &mut self.0 {
            key.zeroize();
        }
    }
}

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct SecretBufferObserver {
    state: std::sync::Arc<ObserverState>,
}

#[cfg(test)]
struct ObserverState {
    expected_initialized_len: usize,
    initialized_len: std::sync::atomic::AtomicUsize,
    initialized_range_was_zero: std::sync::atomic::AtomicBool,
}

#[cfg(test)]
#[allow(dead_code)] // The integration contract compiles this module separately from the library.
impl SecretBufferObserver {
    pub(crate) fn new(expected_initialized_len: usize) -> Self {
        Self {
            state: std::sync::Arc::new(ObserverState {
                expected_initialized_len,
                initialized_len: std::sync::atomic::AtomicUsize::new(0),
                initialized_range_was_zero: std::sync::atomic::AtomicBool::new(false),
            }),
        }
    }

    pub(crate) fn initialized_len(&self) -> usize {
        self.state
            .initialized_len
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn initialized_range_was_zero(&self) -> bool {
        self.state
            .initialized_range_was_zero
            .load(std::sync::atomic::Ordering::Acquire)
    }

    fn record(&self, initialized_len: usize, initialized_range_was_zero: bool) {
        self.state
            .initialized_len
            .store(initialized_len, std::sync::atomic::Ordering::Release);
        self.state.initialized_range_was_zero.store(
            initialized_len == self.state.expected_initialized_len && initialized_range_was_zero,
            std::sync::atomic::Ordering::Release,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::io::{self, Read};

    use super::{SecretBuffer, SecretBufferObserver};

    struct FailsAfterBytes {
        bytes: &'static [u8],
        offset: usize,
    }

    impl Read for FailsAfterBytes {
        fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
            if destination.is_empty() {
                return Ok(0);
            }

            if self.offset < self.bytes.len() {
                let available = &self.bytes[self.offset..];
                let count = available.len().min(destination.len());
                destination[..count].copy_from_slice(&available[..count]);
                self.offset += count;
                return Ok(count);
            }

            Err(io::Error::other("injected read failure"))
        }
    }

    #[test]
    fn partially_read_private_key_is_zeroized_when_reader_fails() {
        const KEY_SENTINEL: &[u8] = b"private-key-read-error-sentinel";

        let observer = SecretBufferObserver::new(KEY_SENTINEL.len());
        let mut reader = FailsAfterBytes {
            bytes: KEY_SENTINEL,
            offset: 0,
        };
        let result = SecretBuffer::read_from_with_observer_for_test(&mut reader, observer.clone());

        assert!(
            result.is_err(),
            "the injected reader must fail after yielding bytes"
        );
        assert_eq!(
            observer.initialized_len(),
            KEY_SENTINEL.len(),
            "the read owner must observe every initialized byte before release"
        );
        assert!(
            observer.initialized_range_was_zero(),
            "the read owner must zero initialized bytes before release"
        );
    }
}
