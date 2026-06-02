use crate::{Atom, Context, ExternalRecord, Functor, RawFunctor, Record, term};
use num_enum::TryFromPrimitive;
use std::marker::PhantomData;
use std::num::NonZero;
use std::ops::Deref;
use std::{fmt, slice};
use swipl_fli as pl;

/// Reference to a Prolog term.
///
/// The lifetime `'a`, for the duration of which the term reference is valid, is
/// the lifetime of the enclosing context, e.g. a [`Frame`](crate::Frame). When
/// the context is dropped it may clean up term references created within it.
#[derive(Clone, Copy)]
pub struct Term<'a> {
    // Not `Send` because it's only valid within the context of the current thread engine
    _marker: PhantomData<*const ()>,
    _lifetime: PhantomData<&'a ()>,
    pub(crate) ptr: NonZero<pl::term_t>,
}

impl<'a> Term<'a> {
    pub(crate) fn from_ptr(ptr: pl::term_t) -> Option<Self> {
        NonZero::new(ptr).map(|ptr| Self {
            _marker: Default::default(),
            _lifetime: Default::default(),
            ptr,
        })
    }

    /// Resets this term reference to a variable
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_variable%27)
    pub fn put_variable(self) -> Self {
        if !unsafe { pl::PL_put_variable(self.ptr.get()) } {
            panic!("PL_put_variable failed");
        }

        self
    }

    /// Puts an atom with the given name in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_atom_chars%27)
    pub fn put_atom_chars(self, chars: &str) -> Self {
        if !unsafe { pl::PL_put_atom_nchars(self.ptr.get(), chars.len(), chars.as_ptr() as _) } {
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
        args: [Self; ARITY],
    ) -> Self {
        if !unsafe { pl::PL_put_functor(self.ptr.get(), functor.0.ptr.get()) } {
            panic!("PL_put_functor failed");
        }

        for (i, arg) in args.into_iter().enumerate() {
            if !unsafe { pl::PL_unify_arg_sz(i + 1, self.ptr.get(), arg.ptr.get()) } {
                panic!("PL_unify_arg_sz failed");
            }
        }

        self
    }

    /// Constructs a list from the given terms and puts the top-most cell of the
    /// list in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_cons_list%27)
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    /// #
    /// let l = engine.new_term().put_list([
    ///     engine.new_term().put_atom_chars("foo"),
    ///     engine.new_term().put_atom_chars("bar"),
    /// ]);
    ///
    /// assert_eq!(l.to_string(), "[foo,bar]");
    /// ```
    pub fn put_list<M, I>(self, members: M) -> Self
    where
        M: IntoIterator<IntoIter = I>,
        I: DoubleEndedIterator<Item = Self>,
    {
        if !unsafe { pl::PL_put_nil(self.ptr.get()) } {
            panic!("PL_put_nil failed");
        }

        for member in members.into_iter().rev() {
            if !unsafe { pl::PL_cons_list(self.ptr.get(), member.ptr.get(), self.ptr.get()) } {
                panic!("PL_cons_list failed");
            }
        }

        self
    }

    /// Records the term into the Prolog database and returns a handle to it.
    /// The returned handle may be shared across threads.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_record%27)
    pub fn record(self) -> Record {
        Record {
            ptr: unsafe { pl::PL_record(self.ptr.get()) },
        }
    }

    /// Serializes the term into a record which can be persisted and shared
    /// between Prolog sessions
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_record_external%27)
    pub fn record_external(self) -> Option<ExternalRecord> {
        let mut len: usize = 0;
        let ptr = unsafe { pl::PL_record_external(self.ptr.get(), &raw mut len) };

        std::ptr::NonNull::new(ptr).map(|ptr| ExternalRecord { ptr, len })
    }

    /// Copies a serialized term into the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_recorded_external%27)
    pub fn put_recorded_external(self, record: &[u8]) -> Self {
        if !unsafe { pl::PL_recorded_external(record.as_ptr() as _, self.ptr.get()) } {
            panic!("PL_recorded_external failed");
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
    /// #
    /// let t1 = engine.new_term().put(1);
    /// let t2 = engine.new_term().put("foo");
    /// let t3 = engine.new_term().put(&engine.atom("foo"));
    ///
    /// assert_eq!(t1.to_string(), "1");
    /// assert_eq!(t2.to_string(), "\"foo\"");
    /// assert_eq!(t3.to_string(), "foo");
    /// ```
    pub fn put(self, value: impl ToTerm) -> Self {
        value.put_in(self);
        self
    }

    /// Unifies the term with a value
    /// * https://www.swi-prolog.org/pldoc/man?section=foreign-unify
    ///
    /// Similar to constructing a new term and unifying with that term, but
    /// saves the unnecessary temporary term construction.
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    /// #
    /// let [t1, t2, t3] = engine.new_terms().into();
    ///
    /// assert!(t1.unify(1));
    /// assert!(t2.unify("foo"));
    /// assert!(t3.unify(&engine.atom("foo")));
    ///
    /// assert!(!t1.unify(2));
    ///
    /// assert_eq!(t1.to_string(), "1");
    /// assert_eq!(t2.to_string(), "\"foo\"");
    /// assert_eq!(t3.to_string(), "foo");
    /// ```
    pub fn unify(self, value: impl ToTerm) -> bool {
        value.unify_with(self)
    }

    /// Puts a term parsed from the given string in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_term_from_chars%27)
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    /// #
    /// let t1 = term! { &engine => foo(bar(foo), _) };
    /// let t2 = engine.new_term().put_parsed("foo(bar(_), foo)").unwrap();
    ///
    /// assert!(t1.unify(t2));
    /// ```
    ///
    /// On failure, returns the `ParseError` containing the exception term.
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// # let engine = session.engine();
    /// #
    /// let e = engine.new_term().put_parsed("foo(").unwrap_err();
    /// assert_eq!(
    ///     e.formal(&engine).unwrap().to_string(),
    ///     "syntax_error(end_of_clause)"
    /// );
    /// ```
    pub fn put_parsed(self, repr: &str) -> Result<Self, Exception<'a>> {
        if unsafe {
            pl::PL_put_term_from_chars(
                self.ptr.get(),
                pl::REP_UTF8 as _,
                repr.len(),
                repr.as_ptr() as _,
            )
        } {
            Ok(self)
        } else {
            Err(self.into())
        }
    }

    /// Returns the string representation of the atom stored in the term
    /// reference, or `None` if it's not an atom.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_get_atom_nchars%27)
    pub fn atom_chars(&self) -> Option<&str> {
        let mut len: usize = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if !unsafe { pl::PL_get_atom_nchars(self.ptr.get(), &raw mut len, &raw mut chars as _) } {
            return None;
        }

        Some(
            str::from_utf8(unsafe { slice::from_raw_parts(chars, len) })
                .expect("PL_get_atom_nchars returned invalid UTF-8"),
        )
    }

    /// Extracts a value from the term
    pub fn get<T: FromTerm>(self) -> Option<T> {
        T::from_term(self)
    }

    pub fn kind(&self) -> TermKind {
        TermKind::try_from(unsafe { pl::PL_term_type(self.ptr.get()) } as u32).unwrap()
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
    /// #
    /// let t = engine.new_term().put_parsed("(1, 2)").unwrap();
    /// assert_eq!(t.canonical().to_string(), "','(1,2)");
    /// ```
    pub fn canonical(self) -> Canonical<'a> {
        Canonical(self)
    }

    fn write(&self, f: &mut fmt::Formatter, flags: u32) -> fmt::Result {
        let mut len: usize = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if !unsafe {
            pl::PL_get_nchars(
                self.ptr.get(),
                &raw mut len,
                &raw mut chars as _,
                flags | pl::REP_UTF8 | pl::BUF_DISCARDABLE,
            )
        } {
            panic!("PL_get_nchars failed");
        }

        let chars = str::from_utf8(unsafe { slice::from_raw_parts(chars, len) })
            .expect("PL_get_nchars returned invalid UTF-8");

        f.write_str(chars)
    }
}

impl fmt::Display for Term<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.write(f, pl::CVT_WRITEQ)
    }
}

pub struct Canonical<'a>(Term<'a>);

impl fmt::Display for Canonical<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.write(f, pl::CVT_WRITE_CANONICAL)
    }
}

impl fmt::Debug for Term<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "<term:{} {}>",
            self.ptr.get(),
            self.canonical()
        ))
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, TryFromPrimitive)]
#[repr(u32)]
pub enum TermKind {
    Variable = pl::PL_VARIABLE,
    Atom = pl::PL_ATOM,
    Nil = pl::PL_NIL,
    Blob = pl::PL_BLOB,
    String = pl::PL_STRING,
    Integer = pl::PL_INTEGER,
    Rational = pl::PL_RATIONAL,
    Float = pl::PL_FLOAT,
    Compound = pl::PL_TERM,
    ListPair = pl::PL_LIST_PAIR,
    Dict = pl::PL_DICT,
}

/// Wrapper around a [`Term`] that represents a Prolog exception
#[derive(Clone, Copy, Debug)]
pub struct Exception<'a>(Term<'a>);

impl<'a> Exception<'a> {
    pub fn into_term(self) -> Term<'a> {
        self.0
    }

    /// The "formal" description of the error, as described in https://www.swi-prolog.org/pldoc/man?section=exceptterm.
    /// Returns `None` if the exception doesn't unify with `error(_, _)`.
    pub fn formal(self, ctx: &'a impl Context) -> Option<Term<'a>> {
        let formal = ctx.new_term();
        if term! { ctx => error({formal}, _) }.unify_with(self.0) {
            Some(formal)
        } else {
            None
        }
    }
}

impl<'a> From<Term<'a>> for Exception<'a> {
    fn from(value: Term<'a>) -> Self {
        Self(value)
    }
}

/// An array of [`Terms`] obtained via `PL_term_refs`. In particular, this means
/// that at each index `i`, the term reference `t[i]` points to `t[0] + i`.
pub struct Terms<'a, const N: usize>([Term<'a>; N]);

impl<const N: usize> Terms<'_, N> {
    pub(crate) unsafe fn new() -> Self {
        let t = unsafe { pl::PL_new_term_refs(N) };
        Self(std::array::from_fn(|i| Term::from_ptr(t + i).unwrap()))
    }

    pub(crate) fn ptr(&self) -> pl::term_t {
        self[0].ptr.get()
    }
}

impl<'a, const N: usize> Deref for Terms<'a, N> {
    type Target = [Term<'a>; N];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, const N: usize> From<Terms<'a, N>> for [Term<'a>; N] {
    fn from(value: Terms<'a, N>) -> Self {
        value.0
    }
}

/// Implemented by types that can be converted to Prolog values and put in a
/// term reference
pub trait ToTerm: Sized {
    /// Puts (copies) the value into a new term reference
    fn to_term<'a>(self, ctx: &'a (impl Context + ?Sized)) -> Term<'a>
    where
        Self: 'a,
    {
        ctx.new_term().put(self)
    }

    /// Puts (copies) the value into the term reference
    fn put_in(self, term: Term);

    /// Unifies the term with the value. The default implementation constructs
    /// a temporary term, calls [`Self::put_in`] and unifies the terms.
    fn unify_with(self, term: Term) -> bool {
        // It's OK construct a new term here, because we own a live `Term` anyway
        let temp = Term::from_ptr(unsafe { pl::PL_new_term_ref() }).unwrap();
        self.put_in(temp);
        term.unify(temp)
    }
}

impl ToTerm for Term<'_> {
    fn to_term<'a>(self, _: &'a (impl Context + ?Sized)) -> Term<'a>
    where
        Self: 'a,
    {
        self
    }

    fn put_in(self, term: Term) {
        if !unsafe { pl::PL_put_term(term.ptr.get(), self.ptr.get()) } {
            panic!("PL_put_term failed");
        }
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { pl::PL_unify(self.ptr.get(), term.ptr.get()) }
    }
}

macro_rules! impl_ToTerm {
    ($type:ty, $put_fn:path, $unify_fn:path) => {
        impl ToTerm for $type {
            fn put_in(self, term: Term) {
                assert!(unsafe { $put_fn(term.ptr.get(), self as _) });
            }

            fn unify_with(self, term: Term) -> bool {
                unsafe { $unify_fn(term.ptr.get(), self as _) }
            }
        }
    };
}

// NOTE: Using `PL_put_(u)int64` rather than `PL_put_integer` for numeric types
// because the integer type is platform-dependent
impl_ToTerm!(i8, pl::PL_put_int64, pl::PL_unify_int64);
impl_ToTerm!(i16, pl::PL_put_int64, pl::PL_unify_int64);
impl_ToTerm!(i32, pl::PL_put_int64, pl::PL_unify_int64);
impl_ToTerm!(i64, pl::PL_put_int64, pl::PL_unify_int64);
impl_ToTerm!(u8, pl::PL_put_uint64, pl::PL_unify_uint64);
impl_ToTerm!(u16, pl::PL_put_uint64, pl::PL_unify_uint64);
impl_ToTerm!(u32, pl::PL_put_uint64, pl::PL_unify_uint64);
impl_ToTerm!(u64, pl::PL_put_uint64, pl::PL_unify_uint64);
impl_ToTerm!(f32, pl::PL_put_float, pl::PL_unify_float);
impl_ToTerm!(f64, pl::PL_put_float, pl::PL_unify_float);

impl ToTerm for &Atom {
    fn put_in(self, term: Term) {
        if !unsafe { pl::PL_put_atom(term.ptr.get(), self.ptr) } {
            panic!("PL_put_atom failed");
        }
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { pl::PL_unify_atom(term.ptr.get(), self.ptr) }
    }
}

impl ToTerm for &Record {
    fn put_in(self, term: Term) {
        assert!(unsafe { pl::PL_recorded(self.ptr, term.ptr.get()) });
    }
}

impl ToTerm for bool {
    fn put_in(self, term: Term) {
        assert!(unsafe { pl::PL_put_bool(term.ptr.get(), self as _) });
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { pl::PL_unify_bool(term.ptr.get(), self as _) }
    }
}

impl ToTerm for &str {
    fn put_in(self, term: Term) {
        assert!(unsafe {
            pl::PL_put_string_nchars(term.ptr.get(), self.len(), self.as_ptr() as _)
        });
    }

    fn unify_with(self, term: Term) -> bool {
        unsafe { pl::PL_unify_string_nchars(term.ptr.get(), self.len(), self.as_ptr() as _) }
    }
}

/// Implemented by types that can be extracted from a term reference
pub trait FromTerm: Sized {
    /// Extracts the value from the term, if the term contains a value of this
    /// type. The extracted value may not be bound to the lifetime of the
    /// term.
    fn from_term(term: Term) -> Option<Self>;
}

impl FromTerm for Atom {
    fn from_term(term: Term) -> Option<Self> {
        let mut atom: pl::atom_t = 0;
        if unsafe { pl::PL_get_atom(term.ptr.get(), &raw mut atom) } {
            Some(Atom::from_ptr(atom))
        } else {
            None
        }
    }
}

impl FromTerm for bool {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: i32 = 0;
        if unsafe { pl::PL_get_bool(term.ptr.get(), &raw mut value) } {
            Some(value != 0)
        } else {
            None
        }
    }
}

impl FromTerm for i64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: i64 = 0;
        if unsafe { pl::PL_get_int64(term.ptr.get(), &raw mut value) } {
            Some(value)
        } else {
            None
        }
    }
}

impl FromTerm for u64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: u64 = 0;
        if unsafe { pl::PL_get_uint64(term.ptr.get(), &raw mut value) } {
            Some(value)
        } else {
            None
        }
    }
}

impl FromTerm for f64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: f64 = 0.0;
        if unsafe { pl::PL_get_float(term.ptr.get(), &raw mut value) } {
            Some(value)
        } else {
            None
        }
    }
}

impl FromTerm for RawFunctor {
    fn from_term(term: Term) -> Option<Self> {
        let mut ptr: pl::functor_t = 0;
        if unsafe { pl::PL_get_functor(term.ptr.get(), &raw mut ptr) } {
            Some(RawFunctor {
                _marker: Default::default(),
                ptr: NonZero::new(ptr).unwrap(),
            })
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{Context, RawFunctor, term};
    use test_log::test;

    #[test]
    fn fmt() {
        let engine = crate::test::SESSION.engine();
        let t = engine
            .new_term()
            .put_parsed("foo(bar, (1, 2), \"hello\")")
            .unwrap();
        assert_eq!(t.to_string(), "foo(bar,(1,2),\"hello\")");
    }

    #[test]
    fn get_raw_functor() {
        let engine = crate::test::SESSION.engine();
        let func = engine.functor::<2>("foo");

        let t1 = term! { &engine => foo(bar, baz) };
        let t2 = term! { &engine => 42 };

        assert_eq!(t1.get::<RawFunctor>(), Some(func.0));
        assert_eq!(t2.get::<RawFunctor>(), None);
    }
}
