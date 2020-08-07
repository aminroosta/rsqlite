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
database.iterate(
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
##
