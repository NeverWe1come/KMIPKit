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
    let result = SecretBuffer::read_from_with_observer_for_test(&mut reader, observer.clone(), 0);

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

    let Ok(owner) = result else {
        panic!("the split key reader must complete");
    };
    assert!(
        owner.as_slice() == KEY_SENTINEL.as_slice(),
        "the read owner must preserve all input bytes"
    );
    drop(owner);

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
