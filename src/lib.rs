//! # Ergonamic sqlite library with zero overhead.
//! 
//! ```
//! use rsqlite::{Database, Result}
//!
//! fn example() -> Result<()> {
//!     let database = Database::open(":memory:")?;
//!
//!     Ok(())
//! }
//! ```
#![allow(incomplete_features)]
#![feature(specialization)]

pub mod error;
pub use error::RsqliteError;

use core::ptr;
use libc::c_int;
use sqlite3_sys as ffi;
use std::ffi::{CString};

pub type Result<T> = std::result::Result<T, RsqliteError>;

pub struct Database {
    pub db: *mut ffi::sqlite3,
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
