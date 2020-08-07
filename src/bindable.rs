
use super::{Result, Statement};
use libc::{c_char, c_double, c_int, c_void};
use sqlite3_sys as ffi;

/// Bindable types can bind themselves to a sqlite statement
pub trait Bindable {
    /// given an index, binds itself and increments the index
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()>;
    // fn bind(param: impl AsRef<Self>, statement: &Statement, index: &mut c_int) -> Result<()>;
}

/// allow binding with `&T` if `T` is Bindable
impl<T> Bindable for &T
where
    T: Bindable,
{
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        (*self).bind(statement, index)
    }
}

impl Bindable for () {
    fn bind(&self, _statement: &Statement, _index: &mut c_int) -> Result<()> {
        Ok(())
    }
}
impl Bindable for i32 {
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        let ecode = unsafe { ffi::sqlite3_bind_int(statement.stmt, *index, *self) };
        *index += 1;
        match ecode {
            ffi::SQLITE_OK => Ok(()),
            other => Err(other.into()),
        }
    }
}
impl Bindable for c_double {
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        let ecode = unsafe { ffi::sqlite3_bind_double(statement.stmt, *index, *self) };
        *index += 1;
        match ecode {
            ffi::SQLITE_OK => Ok(()),
            other => Err(other.into()),
        }
    }
}
impl Bindable for ffi::sqlite3_int64 {
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        let ecode = unsafe { ffi::sqlite3_bind_int64(statement.stmt, *index, *self) };
        *index += 1;
        match ecode {
            ffi::SQLITE_OK => Ok(()),
            other => Err(other.into()),
        }
    }
}
/// sqlite3_bind_text() expects a pointer to well-formed UTF8 text (i.e `&str`)
impl<'a> Bindable for &'a str {
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        let len = self.as_bytes().len() as c_int;
        let ecode = unsafe {
            ffi::sqlite3_bind_text(
                statement.stmt,
                *index,
                self.as_ptr() as *const c_char,
                len,
                Some(std::mem::transmute(-1isize)), // ffi::SQLITE_TRANSIENT
            )
        };
        *index += 1;
        match ecode {
            ffi::SQLITE_OK => Ok(()),
            other => Err(other.into()),
        }
    }
}
/// `&[u8]` is treated as sqlite `blob` data type
impl<'a> Bindable for &'a [u8] {
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        let ecode = unsafe {
            ffi::sqlite3_bind_blob(
                statement.stmt,
                *index,
                self.as_ptr() as *const c_void,
                self.len() as c_int,
                Some(std::mem::transmute(-1isize)), // ffi::SQLITE_TRANSIENT
            )
        };
        *index += 1;
        match ecode {
            ffi::SQLITE_OK => Ok(()),
            other => Err(other.into()),
        }
    }
}

/// binds a tuple of `(T0, T1 ...)` where each type `T?` is itself `Bindable`
macro_rules! bindable_tuple {
    ($($name:ident as $idx:tt),+) => (
        impl<$($name),+> Bindable for ($($name),+,) where
            $($name: Bindable),+
        {
            fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
                $(self.$idx.bind(statement, index)?;)+
                Ok(())
            }
        }
    );
}

bindable_tuple!(T0 as 0);
bindable_tuple!(T0 as 0, T1 as 1);
bindable_tuple!(T0 as 0, T1 as 1, T2 as 2);
bindable_tuple!(T0 as 0, T1 as 1, T2 as 2, T3 as 3);
bindable_tuple!(T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4);
bindable_tuple!(T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5);
bindable_tuple!(T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7
);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7, T8 as 8
);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7, T8 as 8, T9 as 9
);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7, T8 as 8, T9 as 9, T10 as 10
);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7, T8 as 8, T9 as 9, T10 as 10, T11 as 11
);
bindable_tuple!(
    T0 as 0, T1 as 1, T2 as 2, T3 as 3, T4 as 4, T5 as 5, T6 as 6,
    T7 as 7, T8 as 8, T9 as 9, T10 as 10, T11 as 11, T12 as 12
);
