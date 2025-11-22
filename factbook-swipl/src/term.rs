use crate::{Atom, Context, ExternalRecord, Functor, Record, term};
use std::marker::PhantomData;
use std::num::NonZero;
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
        if unsafe { pl::PL_put_variable(self.ptr.get()) } == 0 {
            panic!("PL_put_variable failed");
        }

        self
    }

    /// Puts an atom with the given name in the term reference
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_put_atom_chars%27)
    pub fn put_atom_chars(self, chars: &str) -> Self {
        if unsafe { pl::PL_put_atom_nchars(self.ptr.get(), chars.len(), chars.as_ptr() as _) } == 0
        {
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
        if unsafe { pl::PL_put_functor(self.ptr.get(), functor.ptr) } == 0 {
            panic!("PL_put_functor failed");
        }

        for (i, arg) in args.into_iter().enumerate() {
            if unsafe { pl::PL_unify_arg_sz(i + 1, self.ptr.get(), arg.ptr.get()) } == 0 {
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
        if unsafe { pl::PL_put_nil(self.ptr.get()) } == 0 {
            panic!("PL_put_nil failed");
        }

        for member in members.into_iter().rev() {
            if unsafe { pl::PL_cons_list(self.ptr.get(), member.ptr.get(), self.ptr.get()) } == 0 {
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
        if unsafe { pl::PL_recorded_external(record.as_ptr() as _, self.ptr.get()) } == 0 {
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
    /// assert!(t1.unify_with(t2));
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
        match unsafe {
            pl::PL_put_term_from_chars(
                self.ptr.get(),
                pl::REP_UTF8 as _,
                repr.len(),
                repr.as_ptr() as _,
            )
        } {
            0 => Err(self.into()),
            _ => Ok(self),
        }
    }

    /// Returns the string representation of the atom stored in the term
    /// reference, or `None` if it's not an atom.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_get_atom_nchars%27)
    pub fn atom_chars(&self) -> Option<&str> {
        let mut len: usize = 0;
        let mut chars: *mut u8 = std::ptr::null_mut();

        if unsafe { pl::PL_get_atom_nchars(self.ptr.get(), &raw mut len, &raw mut chars as _) } == 0
        {
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

    /// Unifies the two terms and returns `true` on success or `false` on
    /// failure.
    /// * https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_unify%27)
    pub fn unify_with(self, other: Term) -> bool {
        unsafe { pl::PL_unify(self.ptr.get(), other.ptr.get()) != 0 }
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

        if unsafe {
            pl::PL_get_nchars(
                self.ptr.get(),
                &raw mut len,
                &raw mut chars as _,
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
            "<term:{:p} {}>",
            self.ptr.get() as *const (),
            self.canonical()
        ))
    }
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
}

impl ToTerm for Term<'_> {
    fn to_term<'a>(self, _: &'a (impl Context + ?Sized)) -> Term<'a>
    where
        Self: 'a,
    {
        self
    }

    fn put_in(self, term: Term) {
        if unsafe { pl::PL_put_term(term.ptr.get(), self.ptr.get()) } == 0 {
            panic!("PL_put_term failed");
        }
    }
}

impl ToTerm for &Atom {
    fn put_in(self, term: Term) {
        if unsafe { pl::PL_put_atom(term.ptr.get(), self.ptr) } == 0 {
            panic!("PL_put_atom failed");
        }
    }
}

impl ToTerm for &Record {
    fn put_in(self, term: Term) {
        if unsafe { pl::PL_recorded(self.ptr, term.ptr.get()) } == 0 {
            panic!("PL_recorded failed");
        }
    }
}

macro_rules! impl_ToTerm {
    (|$value:ident: $type:ty, $term:ident| pl::$fn:ident($($args:tt)*)) => {
        impl ToTerm for $type {
            fn put_in(self, $term: Term) {
                let $value = self;
                if unsafe { pl::$fn($($args)*) } == 0 {
                    panic!(concat!(stringify!($fn), " failed"));
                }
            }
        }
    };
}

impl_ToTerm!(|v: bool, t| pl::PL_put_bool(t.ptr.get(), v as _));
impl_ToTerm!(|v: &str, t| pl::PL_put_string_nchars(t.ptr.get(), v.len(), v.as_ptr() as _));
// Not using `PL_put_integer` because the integer type is platform-dependent
impl_ToTerm!(|v: i8, t| pl::PL_put_int64(t.ptr.get(), v as _));
impl_ToTerm!(|v: i16, t| pl::PL_put_int64(t.ptr.get(), v as _));
impl_ToTerm!(|v: i32, t| pl::PL_put_int64(t.ptr.get(), v as _));
impl_ToTerm!(|v: i64, t| pl::PL_put_int64(t.ptr.get(), v));
impl_ToTerm!(|v: u8, t| pl::PL_put_uint64(t.ptr.get(), v as _));
impl_ToTerm!(|v: u16, t| pl::PL_put_uint64(t.ptr.get(), v as _));
impl_ToTerm!(|v: u32, t| pl::PL_put_uint64(t.ptr.get(), v as _));
impl_ToTerm!(|v: u64, t| pl::PL_put_uint64(t.ptr.get(), v));
impl_ToTerm!(|v: f32, t| pl::PL_put_float(t.ptr.get(), v as _));
impl_ToTerm!(|v: f64, t| pl::PL_put_float(t.ptr.get(), v));

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
        if unsafe { pl::PL_get_atom(term.ptr.get(), &raw mut atom) } != 0 {
            Some(Atom::from_ptr(atom))
        } else {
            None
        }
    }
}

impl FromTerm for bool {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: i32 = 0;
        if unsafe { pl::PL_get_bool(term.ptr.get(), &raw mut value) } != 0 {
            Some(value != 0)
        } else {
            None
        }
    }
}

impl FromTerm for i64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: i64 = 0;
        if unsafe { pl::PL_get_int64(term.ptr.get(), &raw mut value) } != 0 {
            Some(value)
        } else {
            None
        }
    }
}

impl FromTerm for u64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: u64 = 0;
        if unsafe { pl::PL_get_uint64(term.ptr.get(), &raw mut value) } != 0 {
            Some(value)
        } else {
            None
        }
    }
}

impl FromTerm for f64 {
    fn from_term(term: Term) -> Option<Self> {
        let mut value: f64 = 0.0;
        if unsafe { pl::PL_get_float(term.ptr.get(), &raw mut value) } != 0 {
            Some(value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use crate::Context;

    #[test]
    fn fmt() {
        let engine = crate::test::SESSION.engine();
        let t = engine
            .new_term()
            .put_parsed("foo(bar, (1, 2), \"hello\")")
            .unwrap();
        assert_eq!(t.to_string(), "foo(bar,(1,2),\"hello\")");
    }
}
