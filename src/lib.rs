//! # Ergonamic sqlite library with zero overhead.
//!
//! ```
//! # use rsqlite::*;
//! 
//! let database = Database::open(":memory:")?;
//!
//! // execute a query with no arguments
//! database.execute(r#"
//!    create table if not exists user (
//!       id integer primary key autoincrement not null,
//!       age int,
//!       name text,
//!       weight real
//!    );"#, ())?;
//!
//! // execute with parameters
//! database.execute(
//!    "insert into user(age, name, weight) values(?, ?, ?)",
//!    (29, "amin", 69.5)
//! )?;
//!
//! // collect example
//! let age: i32 = database.collect("select age from user limit 1", ())?;
//! assert!(age == 29);
//!
//! let info: (i32, String, f64) = database.collect(
//!     "select age, name, weight from user limit 1", ()
//! )?;
//! assert!(info == (29, "amin".to_owned(), 69.5));
//!
//! # Ok::<(), RsqliteError>(())
//! ```
#![allow(incomplete_features)]
#![feature(specialization)]

pub mod error;
pub use error::RsqliteError;

use core::ptr;
use libc::{c_char, c_double, c_int};
use sqlite3_sys as ffi;
use std::ffi::CString;

pub type Result<T> = std::result::Result<T, RsqliteError>;

pub struct Database {
    pub db: *mut ffi::sqlite3,
}

pub struct Statement {
    pub stmt: *mut ffi::sqlite3_stmt,
}

/// Bindable types can bind themselves to a sqlite statement
pub trait Bindable {
    /// given an index, binds itself and returns the next index
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int>;
}
/// Collectable types can be parsed from the columns of the sqlite result row
pub trait Collectable
where
    Self: Sized,
{
    /// given an column, collects itself and returns the next column
    fn collect(statement: &Statement, column: &mut c_int) -> Self;
}

impl Database {
    /// open an existing sqlite3 database or create a new one.
    ///
    /// ```
    /// # use rsqlite::*;
    /// let database = Database::open(":memory:")?;
    /// # assert!(!database.db.is_null());
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn open(path: &str) -> Result<Database> {
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;
        Database::open_with_flags(path, flags)
    }
    /// open a sqlite3 database with explicit flags
    ///
    /// ```
    /// # use rsqlite::*;
    /// use sqlite3_sys as ffi;
    ///
    /// let database = Database::open_with_flags(
    ///     ":memory:",
    ///     ffi::SQLITE_OPEN_CREATE | ffi::SQLITE_OPEN_READWRITE
    /// )?;
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn open_with_flags(path: &str, flags: c_int) -> Result<Database> {
        let path = CString::new(path)?;
        let mut db = ptr::null_mut();
        let retcode = unsafe { ffi::sqlite3_open_v2(path.as_ptr(), &mut db, flags, ptr::null()) };

        // Drop will close this if it is open_v2 has failed
        let database = Database { db };

        match retcode {
            ffi::SQLITE_OK => Ok(database),
            other => Err(other.into()),
        }
    }

    /// prepare a query to be executed
    ///
    /// ```
    /// # use rsqlite::*;
    /// # let database = Database::open(":memory:")?;
    /// let statement = database.prepare("select 1+2;")?;
    /// # assert!(!statement.stmt.is_null());
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn prepare(&self, sql: &str) -> Result<Statement> {
        let sql = CString::new(sql)?;
        let mut stmt = ptr::null_mut();
        let len = sql.as_bytes_with_nul().len() as i32;
        let retcode = unsafe {
            ffi::sqlite3_prepare_v2(self.db, sql.as_ptr(), len, &mut stmt, ptr::null_mut())
        };

        let statement = Statement { stmt };
        match retcode {
            ffi::SQLITE_OK => Ok(statement),
            other => Err(other.into()),
        }
    }

    /// execute an sqlite query
    ///
    /// it is expected that the query does to return any data
    /// if you need to return data, you should use `.query()`
    pub fn execute(&self, sql: &str, arg_or_args: impl Bindable) -> Result<()> {
        let statement = self.prepare(sql)?;
        arg_or_args.bind(&statement, 1)?;

        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        match retcode {
            ffi::SQLITE_DONE => Ok(()),
            other => Err(other.into()),
        }
    }

    pub fn collect<R>(&self, sql: &str, arg_or_args: impl Bindable) -> Result<R>
    where
        R: Collectable,
    {
        let statement = self.prepare(sql)?;
        arg_or_args.bind(&statement, 1)?;

        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        let result = match retcode {
            ffi::SQLITE_ROW => Ok(R::collect(&statement, &mut 0)),
            other => Err(other.into()),
        };

        let _ = unsafe { ffi::sqlite3_reset(statement.stmt) };

        result
    }
}

impl Drop for Database {
    /// closes the `*mut sqlite3` handle on Drop
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close(self.db);
            self.db = ptr::null_mut();
        }
    }
}

impl Drop for Statement {
    /// closes `*mut sqlite3_statement` handle on Drop
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_finalize(self.stmt);
            self.stmt = ptr::null_mut();
        }
    }
}

/// allow binding with `&T` if `T` is Bindable
impl<T> Bindable for &T
where
    T: Bindable,
{
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int> {
        (*self).bind(statement, index)
    }
}

impl Bindable for () {
    fn bind(&self, _statement: &Statement, index: c_int) -> Result<c_int> {
        Ok(index)
    }
}
impl Bindable for i32 {
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int> {
        let ecode = unsafe { ffi::sqlite3_bind_int(statement.stmt, index, *self) };
        match ecode {
            ffi::SQLITE_OK => Ok(index + 1),
            other => Err(other.into()),
        }
    }
}
impl Bindable for c_double {
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int> {
        let ecode = unsafe { ffi::sqlite3_bind_double(statement.stmt, index, *self) };
        match ecode {
            ffi::SQLITE_OK => Ok(index + 1),
            other => Err(other.into()),
        }
    }
}
/// sqlite3_bind_text() expects a pointer to well-formed UTF8 text (i.e `&str`)
impl<'a> Bindable for &'a str {
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int> {
        let len = self.as_bytes().len() as c_int;
        let ecode = unsafe {
            ffi::sqlite3_bind_text(
                statement.stmt,
                index,
                self.as_ptr() as *const c_char,
                len,
                Some(std::mem::transmute(-1isize)), // ffi::SQLITE_TRANSIENT
            )
        };
        match ecode {
            ffi::SQLITE_OK => Ok(index + 1),
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
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int> {
        let index = self.0.bind(statement, index)?;
        let index = self.1.bind(statement, index)?;
        let index = self.2.bind(statement, index)?;
        Ok(index)
    }
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
impl<T0, T1, T2> Collectable for (T0, T1, T2)
    where T0 : Collectable,
          T1: Collectable,
          T2: Collectable,
{
    fn collect(statement: &Statement, column: &mut c_int) -> Self {
        let t0 = T0::collect(statement, column);
        let t1 = T1::collect(statement, column);
        let t2 = T2::collect(statement, column);

        (t0, t1, t2)
    }
}

