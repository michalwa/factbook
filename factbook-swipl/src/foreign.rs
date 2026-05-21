use crate::{Context, Term};
pub use factbook_swipl_macros::predicate;
use std::ffi::CStr;
use std::marker::PhantomData;
use swipl_fli as pl;

/// Re-exports for use by macros
pub mod ffi {
    pub use swipl_fli::{PL_FA_NONDETERMINISTIC, control_t, foreign_t, term_t};
}

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
        raw.map(|ptr| Term::from_ptr(ptr).unwrap())
    }
}

/// Implemented by semi-deterministic foreign predicates
pub trait Semidet: Predicate {
    /// May unify terms in `args` and return `true` for success, or return
    /// `false` for failure. Deterministic predicates are implemented by always
    /// returning `true` from this method.
    fn call(ctx: &mut impl Context, args: Self::Args<'_>) -> bool;
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
    fn next(&mut self, ctx: &mut impl Context, args: Self::Args<'_>) -> bool;
}

/// A context to statically grant foreign prediates access to an attached engine
struct ForeignContext {
    _marker: PhantomData<*const ()>, // !Send
}

unsafe impl Context for ForeignContext {}

/// Generic wrapper implementation for semi-deterministic foreign predicates.
/// Only meant to be used from the [`predicate`] macro.
///
/// # Safety
/// * `args` must be an array of valid `term_t` handles.
pub unsafe fn semidet_impl<P: Semidet>(args: <P::Args<'_> as PredicateArgs>::Raw) -> pl::foreign_t {
    // Each call to a foreign predicate is wrapped in a PL_open_foreign_frame() and
    // PL_close_foreign_frame() pair.
    // * https://www.swi-prolog.org/pldoc/man?section=foreign-discard-term-t
    let mut ctx = ForeignContext {
        _marker: Default::default(),
    };

    match P::call(&mut ctx, unsafe { P::Args::from_raw(args) }) {
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
    // Fixes a "bad alignment" error from SWI-Prolog for zero-sized types
    #[repr(align(4))]
    struct State<P>(P);

    // Each call to a foreign predicate is wrapped in a PL_open_foreign_frame() and
    // PL_close_foreign_frame() pair.
    // * https://www.swi-prolog.org/pldoc/man?section=foreign-discard-term-t
    let mut ctx = ForeignContext {
        _marker: Default::default(),
    };

    match unsafe { pl::PL_foreign_control(ctrl) } as u32 {
        pl::PL_FIRST_CALL => {
            let mut state = P::init(&ctx);

            if state.next(&mut ctx, unsafe { P::Args::from_raw(args) }) {
                let boxed = Box::leak(Box::new(State(state)));
                unsafe { pl::_PL_retry_address(boxed as *mut _ as _) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_REDO => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let state = unsafe { (state_ptr as *mut State<P>).as_mut() }.unwrap();

            if state.0.next(&mut ctx, unsafe { P::Args::from_raw(args) }) {
                unsafe { pl::_PL_retry_address(state_ptr) }
            } else {
                pl::FALSE as _
            }
        },
        pl::PL_PRUNED => {
            let state_ptr = unsafe { pl::PL_foreign_context_address(ctrl) };
            let _state = unsafe { Box::from_raw(state_ptr as *mut State<P>) };

            pl::TRUE as _
        },
        c => panic!("unexpected value returned from PL_foreign_control: {c}"),
    }
}
