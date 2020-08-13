//! This library is a zero-overhead, ergonamic wrapper over sqlite C api.
//!
//! ```
//! # use rsqlite::*;
//! // creates a database file 'dbfile.db' if it does not exists.
//! # /*
//! let database = Database::open("dbfile.db")?;
//! # */
//! # let database = Database::open(":memory:")?;
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
//! database.for_each(
//!     "select name, age, weight from user where age > ?", (27),
//!     |name: String, age: i32, weight: f64| {
//! #       users.push(User { name: name.clone(), age, weight });
//!         dbg!(name, age, weight);
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
//! # assert!(amin == (29, "amin".to_owned(), 69.5));
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
//! let database = Database::open_with_flags("dbfile.db", flags)?;
//!
//! // now you can only read from the database
//! let n: i32 = database.collect(
//!     "select a from table where something >= ?", (1))?;
//! # Ok::<(), RsqliteError>(())
//! ```
//!
//! # Prepared Statements
//!
//! It is possible to retain and reuse statments, this will keep the query plan and might
//! increase the performance significantly if the statement is reused.
//! ```
//! # use rsqlite::*;
//! # let database = Database::open(":memory:")?;
//! # database.execute("create table user (name text, age int)", ())?;
//! let mut statement = database.prepare("select age from user where age > ?")?;
//! // Database methods are simply implemented in terms of statements.
//! statement.for_each((27), |age: i32| {
//!     dbg!(age);
//! })?;
//!
//! let age: i32 = database.prepare("select count(*) from user where age > ? limit 1")?
//!                        .collect((200))?;
//! # Ok::<(), RsqliteError>(())
//! ```
//! # NULL values
//! If you have NULLABLE columes, you can use `Option<T>` to pass and collect the values.
//! ```
//! # use rsqlite::*;
//! # let database = Database::open(":memory:")?;
//! # database.execute("create table user (name text, age int)", ())?;
//! // use `None` to insert NULL values
//! database.execute("insert into user(name, age) values (?,?)", (None::<&str>, 20))?;
//!
//! // use Option<T> to collect them back
//! let (name, age) : (Option<String>, i32) =
//!                       database.collect("select name, age from user limit 1", ())?;
//! assert!((name, age) == (None, 20));
//!
//! // an empty result set, would also be treated as None
//! let name : Option<String> = database.collect("select name from user where age = ?", (200))?;
//! assert!(name == None);
//! # Ok::<(), RsqliteError>(())
//! ```
//! # Type conversions
//!
//! implsit type convertions in sqlite follow this table:
//! for example, if you collect a `NULL` column as `i32`, you'll get `0`.
//!
//! |Internal Type|Requested Type|Conversion
//! |-------------|--------------|----------
//! |NULL         |i32/i64 	     |Result is 0
//! |NULL         |f64   	     |Result is 0.0
//! |NULL         |String        |Result is empty `String::new()`
//! |NULL         |Box<[u8]>     |Result is empty `Box::new([])`
//! |INTEGER      |f64   	     |Convert from integer to f64
//! |INTEGER      |String        |ASCII rendering of the integer
//! |INTEGER      |Box<[u8]>     |Same as INTEGER->String
//! |FLOAT        |i32/i64 	     |CAST to INTEGER
//! |FLOAT        |String        |ASCII rendering of the float
//! |FLOAT        |Box<[u8]>     |CAST to [u8]
//! |TEXT         |i32/i64 	     |CAST to i32/i64
//! |TEXT         |f64   	     |CAST to f64  
//! |TEXT         |Box<[u8]>     |No change
//! |BLOB         |i32/i64 	     |CAST to i32/i64
//! |BLOB         |f64   	     |CAST to f64
//! |BLOB         |String        |No change
//!
//!
//! # Transactions
//! You can use transactions with `begin`, `commit` and `rollback` commands.
//!
//! ```
//! # use rsqlite::*;
//! # let database = Database::open(":memory:")?;
//! # database.execute("create table user (name text, age int)", ())?;
//!
//! database.execute("begin", ())?;    // begin a transaction ...
//! let mut statement = database.prepare("insert into user(name, age) values (?, ?)")?;
//! // insert 10 users using a prepared statement
//! for age in 0..10 {
//!   let name = format!("user-{}", age);
//!   statement.execute((name.as_str(), age))?;
//! }
//! database.execute("commit", ())?;   // commit all the changes
//!
//! database.execute("begin", ())?;    // begin another transaction ...
//! database.execute("delete from user where age > ?", (3))?;
//! database.execute("rollback", ())?; // cancel this transaction
//!
//! let sum_age : i32 = database.collect("select sum(age) from user", ())?;
//! assert!(sum_age == 45);
//! # Ok::<(), RsqliteError>(())
//! ```
//!
//! # Blob
//! Use `&[u8]` to store and `Box<[u8]>` to retrive blob data.
//!
//! ```
//! # use rsqlite::*;
//! # let database = Database::open(":memory:")?;
//! database.execute("create table user (name TEXT, numbers BLOB)", ())?;
//!
//! let numbers = vec![1, 1, 2, 3, 5];
//! database.execute("insert into user values (?, ?)", ("amin", numbers.as_slice()))?;
//! let stored_numbers : Box<[u8]> =
//!          database.collect("select numbers from user where name = ?", ("amin"))?;
//! assert!(numbers.as_slice() == stored_numbers.as_ref());
//! # Ok::<(), RsqliteError>(())
//! ```
//!
//! ## License
//!
//! MIT license - http://www.opensource.org/licenses/mit-license.php

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
    column_count: c_int,
    _marker: PhantomData<&'a ()>,
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
    pub fn prepare(&self, sql: &str) -> Result<Statement<'_>> {
        let sql = CString::new(sql)?;
        let mut stmt = ptr::null_mut();
        let len = sql.as_bytes_with_nul().len() as i32;
        let retcode = unsafe {
            ffi::sqlite3_prepare_v2(self.db, sql.as_ptr(), len, &mut stmt, ptr::null_mut())
        };

        match retcode {
            ffi::SQLITE_OK => Ok(Statement {
                column_count: unsafe { ffi::sqlite3_column_count(stmt) },
                stmt,
                _marker: PhantomData,
            }),
            other => {
                unsafe {
                    ffi::sqlite3_finalize(stmt);
                }
                Err(other.into())
            }
        }
    }

    /// Execute an sqlite query.
    ///
    /// It is expected that the query does to returns any data,
    /// if you need to return data, you should use `.query()`.
    pub fn execute(&self, sql: &str, params: impl Bindable) -> Result<()> {
        let mut statement = self.prepare(sql)?;
        statement.execute(params)
    }

    /// Execute a query and collect the results.
    ///
    /// Your query must return the same column count as type `R`
    /// If the column index is out of range, you will get `Err(SQLITE_RANGE)`
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
        let mut statement = self.prepare(sql)?;
        statement.collect(params)
    }

    /// for_each iterates over multile rows of data using a colusure
    ///
    /// ```
    /// # use rsqlite::*;
    /// # let database = Database::open(":memory:")?;
    /// let mut sum = 0;
    /// database.for_each("select 2 union select 3", (), |x: i32| { sum += x; })?;
    /// // sum is 5
    /// # assert!(sum == 5);
    /// # Ok::<(), RsqliteError>(())
    /// ```
    pub fn for_each<T>(
        &self,
        sql: &str,
        params: impl Bindable,
        iterable: impl Iterable<(), T>,
    ) -> Result<()> {
        let mut statement = self.prepare(sql)?;
        statement.for_each(params, iterable)
    }
}

impl<'a> Statement<'a> {
    pub fn execute(&mut self, params: impl Bindable) -> Result<()> {
        params.bind(self, &mut 1)?;

        let retcode = unsafe { ffi::sqlite3_step(self.stmt) };

        let result = match retcode {
            ffi::SQLITE_DONE => Ok(()),
            other => Err(other.into()),
        };

        let _ = unsafe { ffi::sqlite3_reset(self.stmt) };
        result
    }

    pub fn collect<R>(&mut self, params: impl Bindable) -> Result<R>
    where
        R: Collectable,
    {
        if R::columns_needed() > self.column_count {
            return Err(ffi::SQLITE_RANGE.into());
        }
        params.bind(self, &mut 1)?;

        let result = R::step_and_collect(self);

        let _ = unsafe { ffi::sqlite3_reset(self.stmt) };
        result
    }

    pub fn for_each<I, T>(&mut self, params: impl Bindable, mut iterable: I) -> Result<()>
    where
        I: Iterable<(), T>,
    {
        if I::columns_needed() > self.column_count {
            return Err(ffi::SQLITE_RANGE.into());
        }
        params.bind(self, &mut 1)?;

        let result = loop {
            let retcode = unsafe { ffi::sqlite3_step(self.stmt) };
            let mut index = 0;

            match retcode {
                ffi::SQLITE_ROW => iterable.iterate(self, &mut index),
                ffi::SQLITE_DONE => break Ok(()),
                other => break Err(other.into()),
            };
        };

        let _ = unsafe { ffi::sqlite3_reset(self.stmt) };
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

impl<'a> Drop for Statement<'a> {
    /// closes `*mut sqlite3_statement` handle on Drop
    fn drop(&mut self) {
        unsafe {
            ffi::sqlite3_finalize(self.stmt);
            self.stmt = ptr::null_mut();
        }
    }
}
