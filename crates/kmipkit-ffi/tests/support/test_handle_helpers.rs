use super::{try_make_handle, HandleValue, Kind};

pub(super) fn make_handle<T>(kind: Kind, value: HandleValue) -> *mut T {
    match try_make_handle(kind, value) {
        Ok(handle) => handle,
        Err(status) => panic!("test handle allocation failed with status {status}"),
    }
}
