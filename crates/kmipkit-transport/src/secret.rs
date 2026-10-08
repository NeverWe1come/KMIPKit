//! Private owners for secret input and temporary key encodings.

use std::io::{self, Read};

use rustls::pki_types::PrivateKeyDer;
use zeroize::Zeroize;

const SECRET_READ_BUFFER_SIZE: usize = 4096;

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

    pub(crate) fn read_from<R: Read>(reader: R) -> io::Result<Self> {
        Self::read_from_owned(reader, Self::new(Vec::new()))
    }

    fn read_from_owned<R: Read>(mut reader: R, mut owner: Self) -> io::Result<Self> {
        let mut scratch = SecretReadScratch::new();
        loop {
            let bytes_read = reader
                .read(&mut scratch.0)
                .map_err(|_| credential_read_error())?;
            if bytes_read == 0 {
                break;
            }
            if bytes_read > scratch.0.len() {
                return Err(credential_read_error());
            }

            owner.append(&scratch.0[..bytes_read])?;
            scratch.0.zeroize();
        }
        Ok(owner)
    }

    fn append(&mut self, bytes: &[u8]) -> io::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }

        let new_len = self
            .bytes
            .len()
            .checked_add(bytes.len())
            .ok_or_else(credential_read_error)?;
        if new_len <= self.bytes.capacity() {
            self.bytes.extend_from_slice(bytes);
            return Ok(());
        }

        let new_capacity = self.bytes.capacity().saturating_mul(2).max(new_len);
        let mut replacement = Self::new(Vec::new());
        replacement
            .bytes
            .try_reserve_exact(new_capacity)
            .map_err(|_| credential_read_error())?;
        replacement.bytes.extend_from_slice(&self.bytes);
        replacement.bytes.extend_from_slice(bytes);

        self.bytes.as_mut_slice().zeroize();
        #[cfg(test)]
        if let Some(observer) = &self.observer {
            observer.record_replaced_allocation(self.bytes.as_slice());
        }
        self.bytes = std::mem::take(&mut replacement.bytes);
        Ok(())
    }

    #[cfg(test)]
    fn read_from_with_observer_for_test<R: Read>(
        reader: R,
        observer: SecretBufferObserver,
        initial_capacity: usize,
    ) -> io::Result<Self> {
        Self::read_from_owned(
            reader,
            Self::with_observer(Vec::with_capacity(initial_capacity), observer),
        )
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

fn credential_read_error() -> io::Error {
    io::Error::other("credential source could not be read")
}

struct SecretReadScratch([u8; SECRET_READ_BUFFER_SIZE]);

impl SecretReadScratch {
    fn new() -> Self {
        Self([0; SECRET_READ_BUFFER_SIZE])
    }
}

impl Drop for SecretReadScratch {
    fn drop(&mut self) {
        self.0.zeroize();
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
    replaced_allocation_count: std::sync::atomic::AtomicUsize,
    all_replaced_allocations_were_zero: std::sync::atomic::AtomicBool,
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
                replaced_allocation_count: std::sync::atomic::AtomicUsize::new(0),
                all_replaced_allocations_were_zero: std::sync::atomic::AtomicBool::new(false),
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

    pub(crate) fn replaced_allocation_count(&self) -> usize {
        self.state
            .replaced_allocation_count
            .load(std::sync::atomic::Ordering::Acquire)
    }

    pub(crate) fn all_replaced_allocations_were_zero(&self) -> bool {
        self.state
            .all_replaced_allocations_were_zero
            .load(std::sync::atomic::Ordering::Acquire)
    }

    /// Checks the old initialized range before its allocation is released.
    fn record_replaced_allocation(&self, old_initialized_bytes: &[u8]) {
        let initialized_range_was_zero = old_initialized_bytes.iter().all(|byte| *byte == 0);
        let previous_count = self
            .state
            .replaced_allocation_count
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel);
        if previous_count == 0 {
            self.state.all_replaced_allocations_were_zero.store(
                initialized_range_was_zero,
                std::sync::atomic::Ordering::Release,
            );
        } else {
            self.state.all_replaced_allocations_were_zero.fetch_and(
                initialized_range_was_zero,
                std::sync::atomic::Ordering::AcqRel,
            );
        }
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

    static KEY_SENTINEL: [u8; 65_536] = [0xa7; 65_536];

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

    struct SplitKeyReader {
        offset: usize,
        first_chunk_len: usize,
    }

    impl Read for SplitKeyReader {
        fn read(&mut self, destination: &mut [u8]) -> io::Result<usize> {
            if destination.is_empty() || self.offset == KEY_SENTINEL.len() {
                return Ok(0);
            }

            let remaining = &KEY_SENTINEL[self.offset..];
            let count = if self.offset == 0 {
                self.first_chunk_len.min(destination.len())
            } else {
                remaining.len().min(destination.len())
            };
            destination[..count].copy_from_slice(&remaining[..count]);
            self.offset += count;
            Ok(count)
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
        let result =
            SecretBuffer::read_from_with_observer_for_test(&mut reader, observer.clone(), 0);

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

    #[test]
    fn key_buffer_growth_zeroizes_each_replaced_allocation_before_release() {
        const INITIAL_CAPACITY: usize = 1;

        let observer = SecretBufferObserver::new(KEY_SENTINEL.len());
        let mut reader = SplitKeyReader {
            offset: 0,
            first_chunk_len: INITIAL_CAPACITY,
        };
        let result = SecretBuffer::read_from_with_observer_for_test(
            &mut reader,
            observer.clone(),
            INITIAL_CAPACITY,
        );

        assert!(result.is_ok(), "the split key reader must complete");
        drop(result);

        assert_eq!(
            observer.initialized_len(),
            KEY_SENTINEL.len(),
            "the final owner must observe the complete initialized range"
        );
        assert!(
            observer.initialized_range_was_zero(),
            "the final allocation must be zeroized before release"
        );
        assert!(
            observer.replaced_allocation_count() > 0,
            "the read owner must report at least one replaced allocation"
        );
        assert!(
            observer.all_replaced_allocations_were_zero(),
            "every replaced allocation must be zeroized before release"
        );
    }
}
