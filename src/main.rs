#![allow(incomplete_features)]
#![feature(specialization)]

pub mod error;

use core::ptr;
use error::RsqliteError;
use libc::c_int;
use sqlite3_sys as ffi;
use std::ffi::{CString};

type Result<T> = std::result::Result<T, RsqliteError>;

pub struct Database {
    pub db: *mut ffi::sqlite3,
}

impl Database {
    pub fn open(path: &str) -> Result<Database> {
        let flags = ffi::SQLITE_OPEN_READWRITE | ffi::SQLITE_OPEN_CREATE;
        Database::open_with_flags(path, flags)
    }
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
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_close(self.db);
            self.db = ptr::null_mut();
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn test_can_open_db() {
    let _database = Database::open(":memory:").unwrap();
}
