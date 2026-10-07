use std::cell::Cell;

thread_local! {
    static INDEX_COMPILATION_ATTEMPTS: Cell<usize> = const { Cell::new(0) };
}

pub(crate) fn record_index_compilation_attempt() {
    INDEX_COMPILATION_ATTEMPTS.with(|attempts| attempts.set(attempts.get().saturating_add(1)));
}

pub(crate) fn index_compilation_attempts() -> usize {
    INDEX_COMPILATION_ATTEMPTS.with(Cell::get)
}
