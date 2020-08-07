//! Iterable types are expected to iterate over the sqlite rows
use super::{Collectable, Statement};
use libc::c_int;

/// This library implements `Iterable` for any `FnMut<T1,...>`
/// Other types could be implemented in terms of `FnMut`
///
/// ```no_compile
/// # use rsqlite::*;
/// #[derive(PartialEq)]
/// struct User { name: String, age: i32 };
///
/// // implement iterable in term of closures for your own types
/// impl<> Iterable<User> for &mut Vec<User> {
///     fn iterate(&mut self, statement: &mut Statement) -> Result<()> {
///         (|name: String, age: i32| {
///             self.push(User { name, age });
///         })
///         .iterate(statement)
///     }
/// }
///
/// let mut database = Database::open(":memory:")?;
/// database.execute("create table user (name text, age int)", ())?;
/// database.execute("insert into user(name, age) values(?, ?)", ("amin", 29))?;
/// database.execute("insert into user(name, age) values(?, ?)", ("negar", 26))?;
///
/// let mut users : Vec<User> = vec![];
/// database.iterate("select name, age from user", (), &mut users);
/// //                       ^^^^  ^^^
/// // note the order of columns must match your Iterable implementation
///
/// assert!(users == vec![
///   User { name: "amin".to_owned(), age: 29 },
///   User { name: "negar".to_owned(), age: 26 }
/// ]);
/// # Ok::<(), RsqliteError>(())
/// ```
pub trait Iterable<R, T> {
    fn iterate(&mut self, statement: &mut Statement, index: &mut c_int) -> R;
}

macro_rules! iterable {
    ($($name:ident),+) => (
        impl<F, R, $($name),+> Iterable<R, ($($name),+,)> for F where
            F: FnMut($($name),+) -> R,
            $($name: Collectable),+
        {
            fn iterate(&mut self, statement: &mut Statement, index: &mut c_int) -> R {
                (*self)($($name::collect(statement, index)),+)
            }
        }
    );
}

iterable!(T0);
iterable!(T0, T1);
iterable!(T0, T1, T2);
iterable!(T0, T1, T2, T4);
iterable!(T0, T1, T2, T4, T5);
iterable!(T0, T1, T2, T4, T5, T6);
iterable!(T0, T1, T2, T4, T5, T6, T7);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11, T12);
