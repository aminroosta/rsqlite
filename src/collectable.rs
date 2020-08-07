use super::Statement;

use libc::{c_double, c_int};
use sqlite3_sys as ffi;

/// Collectable types can be parsed from the columns of the sqlite result row
pub trait Collectable
where
    Self: Sized,
{
    /// collects itself and increments to next column
    fn collect(statement: &Statement, column: &mut c_int) -> Self;
}

impl Collectable for () {
    fn collect(_statement: &Statement, _column: &mut c_int) -> Self {}
}

impl Collectable for c_int {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let result = unsafe { ffi::sqlite3_column_int(statement.stmt, *column) };
        *column += 1;
        result
    }
}

impl Collectable for c_double {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let result = unsafe { ffi::sqlite3_column_double(statement.stmt, *column) };
        *column += 1;
        result
    }
}
impl Collectable for String {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let ptr = unsafe { ffi::sqlite3_column_text(statement.stmt, *column) };
        let bytes = unsafe { ffi::sqlite3_column_bytes(statement.stmt, *column) };

        *column += 1;

        match bytes == 0 {
            true => String::new(),
            false => unsafe {
                let slice = std::slice::from_raw_parts(ptr as *const u8, bytes as usize);
                String::from_utf8_unchecked(slice.to_owned())
            },
        }
    }
}
impl Collectable for Box<[u8]> {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let ptr = unsafe { ffi::sqlite3_column_blob(statement.stmt, *column) };
        let bytes = unsafe { ffi::sqlite3_column_bytes(statement.stmt, *column) };

        *column += 1;

        match bytes == 0 {
            true => (vec![]).into_boxed_slice(),
            false => unsafe {
                let slice = std::slice::from_raw_parts(ptr as *const u8, bytes as usize);
                slice.to_owned().into_boxed_slice()
            },
        }
    }
}

impl<T0, T1, T2> Collectable for (T0, T1, T2)
    where T0 : Collectable,
          T1: Collectable,
          T2: Collectable,
{
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        (
            T0::collect(statement, column),
            T1::collect(statement, column),
            T2::collect(statement, column),
        )
    }
}

