use crate::{Context, EngineHandle, Term};
use std::ffi::CStr;
use swipl_fli as pl;

/// Implemented by all foreign predicates. Use the [`predicates`] macro to
/// generate types implementing this trait or [`predicate`] to implement it for
/// existing types.
///
/// # Safety
/// `EXTERN_FN` must be a pointer to a valid SWI-Prolog callback function as
/// specified in https://www.swi-prolog.org/pldoc/man?section=foreign-register-predicate.
pub unsafe trait Predicate {
    type Args<'a>: PredicateArgs;

    /// Name of the predicate as seen from Prolog
    const NAME: &'static CStr;
    /// Function pointer to the `extern "C" fn` callback function
    const EXTERN_FN: *const ();
    /// Flags passed to [`swipl_fli::PL_register_foreign`]
    const FLAGS: u32;
}

/// Internal trait for the args array passed to foreign predicates.
/// It would be simpler to define a `const ARITY: usize` on `PredicateMeta`, but
/// using the associated constant as an array size is not yet supported in
/// stable Rust.
pub trait PredicateArgs {
    type Raw;

    const ARITY: usize;

    /// # Safety
    /// The returned value must actually be valid for the duration of its
    /// lifetime.
    unsafe fn from_raw(_: Self::Raw) -> Self;
}

impl<const N: usize> PredicateArgs for [Term<'_>; N] {
    type Raw = [pl::term_t; N];

    const ARITY: usize = N;

    unsafe fn from_raw(raw: Self::Raw) -> Self {
        raw.map(Term::from_ptr)
    }
}

/// Generates foreign predicate types, implements [`Predicate`] for them and
/// generates the necessary `extern "C" fn` items. It does not generate
/// predicate implementations. You must implement [`Semidet`] or [`Nondet`] for
/// the generated types explicitly.
///
/// Each predicate can be declared as _semi-deterministic_ (`semidet`) or
/// _non-deterministic_ (`nondet`). _Deterministic_ predicates are a subset of
/// _semi-deterministic_ predicates and are trivial to implement in terms of the
/// `Semidet` trait, and are therefore left out.
///
/// For existing types use [`predicate`] instead.
///
/// * https://www.swi-prolog.org/pldoc/man?section=determinism
///
/// ```ignore
/// predicates! {
///     my_semidet_pred(t1, t2) semidet as MySemidetPred;
///     my_nondet_pred(t1, t2) nondet as MyNondetPred {
///         // struct fields
///     }
/// }
///
/// impl Semidet for MySemidetPred {
///     // ...
/// }
///
/// impl Nondet for MyNondetPred {
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! predicates {
    () => {};
    ($(#[$attr:meta])* $pub:vis $name:ident($($arg:ident),+) $det:ident as $type:ident; $($rest:tt)*) => {
        $(#[$attr])* $pub struct $type;
        $crate::predicate!($type as $name($($arg),+) $det);
        $crate::predicates!($($rest)*);
    };
    ($(#[$attr:meta])* $pub:vis $name:ident($($arg:ident),+) $det:ident as $type:ident { $($field:tt)* } $($rest:tt)*) => {
        $(#[$attr])* $pub struct $type { $($field)* }
        $crate::predicate!($type as $name($($arg),+) $det);
        $crate::predicates!($($rest)*);
    };
    ($(#[$attr:meta])* $pub:vis $name:ident($($arg:ident),+) $det:ident as $type:ident ( $($field:tt)* ); $($rest:tt)*) => {
        $(#[$attr])* $pub struct $type ( $($field)* );
        $crate::predicate!($type as $name($($arg),+) $det);
        $crate::predicates!($($rest)*);
    };
}

/// Implements [`Predicate`] for an existing type and generates the necessary
/// `extern "C" fn` item. See [`predicates`] for more info.
///
/// ```ignore
/// struct MyPredicate;
///
/// predicate!(MyPredicate as my_predicate(t1, t2) semidet);
///
/// impl Semidet for MyPredicate {
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! predicate {
    ($type:ty as $name:ident($($arg:ident),+) $det:ident) => {
        unsafe impl $crate::foreign::Predicate for $type {
            type Args<'a> = [$crate::Term<'a>; $crate::predicate!(@count $($arg),+)];

            const NAME: &'static ::std::ffi::CStr = unsafe {
                ::std::ffi::CStr::from_ptr(
                    concat!(stringify!($name), "\0").as_ptr() as *const _,
                )
            };
            const EXTERN_FN: *const () = ::paste::paste!([<$name $(_ $arg)+>]) as _;
            const FLAGS: u32 = $crate::predicate!(@flags $det);
        }

        $crate::predicate!(@extern $type as $name($($arg),+) $det);
    };
    (@count $_:tt, $($rest:tt),+) => { 1 + $crate::predicate!(@count $($rest),+) };
    (@count $_:tt) => { 1 };
    (@extern $type:ty as $name:ident($($arg:ident),+) semidet) => {
        ::paste::paste! {
            extern "C" fn [<$name $(_ $arg)+>](
                $($arg: ::swipl_fli::term_t),+,
            ) -> ::swipl_fli::foreign_t {
                unsafe { $crate::foreign::semidet_impl::<$type>([$($arg),+]) }
            }
        }
    };
    (@extern $type:ty as $name:ident($($arg:ident),+) nondet) => {
        ::paste::paste! {
            extern "C" fn [<$name $(_ $arg)+>](
                $($arg: ::swipl_fli::term_t),+,
                ctrl: ::swipl_fli::control_t,
            ) -> ::swipl_fli::foreign_t {
                unsafe { $crate::foreign::nondet_impl::<$type>([$($arg),+], ctrl) }
            }
        }
    };
    (@flags semidet) => { 0 };
    (@flags nondet) => { ::swipl_fli::PL_FA_NONDETERMINISTIC };
}

/// Implemented by semi-deterministic foreign predicates
pub trait Semidet: Predicate {
    /// May unify terms in `args` and return `true` for success, or return
    /// `false` for failure. Deterministic predicates are implemented by always
    /// returning `true` from this method.
    fn call(ctx: &impl Context, args: Self::Args<'_>) -> bool;
}

/// Implemented by non-deterministic foreign predicates
pub trait Nondet: Predicate {
    /// Initializes the state of the foreign predicate call. This is called once
    /// when the foreign predicate is called from Prolog. Multiple instances can
    /// exist at the same time.
    fn init(ctx: &impl Context) -> Self;

    /// Advances the foreign predicate to the next solution. The implementation
    /// may unify the terms in `args` with values and return `true` for success,
    /// or return `false` in which case the search will be terminated.
    fn next(&mut self, ctx: &impl Context, args: Self::Args<'_>) -> bool;
}

/// Generic wrapper implementation for semi-deterministic foreign predicates.
/// Only meant to be used from the [`predicate`] macro.
///
/// # Safety
/// * `args` must be an array of valid `term_t` handles.
pub unsafe fn semidet_impl<P: Semidet>(args: <P::Args<'_> as PredicateArgs>::Raw) -> pl::foreign_t {
    // Each call to a foreign predicate is wrapped in a PL_open_foreign_frame() and
    // PL_close_foreign_frame() pair.
    // * https://www.swi-prolog.org/pldoc/man?section=foreign-discard-term-t
    let ctx = unsafe { EngineHandle::assume_attached() };

    match P::call(&ctx, unsafe { P::Args::from_raw(args) }) {
        true => pl::TRUE as _,
        false => pl::FALSE as _,
    }
}

/// Generic wrapper implementation for non-deterministic foreign predicates.
/// Only meant to be used from the [`predicate`] macro.
///
/// # Safety
/// * `args` must be an array of valid `term_t` handles.
/// * `ctrl` must be a valid `control_t` value passed to the callback function
///   `P::EXTERN_FN` by SWI-Prolog. The implementation assumes that it contains
///   a valid address of the heap-allocated instance of `P`.
pub unsafe fn nondet_impl<P: Nondet>(
    args: <P::Args<'_> as PredicateArgs>::Raw,
    ctrl: pl::control_t,
) -> pl::foreign_t {
    // Each call to a foreign predicate is wrapped in a PL_open_foreign_frame() and
    // PL_close_foreign_frame() pair.
    // * https://www.swi-prolog.org/pldoc/man?section=foreign-discard-term-t
    let ctx = unsafe { EngineHandle::assume_attached() };

    match unsafe { pl::PL_foreign_control(ctrl) } as u32 {
        pl::PL_FIRST_CALL => {
            let mut state = P::init(&ctx);

            if state.next(&ctx, unsafe { P::Args::from_raw(args) }) {
                let boxed = Box::leak(Box::new(state));
                unsafe { pl::_PL_retry_address(boxed as *mut _ as _) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_REDO => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let state = unsafe { (state_ptr as *mut P).as_mut() }.unwrap();

            if state.next(&ctx, unsafe { P::Args::from_raw(args) }) {
                unsafe { pl::_PL_retry_address(state_ptr) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_PRUNED => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let _state = unsafe { Box::from_raw(state_ptr as *mut P) };

            pl::TRUE as _
        },
        c => panic!("unexpected value returned from PL_foreign_control: {c}"),
    }
}
