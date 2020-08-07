//! This library is a zero-overhead, ergonamic wrapper over sqlite C api.
//!
//! ```
//! # use rsqlite::*;
//! // creates a database file 'dbfile.db' if it does not exists.
//! # /*
//! let mut database = Database::open("dbfile.db")?;
//! # */
//! # let mut database = Database::open(":memory:")?;
//!
//! // executes the query and creates a 'user' table
//! database.execute(r#"
//!    create table if not exists user (
//!       id integer primary key autoincrement not null,
//!       age int,
//!       name text,
//!       weight real
//!    );"#, ())?;
//!
//! // inserts a new user record.
//! // binds the fields to '?' .
//! // note that only these types are allowed for bindings:
//! //     int32, i64, f64, &str, &[u8]
//! // use `&[u8]` to store blob data.
//! database.execute(
//!    "insert into user(age, name, weight) values(?, ?, ?)",
//!    (29, "amin", 69.5)
//! )?;
//! let name = String::from("negar");
//! database.execute(
//!    "insert into user(age, name, weight) values(?, ?, ?)",
//!    (26, name.as_str(), 61.0)
//! )?;
//!
//! # #[derive(PartialEq, Debug)]
//! # struct User { name: String, age: i32, weight: f64 };
//! # let mut users = vec![];
//!
//! // slects from user table on a condition ( age > 27 ),
//! // and executes the closure for each row returned.
//! database.iterate(
//!     "select name, age, weight from user where age > ?", (27),
//!     |name: String, age: i32, weight: f64| {
//! # /*
//!         dbg!(name, age, weight);
//! # */
//! #       users.push(User { name, age, weight });
//!     }
//! )?;
//! # assert!(users == vec![
//! #   User { name: "amin".to_owned(), age: 29, weight: 69.5 },
//! # ]);
//!
//! // selects the count(*) from user table
//! // you can extract a single culumn single row result to:
//! // i32, i64, f64, String, Box<[u8]>
//! let count: i32 = database.collect("select count(*) from user", ())?;
//! # assert!(count == 2);
//!
//! // you can also extract single row with multiple columns
//! let amin: (i32, String, f64) = database.collect(
//!     "select age, name, weight from user where name = ?", ("amin")
//! )?;
//!
//! // this also works, the returned value will be automatically converted to String
//! let str_count: String = database.collect("select count(*) from user", ())?;
//! # assert!(str_count == "2");
//!
//! # Ok::<(), RsqliteError>(())
//! ```
//!
//! # Additional flags
//!
//! You can pass additional open flags to SQLite:
//! 
//! ```toml
//! [dependencies]
//! sqlite3-sys = "*"
//! ```
//! ```no_run
//! use rsqlite::{ffi, Database};
//! # use rsqlite::RsqliteError;
//!
//! let flags = ffi::SQLITE_READONLY;
//! let mut database = Database::open_with_flags("dbfile.db", flags)?;
//!
//! // now you can only read from the database
//! let n: i32 = database.collect(
//!     "select a from table where something >= ?", (1))?;
//! # Ok::<(), RsqliteError>(())
//! ```
//! #

pub mod bindable;
pub mod collectable;
pub mod error;
pub mod iterable;

pub use bindable::Bindable;
pub use collectable::Collectable;
pub use error::RsqliteError;
pub use iterable::Iterable;
pub use sqlite3_sys as ffi;

use core::ptr;
use libc::c_int;
use std::ffi::CString;
use std::marker::PhantomData;

pub type Result<T> = std::result::Result<T, RsqliteError>;

pub struct Database {
    pub db: *mut ffi::sqlite3,
}

pub struct Statement<'a> {
    pub stmt: *mut ffi::sqlite3_stmt,
    _marker : PhantomData<&'a Database>,
}

impl Database {
    /// open an existing sqlite3 database or create a new one.
    ///
    /// ```
    /// # use rsqlite::*;
    /// let mut database = Database::open(":memory:")?;
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
    /// let mut database = Database::open_with_flags(
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
    /// # let mut database = Database::open(":memory:")?;
    /// let statement = database.prepare("select 1+2;")?;
    /// # assert!(!statement.stmt.is_null());
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn prepare(&mut self, sql: &str) -> Result<Statement<'_>> {
        let sql = CString::new(sql)?;
        let mut stmt = ptr::null_mut();
        let len = sql.as_bytes_with_nul().len() as i32;
        let retcode = unsafe {
            ffi::sqlite3_prepare_v2(self.db, sql.as_ptr(), len, &mut stmt, ptr::null_mut())
        };

        let statement = Statement { stmt, _marker: PhantomData };
        match retcode {
            ffi::SQLITE_OK => Ok(statement),
            other => Err(other.into()),
        }
    }

    /// Execute an sqlite query.
    ///
    /// It is expected that the query does to returns any data,
    /// if you need to return data, you should use `.query()`.
    pub fn execute(&mut self, sql: &str, params: impl Bindable) -> Result<()> {
        let mut statement = self.prepare(sql)?;
        params.bind(&mut statement, &mut 1)?;

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
    /// # let mut database = Database::open(":memory:")?;
    /// let result : (i32, String, f64) = database.collect("select 100, 'hello', 3.14;", ())?;
    /// # assert!(result == (100, "hello".to_owned(), 3.14));
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn collect<R>(&mut self, sql: &str, params: impl Bindable) -> Result<R>
    where
        R: Collectable,
    {
        let mut statement = self.prepare(sql)?;
        params.bind(&mut statement, &mut 1)?;

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
    /// # let mut database = Database::open(":memory:")?;
    /// let mut sum = 0;
    /// database.iterate("select 2 union select 3", (), |x: i32| { sum += x; })?;
    /// // sum is 5
    /// # assert!(sum == 5);
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn iterate<T>(
        &mut self,
        sql: &str,
        params: impl Bindable,
        mut iterable: impl Iterable<T>,
    ) -> Result<()> {
        let mut statement = self.prepare(sql)?;
        params.bind(&mut statement, &mut 1)?;

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

impl<'a> Drop for Statement<'a> {
    /// closes `*mut sqlite3_statement` handle on Drop
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_finalize(self.stmt);
            self.stmt = ptr::null_mut();
        }
    }
}
