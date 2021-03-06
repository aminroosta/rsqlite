use super::{Result, Statement};

use libc::{c_double, c_int};
use sqlite3_sys as ffi;

/// Collectable types can be parsed from the columns of the sqlite result row
pub trait Collectable
where
    Self: Sized,
{
    /// collects itself and increments to next column
    fn collect(statement: &Statement, column: &mut c_int) -> Self;

    fn step_and_collect(statement: &mut Statement) -> Result<Self> {
        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        match retcode {
            ffi::SQLITE_ROW => Ok(Self::collect(statement, &mut 0)),
            other => Err(other.into()),
        }
    }

    /// number of columns needed by this type
    fn columns_needed() -> c_int;
}

impl Collectable for () {
    fn collect(_statement: &Statement, _column: &mut c_int) -> Self {}
    fn columns_needed() -> c_int {
        0
    }
}
impl<T> Collectable for Option<T>
where
    T: Collectable,
{
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let sqlite_type = unsafe { ffi::sqlite3_column_type(statement.stmt, *column) };
        match sqlite_type {
            ffi::SQLITE_NULL => {
                *column += 1;
                None
            }
            _ => Some(T::collect(statement, column)),
        }
    }
    fn step_and_collect(statement: &mut Statement) -> Result<Self> {
        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        match retcode {
            ffi::SQLITE_ROW => Ok(Self::collect(statement, &mut 0)),
            ffi::SQLITE_DONE => Ok(None),
            other => Err(other.into()),
        }
    }
    fn columns_needed() -> c_int {
        T::columns_needed()
    }
}
impl Collectable for c_int {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let result = unsafe { ffi::sqlite3_column_int(statement.stmt, *column) };
        *column += 1;
        result
    }
    fn columns_needed() -> c_int {
        1
    }
}
impl Collectable for c_double {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let result = unsafe { ffi::sqlite3_column_double(statement.stmt, *column) };
        *column += 1;
        result
    }
    fn columns_needed() -> c_int {
        1
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
    fn columns_needed() -> c_int {
        1
    }
}
impl Collectable for Box<[u8]> {
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let ptr = unsafe { ffi::sqlite3_column_blob(statement.stmt, *column) };
        let bytes = unsafe { ffi::sqlite3_column_bytes(statement.stmt, *column) };

        *column += 1;

        match bytes == 0 {
            true => Box::new([]),
            false => unsafe {
                let slice = std::slice::from_raw_parts(ptr as *const u8, bytes as usize);
                slice.to_owned().into_boxed_slice()
            },
        }
    }
    fn columns_needed() -> c_int {
        1
    }
}

macro_rules! collectable_tuple {
    ($columns_needed:expr, $($name: ident),+) => (
        impl<$($name),+> Collectable for ($($name,)+)
        where
            $($name: Collectable,)+
        {
            fn collect(statement: &Statement, column: &mut c_int) -> Self {
                (
                    $($name::collect(statement, column),)+
                )
            }
            fn columns_needed() -> c_int { $columns_needed }
        }
    );
}

collectable_tuple!(1, T0);
collectable_tuple!(2, T0, T1);
collectable_tuple!(3, T0, T1, T2);
collectable_tuple!(4, T0, T1, T2, T3);
collectable_tuple!(5, T0, T1, T2, T3, T4);
collectable_tuple!(6, T0, T1, T2, T3, T4, T5);
collectable_tuple!(7, T0, T1, T2, T3, T4, T5, T6);
collectable_tuple!(8, T0, T1, T2, T3, T4, T5, T6, T7);
collectable_tuple!(9, T0, T1, T2, T3, T4, T5, T6, T7, T8);
collectable_tuple!(10, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9);
collectable_tuple!(11, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10);
collectable_tuple!(12, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11);
collectable_tuple!(13, T0, T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12);
