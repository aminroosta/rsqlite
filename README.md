# rsqlite

This library is a zero-overhead, ergonamic wrapper over sqlite C api.

```rust
// creates a database file 'dbfile.db' if it does not exists.
let database = Database::open("dbfile.db")?;

// executes the query and creates a 'user' table
database.execute(r#"
   create table if not exists user (
      id integer primary key autoincrement not null,
      age int,
      name text,
      weight real
   );"#, ())?;

// inserts a new user record.
// binds the fields to '?' .
// note that only these types are allowed for bindings:
//     int32, i64, f64, &str, &[u8]
// use `&[u8]` to store blob data.
database.execute(
   "insert into user(age, name, weight) values(?, ?, ?)",
   (29, "amin", 69.5)
)?;
let name = String::from("negar");
database.execute(
   "insert into user(age, name, weight) values(?, ?, ?)",
   (26, name.as_str(), 61.0)
)?;


// slects from user table on a condition ( age > 27 ),
// and executes the closure for each row returned.
database.for_each(
    "select name, age, weight from user where age > ?", (27),
    |name: String, age: i32, weight: f64| {
        dbg!(name, age, weight);
    }
)?;

// selects the count(*) from user table
// you can extract a single culumn single row result to:
// i32, i64, f64, String, Box<[u8]>
let count: i32 = database.collect("select count(*) from user", ())?;

// you can also extract single row with multiple columns
let amin: (i32, String, f64) = database.collect(
    "select age, name, weight from user where name = ?", ("amin")
)?;

// this also works, the returned value will be automatically converted to String
let str_count: String = database.collect("select count(*) from user", ())?;

```

## Additional flags

You can pass additional open flags to SQLite:

```toml
[dependencies]
sqlite3-sys = "*"
```
```rust
use rsqlite::{ffi, Database};

let flags = ffi::SQLITE_READONLY;
let database = Database::open_with_flags("dbfile.db", flags)?;

// now you can only read from the database
let n: i32 = database.collect(
    "select a from table where something >= ?", (1))?;
```

## Prepared Statements

It is possible to retain and reuse statments, this will keep the query plan and might
increase the performance significantly if the statement is reused.
```rust
let mut statement = database.prepare("select age from user where age > ?")?;
// Database methods are simply implemented in terms of statements.
statement.for_each((27), |age: i32| {
    dbg!(age);
})?;

let age: i32 = database.prepare("select count(*) from user where age > ? limit 1")?
                       .collect((200))?;
```
## NULL values
If you have NULLABLE columes, you can use `Option<T>` to pass and collect the values.
```rust
// use `None` to insert NULL values
database.execute("insert into user(name, age) values (?,?)", (None::<&str>, 20))?;

// use Option<T> to collect them back
let (name, age) : (Option<String>, i32) =
                      database.collect("select name, age from user limit 1", ())?;
assert!((name, age) == (None, 20));

// an empty result set, would also be treated as None
let name : Option<String> = database.collect("select name from user where age = ?", (200))?;
assert!(name == None);
```
## Type conversions

implsit type convertions in sqlite follow this table:

|Internal Type|Requested Type|Conversion
|-------------|--------------|----------
|NULL         |i32/i64 	     |Result is 0
|NULL         |f64   	     |Result is 0.0
|NULL         |String        |Result is empty `String::new()`
|NULL         |Box<[u8]>     |Result is empty `Box::new([])`
|INTEGER      |FLOAT 	     |Convert from integer to float
|INTEGER      |TEXT 	     |ASCII rendering of the integer
|INTEGER      |BLOB 	     |Same as INTEGER->TEXT
|FLOAT        |INTEGER 	     |CAST to INTEGER
|FLOAT        |TEXT 	     |ASCII rendering of the float
|FLOAT        |BLOB 	     |CAST to BLOB
|TEXT         |INTEGER 	     |CAST to INTEGER
|TEXT         |FLOAT 	     |CAST to REAL
|TEXT         |BLOB 	     |No change
|BLOB         |INTEGER 	     |CAST to INTEGER
|BLOB         |FLOAT 	     |CAST to REAL
|BLOB         |TEXT 	     |Add a zero terminator if needed

i.e if you collect a `NULL` column as `i32`, you'll get `0`.

## Transactions
You can use transactions with `begin`, `commit` and `rollback` commands.
## use rsqlite::*;
## let database = Database::open(":memory:")?;
## database.execute("create table user (name text, age int)", ())?;

db.execute("begin", ())?; // begin a transaction
```rust
