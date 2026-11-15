//! Unit tests for KMIPKit-owned credential secret storage.

use std::{cell::Cell, rc::Rc};

use zeroize::Zeroize;

use super::Secret;

struct DropProbe(Rc<Cell<usize>>);

impl Zeroize for DropProbe {
    fn zeroize(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn owned_secret_storage_calls_zeroize_before_drop() {
    let call_count = Rc::new(Cell::new(0));

    drop(Secret(DropProbe(Rc::clone(&call_count))));

    assert_eq!(call_count.get(), 1);
}
