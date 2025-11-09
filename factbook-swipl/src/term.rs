use crate::{Functor, Record};
use std::marker::PhantomData;
use std::slice;
use swipl_fli as pl;

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

    /// Resets this term reference to a variable
    /// https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_variable%27)
    pub fn put_variable(self) -> Self {
        if unsafe { pl::PL_put_variable(self.ptr) } == 0 {
            panic!("PL_put_variable failed");
        }

        self
    }

    /// Puts one of the atoms `true` or `false` in the term reference
    /// https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_bool%27)
    pub fn put_bool(self, value: bool) -> Self {
        if unsafe { pl::PL_put_bool(self.ptr, value as _) } == 0 {
            panic!("PL_put_bool failed");
        }

        self
    }

    /// Puts an atom with the given name in the term reference
    /// https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_atom_chars%27)
    pub fn put_atom_chars(self, chars: &str) -> Self {
        if unsafe { pl::PL_put_atom_nchars(self.ptr, chars.len(), chars.as_ptr() as _) } == 0 {
            panic!("PL_put_atom_nchars failed");
        }

        self
    }

    /// Puts a compound term with the given functor and arguments in the term
    /// reference https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_functor%27)
    /// https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_unify_arg%27)
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

    pub fn put_recorded(self, record: &Record) -> Self {
        if unsafe { pl::PL_recorded(record.ptr, self.ptr) } == 0 {
            panic!("PL_recorded failed");
        }

        self
    }

    /// Puts a term parsed from the given string in the term reference
    /// https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_term_from_chars%27)
    ///
    /// ```
    /// use factbook_swipl::*;
    ///
    /// let session = Session::init(None).unwrap();
    /// let engine = session.engine();
    ///
    /// let t1 = term! { engine => foo(bar(foo), _) };
    /// let t2 = engine.new_term().put("foo(bar(_), foo)");
    ///
    /// assert!(t1.unify_with(t2));
    /// ```
    pub fn put(self, repr: &str) -> Self {
        if unsafe {
            pl::PL_put_term_from_chars(self.ptr, pl::REP_UTF8 as _, repr.len(), repr.as_ptr() as _)
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

    pub fn record(self) -> Record {
        Record {
            ptr: unsafe { pl::PL_record(self.ptr) },
        }
    }
}
