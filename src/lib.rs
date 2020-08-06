//! # Ergonamic sqlite library with zero overhead.
//!
//! ```
//! # use rsqlite::*;
//! // open existing database or create a new one
//! let database = Database::open(":memory:")?;
//!
//! // execute a query
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
//! database.execute(
//!    "insert into user(age, name, weight) values(?, ?, ?)",
//!    (26, "negar", 61.0)
//! )?;
//!
//! // collect a single row
//! let info: (i32, String, f64) = database.collect(
//!     "select age, name, weight from user where name = ?", ("amin")
//! )?;
//! // collect a single row single column
//! let age: i32 = database.collect(
//!     "select age from user where name = ?", ("amin")
//! )?;
//! # assert!(age == 29);
//!
//! // given your own data structure:
//! #[derive(PartialEq, Debug)]
//! struct User { name: String, age: i32, weight: f64 };
//! let mut users = vec![];
//! // collect multiple rows using `.iterate()`
//! // returned columns should match the lambda arguments
//! database.iterate(
//!     "select name, age, weight from user", (),
//!     |name: String, age: i32, weight: f64| {
//!         users.push(User { name, age, weight });
//!     }
//! )?;
//!     
//! # assert!(users == vec![
//! #   User { name: "amin".to_owned(), age: 29, weight: 69.5 },
//! #   User { name: "negar".to_owned(), age: 26, weight: 61.0 }
//! # ]);
//! # Ok::<(), RsqliteError>(())
//! ```
#![allow(incomplete_features)]
#![feature(specialization)]

pub mod error;
pub mod bindable;
pub mod collectable;
pub mod iterable;

pub use error::RsqliteError;
pub use bindable::Bindable;
pub use iterable::Iterable;
pub use collectable::Collectable;

use core::ptr;
use libc::{c_int};
use sqlite3_sys as ffi;
use std::ffi::CString;

pub type Result<T> = std::result::Result<T, RsqliteError>;

pub struct Database {
    pub db: *mut ffi::sqlite3,
}

pub struct Statement {
    pub stmt: *mut ffi::sqlite3_stmt,
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

    /// Execute an sqlite query.
    ///
    /// It is expected that the query does to returns any data,
    /// if you need to return data, you should use `.query()`.
    pub fn execute(&self, sql: &str, params: impl Bindable) -> Result<()> {
        let statement = self.prepare(sql)?;
        params.bind(&statement, &mut 1)?;

        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        match retcode {
            ffi::SQLITE_DONE => Ok(()),
            other => Err(other.into()),
        }
    }

    /// Execute a query and collect the results.
    ///
    /// Your query must return the same column count as type `R`
    /// If the column index is out of range, the result is undefined.
    ///
    ///
    /// ```
    /// # use rsqlite::*;
    /// # let database = Database::open(":memory:")?;
    /// let result : (i32, String, f64) = database.collect("select 100, 'hello', 3.14;", ())?;
    /// # assert!(result == (100, "hello".to_owned(), 3.14));
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn collect<R>(&self, sql: &str, params: impl Bindable) -> Result<R>
    where
        R: Collectable,
    {
        let statement = self.prepare(sql)?;
        params.bind(&statement, &mut 1)?;

        let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };

        let result = match retcode {
            ffi::SQLITE_ROW => Ok(R::collect(&statement, &mut 0)),
            other => Err(other.into()),
        };

        let _ = unsafe { ffi::sqlite3_reset(statement.stmt) };

        result
    }


    /// iterate over multile rows of data using a colusure
    ///
    /// ```
    /// # use rsqlite::*;
    /// # let database = Database::open(":memory:")?;
    /// let mut sum = 0;
    /// database.iterate("select 2 union select 3", (), |x: i32| { sum += x; })?;
    /// // sum is 5
    /// # assert!(sum == 5);
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn iterate<T>(&self, sql: &str, params: impl Bindable, mut iterable: impl Iterable<T>) -> Result<()> {
        let statement = self.prepare(sql)?;
        params.bind(&statement, &mut 1)?;

        iterable.iterate(&statement)?;

        let _ = unsafe { ffi::sqlite3_reset(statement.stmt) };
        Ok(())
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

