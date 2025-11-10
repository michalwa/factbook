use crate::{Atom, Functor, Record};
use std::marker::PhantomData;
use std::{fmt, slice};
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
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_variable%27)
    pub fn put_variable(self) -> Self {
        if unsafe { pl::PL_put_variable(self.ptr) } == 0 {
            panic!("PL_put_variable failed");
        }

        self
    }

    /// Puts an atom in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_atom%27)
    pub fn put_atom(self, atom: &Atom) -> Self {
        if unsafe { pl::PL_put_atom(self.ptr, atom.ptr) } == 0 {
            panic!("PL_put_atom failed");
        }

        self
    }

    /// Puts an atom with the given name in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_atom_chars%27)
    pub fn put_atom_chars(self, chars: &str) -> Self {
        if unsafe { pl::PL_put_atom_nchars(self.ptr, chars.len(), chars.as_ptr() as _) } == 0 {
            panic!("PL_put_atom_nchars failed");
        }

        self
    }

    /// Puts a compound term with the given functor and arguments in the term
    /// reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_functor%27)
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_unify_arg%27)
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

    /// Records the term into the Prolog database and returns a handle to it. The returned handle
    /// may be shared across threads.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_record%27)
    pub fn record(self) -> Record {
        Record {
            ptr: unsafe { pl::PL_record(self.ptr) },
        }
    }

    /// Copies a recorded term into the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_recorded%27)
    pub fn put_recorded(self, record: &Record) -> Self {
        if unsafe { pl::PL_recorded(record.ptr, self.ptr) } == 0 {
            panic!("PL_recorded failed");
        }

        self
    }

    /// Puts a value in the term reference
    /// * https://www.swi-prolog.org/pldoc/man?section=foreign-term-construct
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    ///
    /// let t1 = engine.new_term().put(1i32);
    /// let t2 = engine.new_term().put(1i64);
    /// let t3 = engine.new_term().put(1u64);
    ///
    /// assert!(t1.unify_with(t2));
    /// assert!(t2.unify_with(t3));
    /// ```
    pub fn put(self, value: impl ToTerm) -> Self {
        value.put_in(&self);
        self
    }

    /// Puts a term parsed from the given string in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_term_from_chars%27)
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    ///
    /// let t1 = term! { engine => foo(bar(foo), _) };
    /// let t2 = engine.new_term().put_parsed("foo(bar(_), foo)");
    ///
    /// assert!(t1.unify_with(t2));
    /// ```
    pub fn put_parsed(self, repr: &str) -> Self {
        if unsafe {
            pl::PL_put_term_from_chars(self.ptr, pl::REP_UTF8 as _, repr.len(), repr.as_ptr() as _)
        } == 0
        {
            // TODO: Return exception on failure
            panic!("PL_put_term_from_chars failed");
        }

        self
    }

    /// Returns the string representation of the atom stored in the term reference,
    /// or `None` if it's not an atom.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_get_atom_nchars%27)
    pub fn atom_chars(&self) -> Option<&str> {
        let mut len = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if unsafe { pl::PL_get_atom_nchars(self.ptr, &mut len as _, &mut chars as *mut _ as _) }
            == 0
        {
            return None;
        }

        Some(
            str::from_utf8(unsafe { slice::from_raw_parts(chars, len) })
                .expect("PL_get_atom_nchars returned invalid UTF-8"),
        )
    }

    /// Unifies the two terms and returns `true` on success or `false` on failure.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_unify%27)
    pub fn unify_with(self, other: Term) -> bool {
        unsafe { pl::PL_unify(self.ptr, other.ptr) != 0 }
    }

    /// Used with [`std::fmt::Display`] to obtain a canonical string
    /// representation of the term.
    /// * https://www.swi-prolog.org/pldoc/man?predicate=write_canonical/1
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    ///
    /// let t = engine.new_term().put_parsed("(1, 2)");
    /// assert_eq!(t.canonical().to_string(), "','(1,2)");
    /// ```
    pub fn canonical<'t>(&'t self) -> Canonical<'t> {
        Canonical(self)
    }

    fn write(&self, f: &mut fmt::Formatter, flags: u32) -> fmt::Result {
        let mut len = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if unsafe {
            pl::PL_get_nchars(
                self.ptr,
                &mut len as _,
                &mut chars as *mut _ as _,
                flags | pl::REP_UTF8 | pl::BUF_DISCARDABLE,
            )
        } == 0
        {
            panic!("PL_get_nchars failed");
        }

        let chars = str::from_utf8(unsafe { slice::from_raw_parts(chars, len) })
            .expect("PL_get_nchars returned invalid UTF-8");

        f.write_str(chars)
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, pl::CVT_WRITEQ)
    }
}

pub struct Canonical<'t>(&'t Term);

impl fmt::Display for Canonical<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.write(f, pl::CVT_WRITE_CANONICAL)
    }
}

/// Implemented by types that can be converted to Prolog values and put in a term reference
pub trait ToTerm {
    fn put_in(self, term: &Term);
}

macro_rules! impl_ToTerm {
    (|$value:ident: $type:ty, $term:ident| pl::$fn:ident($($args:tt)*)) => {
        impl ToTerm for $type {
            fn put_in(self, term: &Term) {
                let $value = self;
                let $term = term;
                if unsafe { pl::$fn($($args)*) } == 0 {
                    panic!(concat!(stringify!($fn), " failed"));
                }
            }
        }
    };
}

impl_ToTerm!(|v: bool, t| pl::PL_put_bool(t.ptr, v as _));
impl_ToTerm!(|v: &str, t| pl::PL_put_string_nchars(t.ptr, v.len(), v.as_ptr() as _));
impl_ToTerm!(|v: i32, t| pl::PL_put_integer(t.ptr, v));
impl_ToTerm!(|v: i64, t| pl::PL_put_int64(t.ptr, v));
impl_ToTerm!(|v: u64, t| pl::PL_put_uint64(t.ptr, v));
impl_ToTerm!(|v: f64, t| pl::PL_put_float(t.ptr, v));

#[cfg(test)]
mod test {
    #[test]
    fn fmt() {
        let engine = crate::test::SESSION.engine();
        let t = engine.new_term().put_parsed("foo(bar, (1, 2), \"hello\")");
        assert_eq!(t.to_string(), "foo(bar,(1,2),\"hello\")");
    }
}
