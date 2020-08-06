
use super::{Result, Statement};
use libc::{c_char, c_double, c_int};
use sqlite3_sys as ffi;

/// Bindable types can bind themselves to a sqlite statement
pub trait Bindable {
    /// given an index, binds itself and increments the next
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()>;
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

impl<T0, T1, T2> Bindable for (T0, T1, T2)
where
    T0: Bindable,
    T1: Bindable,
    T2: Bindable,
{
    fn bind(&self, statement: &Statement, index: &mut c_int) -> Result<()> {
        self.0.bind(statement, index)?;
        self.1.bind(statement, index)?;
        self.2.bind(statement, index)?;
        Ok(())
    }
}
