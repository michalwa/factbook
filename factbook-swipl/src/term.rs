use crate::Functor;
use std::marker::PhantomData;
use std::slice;
use swipl_fli::{self as pl, REP_UTF8};

/// Shared reference to a Prolog term
#[derive(Clone, Copy)]
pub struct Term {
    // Not `Send` because it's only valid within the context of the current thread engine
    _marker: PhantomData<*const ()>,
    ptr: pl::term_t,
}

impl Term {
    pub(crate) fn from_ptr(ptr: pl::term_t) -> Self {
        Self {
            _marker: Default::default(),
            ptr,
        }
    }

    pub fn ptr(self) -> pl::term_t {
        self.ptr
    }

    pub fn put_variable(self) -> Self {
        if unsafe { pl::PL_put_variable(self.ptr) } == 0 {
            panic!("PL_put_variable failed");
        }

        self
    }

    pub fn put_bool(self, value: bool) -> Self {
        if unsafe { pl::PL_put_bool(self.ptr, value as _) } == 0 {
            panic!("PL_put_bool failed");
        }

        self
    }

    pub fn put_atom_chars(self, chars: &str) -> Self {
        if unsafe { pl::PL_put_atom_nchars(self.ptr, chars.len(), chars.as_ptr() as _) } == 0 {
            panic!("PL_put_atom_nchars failed");
        }

        self
    }

    pub fn put_functor<const ARITY: usize>(
        self,
        functor: Functor<ARITY>,
        args: [Term; ARITY],
    ) -> Self {
        if unsafe { pl::PL_put_functor(self.ptr, functor.ptr) } == 0 {
            panic!("PL_put_functor failed");
        }

        for (i, arg) in args.into_iter().enumerate() {
            if unsafe { pl::PL_unify_arg_sz(i + 1, self.ptr, arg.ptr) } == 0 {
                panic!("PL_unify_arg_sz failed");
            }
        }

        self
    }

    pub fn put(self, repr: &str) -> Self {
        if unsafe {
            pl::PL_put_term_from_chars(self.ptr, REP_UTF8 as _, repr.len(), repr.as_ptr() as _)
        } == 0
        {
            // TODO: Return exception on failure
            panic!("PL_put_term_from_chars failed");
        }

        self
    }

    pub fn atom_chars(&self) -> &str {
        let mut len = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if unsafe { pl::PL_get_atom_nchars(self.ptr, &mut len as _, &mut chars as *mut _ as _) }
            == 0
        {
            panic!("PL_get_atom_nchars failed");
        }

        str::from_utf8(unsafe { slice::from_raw_parts(chars, len) })
            .expect("PL_get_atom_nchars returned invalid UTF-8")
    }

    pub fn unify_with(self, other: Term) -> bool {
        unsafe { pl::PL_unify(self.ptr, other.ptr) != 0 }
    }
}
