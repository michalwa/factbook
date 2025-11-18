use crate::{Context, EngineHandle, Term};
use std::ffi::CStr;
use swipl_fli as pl;

/// Implemented by non-deterministic foreign predicates
pub trait Nondet: NondetMeta {
    /// Initializes the state of the foreign predicate call. This is called once
    /// when the foreign predicate is called from Prolog. Multiple instances can
    /// exist at the same time.
    fn init(ctx: &impl Context) -> Self;

    /// Advances the foreign predicate to the next solution. The implementation
    /// may unify the terms in `args` with values and return `true` for success,
    /// or return `false` in which case the search will be terminated.
    fn next(&mut self, ctx: &impl Context, args: Self::Args<'_>) -> bool;
}

/// Implemented by the [`impl_nondet`] macro
pub unsafe trait NondetMeta {
    type Args<'a>: PredicateArgs;

    const NAME: &'static CStr;
    const EXTERN_FN: *const ();
}

/// Internal trait for the args array passed to foreign predicates to work
/// around using `NondetMeta::ARITY` as an array size, which is not supported in
/// stable Rust currently
pub trait PredicateArgs {
    type Raw;

    const ARITY: usize;

    unsafe fn from_raw(_: Self::Raw) -> Self;
}

impl<const N: usize> PredicateArgs for [Term<'_>; N] {
    type Raw = [pl::term_t; N];

    const ARITY: usize = N;

    unsafe fn from_raw(raw: Self::Raw) -> Self {
        raw.map(Term::from_ptr)
    }
}

/// Implements `NondetMeta` related to a non-deterministic foreign predicate and
/// generates the required `extern "C" fn` item.
///
/// ```ignore
/// struct MyNondetPred;
///
/// impl_nondet!(MyNondetPred as my_nondet_pred(t1, t2));
///
/// impl Nondet for MyNondetPred {
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! impl_nondet {
    ($type:ty as $name:ident($($arg:ident),+)) => {
        unsafe impl $crate::foreign::NondetMeta for $type {
            type Args<'a> = [$crate::Term<'a>; $crate::impl_nondet!(@count $($arg),+)];

            const NAME: &'static std::ffi::CStr = unsafe {
                std::ffi::CStr::from_ptr(
                    concat!(stringify!($name), "\0").as_ptr() as *const _,
                )
            };
            const EXTERN_FN: *const () = $name as _;
        }

        extern "C" fn $name(
            $($arg: ::swipl_fli::term_t),+,
            ctrl: ::swipl_fli::control_t,
        ) -> ::swipl_fli::foreign_t {
            unsafe { $crate::foreign::nondet_impl::<$type>([$($arg),+], ctrl) }
        }
    };
    (@count $_:tt, $($rest:tt),+) => { 1 + $crate::impl_nondet!(@count $($rest),+) };
    (@count $_:tt) => { 1 };
}

/// Generic wrapper implementation for non-deterministic foreign predicates.
/// Only meant to be used from the [`impl_nondet`] macro.
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
            let _state = unsafe { Box::from_raw(state_ptr as _) };

            pl::TRUE as _
        },
        c => panic!("unexpected value returned from PL_foreign_control: {c}"),
    }
}
