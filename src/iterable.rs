//! Iterable types are expected to iterate over the sqlite rows
use super::{Collectable, Statement};
use libc::c_int;

/// This library implements `Iterable` for any `FnMut<T1,...> -> R`  
/// Note that the return type `R` comes first in the generic parameter list
pub trait Iterable<R, T> {
    fn iterate(&mut self, statement: &mut Statement, index: &mut c_int) -> R;
}

macro_rules! iterable_fn_mut {
    ($($name:ident),+) => (
        impl<F, R, $($name),+> Iterable<R, ($($name,)+)> for F where
            F: FnMut($($name),+) -> R,
            $($name: Collectable),+
        {
            fn iterate(&mut self, statement: &mut Statement, index: &mut c_int) -> R {
                (*self)($($name::collect(statement, index)),+)
            }
        }
    );
}

iterable_fn_mut!(T0);
iterable_fn_mut!(T0, T1);
iterable_fn_mut!(T0, T1, T2);
iterable_fn_mut!(T0, T1, T2, T4);
iterable_fn_mut!(T0, T1, T2, T4, T5);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7, T8);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7, T8, T9);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11);
iterable_fn_mut!(T0, T1, T2, T4, T5, T6, T7, T8, T9, T10, T11, T12);
