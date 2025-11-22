use crate::foreign::{Predicate, PredicateArgs};
use crate::term::{Exception, Term};
use std::cell::RefCell;
use std::fmt::{self, Write};
use std::marker::PhantomData;
use std::sync::atomic::{AtomicBool, Ordering};
use swipl_fli as pl;

pub mod blob;
pub mod foreign;
pub mod term;

/// Global session handle which, when held, statically guarantees that the
/// Prolog runtime has been initialized. Parameterized by the lifetime of the
/// state passed to `init`.
pub struct Session<'s> {
    _state: PhantomData<&'s [u8]>,
}

impl<'s> Session<'s> {
    /// Initializes the global Prolog runtime and returns a handle to it. Only
    /// returns the handle once and returns `None` on subsequent calls.
    pub fn init(state: impl Into<Option<&'s [u8]>>) -> Option<Self> {
        // We can't allow calling this method multiple times, because
        // `PL_set_resource_db_mem` is said to only support being called once
        // https://www.swi-prolog.org/pldoc/doc_for?object=c(%27PL_set_resource_db_mem%27)
        static INIT: AtomicBool = AtomicBool::new(false);
        if INIT.swap(true, Ordering::AcqRel) {
            return None;
        }

        #[cfg(debug_assertions)]
        if unsafe { pl::PL_is_initialised(std::ptr::null_mut(), std::ptr::null_mut()) } != 0 {
            panic!("failed to initialize SWI-Prolog: already initialized");
        }

        if let Some(state) = state.into()
            && unsafe { pl::PL_set_resource_db_mem(state.as_ptr(), state.len()) } == 0
        {
            panic!("failed to initialize SWI-Prolog: PL_set_resource_db_mem failed");
        }

        let mut args = [
            c"factbook".as_ptr() as *mut _,
            c"--quiet".as_ptr() as *mut _,
            std::ptr::null_mut(),
        ];

        if unsafe { pl::PL_initialise((args.len() - 1) as _, args.as_mut_ptr()) } == 0 {
            panic!("failed to initialize SWI-Prolog: PL_initialise failed");
        }

        Some(Self {
            _state: Default::default(),
        })
    }

    /// Optionally attaches and returns an engine for the current thread.
    /// Engines are managed according to the one-to-one strategy documented
    /// here: https://www.swi-prolog.org/pldoc/man?section=threadoneone
    /// with the additional constraint that `PL_thread_attach_engine` is only
    /// called once per thread on the first call to this method.
    pub fn engine(&self) -> EngineHandle {
        thread_local! {
            static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
        }
        ENGINE.with_borrow_mut(|e| (&*e.get_or_insert_with(|| self.attach_engine())).into())
    }

    fn attach_engine(&self) -> Engine {
        match unsafe {
            pl::PL_thread_attach_engine(&mut pl::PL_thread_attr_t {
                stack_limit: 0,
                table_space: 0,
                alias: std::ptr::null_mut(),
                cancel: None,
                flags: 0,
                max_queue_size: 0,
                reserved: [std::ptr::null_mut(); 3],
            } as _)
        } {
            -1 => panic!("failed to create Prolog engine: PL_thread_attach_engine failed"),
            -2 => panic!(
                "failed to create Prolog engine: SWI-Prolog version does not support threads"
            ),
            _ => Engine {
                _marker: Default::default(),
            },
        }
    }
}

impl Drop for Session<'_> {
    fn drop(&mut self) {
        if unsafe { pl::PL_cleanup(pl::PL_CLEANUP_NO_CANCEL as _) } != pl::PL_CLEANUP_SUCCESS as _ {
            eprintln!("warning: PL_cleanup failed");
        }
    }
}

/// Internal engine handle which destroys the current engine on drop. We can't
/// give out references to this, since we're storing it as a thread-local, so
/// `EngineHandle` is used instead.
struct Engine {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Don't attempt to call `PL_thread_destroy_engine` if `PL_cleanup` was already
        // called, otherwise it will hang. This can be the case if `Session` is dropped
        // before the `Engine`, e.g. when the thread holding the `Session` exits.
        if unsafe { pl::PL_is_initialised(std::ptr::null_mut(), std::ptr::null_mut()) } != 0
            && unsafe { pl::PL_thread_destroy_engine() } == 0
        {
            eprintln!("warning: PL_thread_destroy_engine failed");
        }
    }
}

/// A handle/guard which, when held, statically guarantees that the current
/// thread has an attached engine. It has a static lifetime, but is not `Send`,
/// because engines are managed as thread-locals and are destroyed at the end of
/// the thread.
pub struct EngineHandle {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
}

impl EngineHandle {
    pub(crate) unsafe fn assume_attached() -> Self {
        Self {
            _marker: Default::default(),
        }
    }
}

impl From<&Engine> for EngineHandle {
    fn from(_: &Engine) -> Self {
        // SAFETY: If an `Engine` exists then an engine has been attached on the current
        // thread (`Engine` is not `Send`) and a handle can be created. It's enough for
        // `EngineHandle` not to be `Send` to ensure that is is always valid (for the
        // duration of the thread).
        unsafe { Self::assume_attached() }
    }
}

/// A foreign frame - a contained environment for operating on the Prolog
/// stack
pub struct Frame<'p> {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<(*const (), &'p ())>,
    ptr: pl::PL_fid_t,
}

impl Drop for Frame<'_> {
    fn drop(&mut self) {
        unsafe { pl::PL_close_foreign_frame(self.ptr) };
    }
}

/// Shared trait for types which allow constructing new terms
pub trait Context {
    /// Opens a new foreign frame.
    ///
    /// Terms created within the frame are bound to its lifetime. The following
    /// will fail to compile:
    ///
    /// ```compile_fail
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// let mut engine = session.engine();
    /// let frame = engine.frame();
    /// let t = term! { &frame => foo(bar) };
    /// std::mem::drop(frame);
    /// println!("{t}");
    /// ```
    fn frame<'a>(&'a mut self) -> Frame<'a> {
        Frame {
            _marker: Default::default(),
            ptr: unsafe { pl::PL_open_foreign_frame() },
        }
    }

    fn new_term<'a>(&'a self) -> Term<'a> {
        Term::from_ptr(unsafe { pl::PL_new_term_ref() }).unwrap()
    }

    fn new_terms<'a, const N: usize>(&'a self) -> [Term<'a>; N] {
        let t = unsafe { pl::PL_new_term_refs(N) };
        std::array::from_fn(|i| Term::from_ptr(t + i).unwrap())
    }

    fn atom(&self, chars: &str) -> Atom {
        Atom {
            _marker: Default::default(),
            ptr: unsafe { pl::PL_new_atom_nchars(chars.len(), chars.as_ptr() as _) },
        }
    }

    fn functor<const ARITY: usize>(&self, name: &str) -> Functor<ARITY> {
        self.atom(name).to_functor()
    }

    /// Calls the given goal and returns one of three possible results;
    /// * `Ok(true)` if the goal succeeds,
    /// * `Ok(false)` if the goal fails without exception,
    /// * `Err(_)` if the goal raises an exception.
    ///
    /// Takes an optional module name to run the goal in.
    ///
    /// ```
    /// # use factbook_swipl::*;
    /// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
    /// # let session = Session::init(STATE).unwrap();
    /// let engine = session.engine();
    /// assert!(engine.call(term! { &engine => true }, None).unwrap());
    /// assert!(!engine.call(term! { &engine => false }, None).unwrap());
    /// assert!(engine.call(term! { &engine => nonexistent }, None).is_err());
    /// ```
    fn call<'a, 'm>(
        &'a self,
        term: Term,
        module: impl Into<Option<&'m str>>,
    ) -> Result<bool, Exception<'a>> {
        let module = match module.into() {
            Some(module) => {
                let module_atom = self.atom(module);
                unsafe { pl::PL_new_module(module_atom.ptr) }
            },
            None => std::ptr::null_mut(),
        };

        if unsafe { pl::PL_call(term.ptr.get(), module) } != 0 {
            Ok(true)
        } else {
            // https://www.swi-prolog.org/pldoc/man?section=foreign-exceptions
            match Term::from_ptr(unsafe { pl::PL_exception(std::ptr::null_mut()) }) {
                Some(exception) => {
                    // Make a copy of the exception term before calling `PL_clear_exception`
                    let exception = self.new_term().put(exception);
                    unsafe { pl::PL_clear_exception() };
                    Err(Exception::from(exception))
                },
                None => Ok(false),
            }
        }
    }

    fn assert(&self, term: Term, mode: Assert) {
        if unsafe { pl::PL_assert(term.ptr.get(), std::ptr::null_mut(), mode as _) } == 0 {
            panic!("PL_assert failed");
        }
    }

    /// (Re)defines the specified module using the given Prolog source code
    fn load_module_from_str<'a>(&'a self, module: &str, source: &str) -> Result<(), Exception<'a>> {
        // https://www.swi-prolog.org/pldoc/man?section=defmodule
        let scoped_source = format!(":- module({module}, []). {source}");
        let s = self.new_term();

        let load_goal = term! {
            self => ","(
                // https://www.swi-prolog.org/pldoc/man?predicate=open_string/2
                open_string({scoped_source.as_str()}, {s}),
                // https://www.swi-prolog.org/pldoc/man?predicate=load_files/2
                load_files(user, [stream({s}), redefine_module(true)])
            )
        };

        if !self.call(load_goal, None)? {
            panic!("could not load module from source");
        }

        Ok(())
    }

    fn predicate_defined<'m, const ARITY: usize>(
        &self,
        name: &str,
        module: impl Into<Option<&'m str>>,
    ) -> bool {
        let head = self.new_term();
        if unsafe { pl::PL_put_functor(head.ptr.get(), self.functor::<ARITY>(name).ptr) } == 0 {
            panic!("PL_put_functor failed");
        }

        let module = module.into().map(|m| self.atom(m));
        let qualified_head = match &module {
            Some(module) => {
                term! { self => ":"({module}, { head }) }
            },
            None => head,
        };

        // https://www.swi-prolog.org/pldoc/man?predicate=predicate_property/2
        self.call(
            term! { self => predicate_property({qualified_head}, defined) },
            None,
        )
        .unwrap()
    }

    fn register_predicate<P: Predicate>(&self) {
        if unsafe {
            pl::PL_register_foreign(
                P::NAME.as_ptr(),
                P::Args::ARITY as _,
                std::mem::transmute::<*const (), Option<unsafe extern "C" fn() -> _>>(P::EXTERN_FN),
                P::FLAGS as _,
            )
        } == 0
        {
            panic!("PL_register_foreign failed");
        }
    }
}

impl Context for EngineHandle {}
impl Context for Frame<'_> {}

pub struct Atom {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
    ptr: pl::atom_t,
}

impl Clone for Atom {
    fn clone(&self) -> Self {
        Self::from_ptr(self.ptr)
    }
}

impl Drop for Atom {
    fn drop(&mut self) {
        unsafe { pl::PL_unregister_atom(self.ptr) };
    }
}

impl Atom {
    pub(crate) fn from_ptr(ptr: pl::atom_t) -> Self {
        unsafe { pl::PL_register_atom(ptr) };
        Self {
            _marker: Default::default(),
            ptr,
        }
    }

    pub fn to_functor<const ARITY: usize>(&self) -> Functor<ARITY> {
        Functor {
            _marker: Default::default(),
            ptr: unsafe { pl::PL_new_functor_sz(self.ptr, ARITY) },
        }
    }
}

#[derive(Clone, Copy)]
pub struct Functor<const ARITY: usize> {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
    ptr: pl::functor_t,
}

/// A handle to a recorded term which can be shared between threads
pub struct Record {
    ptr: pl::record_t,
}

unsafe impl Send for Record {}
unsafe impl Sync for Record {}

impl Clone for Record {
    fn clone(&self) -> Self {
        Self {
            ptr: unsafe { pl::PL_duplicate_record(self.ptr) },
        }
    }
}

impl Drop for Record {
    fn drop(&mut self) {
        unsafe { pl::PL_erase(self.ptr) };
    }
}

/// A serialized term which can be persisted and shared between Prolog sessions
pub struct ExternalRecord {
    ptr: std::ptr::NonNull<i8>,
    len: usize,
}

unsafe impl Send for ExternalRecord {}

impl Drop for ExternalRecord {
    fn drop(&mut self) {
        if unsafe { pl::PL_erase_external(self.ptr.as_ptr()) } == 0 {
            panic!("PL_erase_external failed");
        }
    }
}

impl AsRef<[u8]> for ExternalRecord {
    fn as_ref(&self) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr() as _, self.len) }
    }
}

impl fmt::Debug for ExternalRecord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("b\"")?;
        for byte in self.as_ref() {
            f.write_fmt(format_args!("\\x{byte:02X}"))?;
        }
        f.write_char('"')
    }
}

#[derive(Default, Clone, Copy)]
#[repr(u32)]
pub enum Assert {
    #[default]
    Last = pl::PL_ASSERTZ,
    First = pl::PL_ASSERTA,
}

/// Constructs a Prolog term using Prolog-like syntax
///
/// Supported terms:
/// * `{term}` will include the expression `term` verbatim in the term
///   construction - useful for nesting variables within compound terms. It
///   supports `Term` as well as other types implementing `ToTerm`.
/// * `term` will construct an atom `term`.
/// * `_` will construct an empty term (variable).
/// * `f(a, b, ...)` will construct a compound term with the given functor `f`
///   and args. Functors which are not valid Rust idents, e.g. `,` can be
///   wrapped in a string literal: `","(a, b)`.
/// * `[a, b, ...]` will construct a list with the given members.
/// * literals like `42` or `"hello"` will also be converted to Prolog values
///   using `ToTerm`.
///
/// Not supported:
/// * Variables based on casing, i.e. `X` will still produce an atom.
///
/// ```
/// # use factbook_swipl::*;
/// # const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));
/// # let session = Session::init(STATE).unwrap();
/// # let engine = session.engine();
///
/// let t1 = term! { &engine => foo };
/// let t2 = term! { &engine => foo(bar({t1}), {t1}) };
/// let t3 = term! { &engine => [{t1}, "bar", [3, 4]] };
/// let t4 = term! { &engine => ","(a, ","(b, c)) };
///
/// assert_eq!(t1.to_string(), "foo");
/// assert_eq!(t2.to_string(), "foo(bar(foo),foo)");
/// assert_eq!(t3.to_string(), "[foo,\"bar\",[3,4]]");
/// assert_eq!(t4.to_string(), "a,b,c");
/// ```
#[macro_export]
macro_rules! term {
    ($ctx:expr => {$term:expr}) => {
        $crate::term::ToTerm::to_term($term, $ctx)
    };
    ($ctx:expr => $value:literal) => {
        $crate::term::ToTerm::to_term($value, $ctx)
    };
    ($ctx:expr => $atom:ident) => {
        $ctx.new_term().put_atom_chars(stringify!($atom))
    };
    ($ctx:expr => _) => {
        $ctx.new_term()
    };
    ($ctx:expr => $functor:literal ( $($args:tt)+ )) => {{
        let ctx = $ctx;
        ctx.new_term().put_functor(
            ctx.functor($functor),
            term!(@args () ctx => $($args)+),
        )
    }};
    ($ctx:expr => $functor:ident ( $($args:tt)+ )) => {{
        let ctx = $ctx;
        ctx.new_term().put_functor(
            ctx.functor(stringify!($functor)),
            term!(@args () ctx => $($args)+),
        )
    }};
    ($ctx:expr => [ $($args:tt)+ ]) => {{
        let ctx = $ctx;
        ctx.new_term().put_list(term!(@args () ctx => $($args)+))
    }};
    // Recursively builds nested terms. The base case without arguments wraps
    // the resulting expressions in an array, because macro invocations cannot
    // yield bare comma-separated expressions. The `$($out:tt)*` group is used
    // as an accumulator for the constructed expressions.
    //
    // term!(@args ()           ctx => arg0, arg1, arg2)
    // term!(@args (out0)       ctx => arg1, arg2)
    // term!(@args (out0, out1) ctx => arg2)
    // term!(@args (out0, out1, out2))
    //
    (@args ($($out:tt)*)) => {
        [$($out)*]
    };
    (@args ($($($out:tt)+)?) $ctx:expr => $term:tt $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($ctx => $term))
            $($ctx => $($rest)+)?)
    };
    (@args ($($($out:tt)+)?) $ctx:expr => $functor:ident ( $($args:tt)+ ) $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($ctx => $functor ( $($args)+ )))
            $($ctx => $($rest)+)?)
    };
    (@args ($($($out:tt)+)?) $ctx:expr => $functor:literal ( $($args:tt)+ ) $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($ctx => $functor ( $($args)+ )))
            $($ctx => $($rest)+)?)
    };
}

#[macro_export]
macro_rules! assert_unify {
    ($a:expr, $b:expr) => {{
        let a = $a;
        let b = $b;
        assert!(
            a.unify_with(b),
            "assertion `left.unify_with(right)` failed\n  left: {a},\n right: {b}"
        );
    }};
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::LazyLock;
    use std::thread;

    const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

    // Normally you would store the session somewhere in the state of your program.
    // The reason why the library doesn't provide a `LazyLock` out of the box
    // is to allow initializing the session with a non-static state memory area.
    pub(crate) static SESSION: LazyLock<Session<'static>> =
        LazyLock::new(|| Session::init(STATE).unwrap());

    #[test]
    fn threads() {
        thread::spawn(|| {
            let engine = SESSION.engine();
            let t = term! { &engine => foo(bar(baz), qux) };

            engine.assert(t, Default::default());
        });

        let engine = SESSION.engine();
        let t = engine.new_term();
        let q = term! { &engine => foo(bar({t}), _) };

        assert!(engine.call(q, None).unwrap());
        assert_eq!(t.atom_chars(), Some("baz"));
    }

    #[test]
    fn record() {
        let engine = SESSION.engine();
        let t = term! { &engine => foo(bar) };
        let record = t.record();

        thread::spawn(move || {
            let engine = SESSION.engine();
            let t = engine.new_term().put(&record);

            assert_eq!(t.to_string(), "foo(bar)");
        });
    }

    #[test]
    fn external_record() {
        let engine = SESSION.engine();
        let t1 = term! { &engine => foo(bar(42), ["hello", "world"]) };

        let bytes = {
            let record = t1.record_external().unwrap();
            record.as_ref().to_owned()
        };

        let t2 = engine.new_term().put_recorded_external(&bytes);
        assert_unify!(t1, t2);
    }

    #[test]
    fn frames() {
        let mut engine = SESSION.engine();
        {
            let frame = engine.frame();
            frame.assert(frame.new_term().put_atom_chars("foo"), Default::default());
        }
        assert!(
            engine
                .call(engine.new_term().put_atom_chars("foo"), None)
                .unwrap()
        );
    }
}
