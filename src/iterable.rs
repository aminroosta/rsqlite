use super::{Statement, Collectable, Result};

use sqlite3_sys as ffi;

pub trait Iterable<T> {
    fn iterate(&mut self, statement: &Statement) -> Result<()>;
}

macro_rules! iterable {
    ( $($name:ident),+) => (
        impl<F, $($name),+> Iterable<($($name),+,)> for F where
            F: FnMut($($name),+) -> (),
            $($name: Collectable),+
        {
            fn iterate(&mut self, statement: &Statement) -> Result<()> {
                loop {
                    let retcode = unsafe { ffi::sqlite3_step(statement.stmt) };
                    let mut index = 0;

                    match retcode {
                        ffi::SQLITE_ROW => (*self)(
                            $($name::collect(statement, &mut index)),+
                        ),
                        ffi::SQLITE_DONE => break Ok(()),
                        other => break Err(other.into()),
                    };
                }
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
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14);
iterable!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15);

