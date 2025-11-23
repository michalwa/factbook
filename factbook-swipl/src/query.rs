use crate::term::{Exception, Term, Terms};
use crate::{Context, get_exception, get_module};
pub use factbook_swipl_macros::open_query;
use swipl_fli as pl;

/// An open Prolog query.
///
/// Exclusively borrows a [`Context`].
///
/// Dropping this will call `PL_cut_query`, which will not discard bindings made
/// by the query by default.
/// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_cut_query%27)
pub struct Query<'a, const ARITY: usize> {
    args: Terms<'a, ARITY>,
    ptr: pl::qid_t,
}

impl<'a, const ARITY: usize> Query<'a, ARITY> {
    pub fn new<'m, C, F>(
        // Take a mutable reference to the context to discourage opening multiple queries at once,
        // which is not supported
        ctx: &'a mut C,
        pred: &str,
        args_fn: F,
        module: impl Into<Option<&'m str>>,
    ) -> Result<Self, Exception<'a>>
    where
        C: Context + ?Sized,
        F: FnOnce(&C, &[Term; ARITY]),
    {
        let module = get_module(ctx, module.into());
        let predicate = unsafe { pl::PL_pred(ctx.functor::<ARITY>(pred).ptr, module) };
        let args = ctx.new_terms();
        args_fn(ctx, &args);

        let ptr = unsafe {
            pl::PL_open_query(
                module,
                (pl::PL_Q_NODEBUG | pl::PL_Q_CATCH_EXCEPTION | pl::PL_Q_EXT_STATUS) as _,
                predicate,
                args.ptr(),
            )
        };

        if ptr.is_null() {
            Err(get_exception(ctx).unwrap())
        } else {
            Ok(Self { args, ptr })
        }
    }

    /// Advances the query to the next solution, returning `true` on success or
    /// `false` if the query could not be proven
    pub fn next_solution(&self) -> Result<Option<&[Term<'a>; ARITY]>, Exception<'a>> {
        const PL_S_FALSE: i32 = pl::PL_S_FALSE as _;
        const PL_S_TRUE: i32 = pl::PL_S_TRUE as _;
        const PL_S_LAST: i32 = pl::PL_S_LAST as _;

        match unsafe { pl::PL_next_solution(self.ptr) } {
            pl::PL_S_NOT_INNER => panic!("not the innermost query"),
            pl::PL_S_EXCEPTION => Err(Term::from_ptr(unsafe { pl::PL_exception(self.ptr) })
                .unwrap()
                .into()),
            PL_S_FALSE => Ok(None),
            PL_S_TRUE | PL_S_LAST => Ok(Some(&self.args)),
            s => panic!("unsupported query status: {s}"),
        }
    }

    /// Closes the query, destroying all data and bindings created by it.
    ///
    /// If you don't care about bindings being left undiscarded, you can simply
    /// drop the [`Query`].
    ///
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_close_query%27)
    pub fn close(self) {
        unsafe { pl::PL_close_query(self.ptr) };
        std::mem::forget(self)
    }
}

impl<const ARITY: usize> Drop for Query<'_, ARITY> {
    fn drop(&mut self) {
        unsafe { pl::PL_cut_query(self.ptr) };
    }
}
