use super::{HandleValue, Kind, try_make_handle};

pub(super) fn make_handle<T>(kind: Kind, value: HandleValue) -> *mut T {
    match try_make_handle(kind, value) {
        Ok(handle) => handle,
        Err(status) => panic!("test handle allocation failed with status {status}"),
    }
}
