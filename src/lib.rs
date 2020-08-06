//! # Ergonamic sqlite library with zero overhead.
//!
//! ```
//! use rsqlite::{Database, Result};
//!
//! fn foo() -> Result<()> {
//!     let database = Database::open(":memory:")?;
//!
//!     // execute a query with no arguments
//!     database.execute(r#"
//!        create table if not exists user (
//!           id integer primary key autoincrement not null,
//!           age int,
//!           name text,
//!           weight real
//!        );"#, ())?;
//!
//!     // execute with parameters
//!     database.execute(
//!        "insert into user(age, name, weight) values(?, ?, ?)",
//!        (29, "amin", 69.5)
//!     )?;
//!
//!     Ok(())
//! }
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

pub trait Bindable {
    /// binds &self to the sqlite statement and returns the next `index`
    fn bind(&self, statement: &Statement, index: c_int) -> Result<c_int>;
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
        arg_or_args.bind(&statement, 0)?;

        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        match retcode {
            ffi::SQLITE_DONE => Ok(()),
            other => Err(other.into()),
        }
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
