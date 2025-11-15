use std::cell::RefCell;
use std::fmt::{self, Write};
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use swipl_fli as pl;
pub use term::{Term, ToTerm};

mod term;

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

        if let Some(state) = state.into() {
            if unsafe { pl::PL_set_resource_db_mem(state.as_ptr(), state.len()) } == 0 {
                panic!("failed to initialize SWI-Prolog: PL_set_resource_db_mem failed");
            }
        }

        let mut args = [
            b"factbook\0".as_ptr() as *mut _,
            b"--quiet\0".as_ptr() as *mut _,
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
            id => Engine {
                _marker: Default::default(),
                id,
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
    id: c_int,
}

impl Drop for Engine {
    fn drop(&mut self) {
        // Don't attempt to call `PL_thread_destroy_engine` if `PL_cleanup` was already
        // called, otherwise it will hang. This can be the case if `Session` is dropped
        // before the `Engine`, e.g. when the thread holding the `Session` exits.
        if unsafe { pl::PL_is_initialised(std::ptr::null_mut(), std::ptr::null_mut()) } != 0 {
            if unsafe { pl::PL_thread_destroy_engine() } == 0 {
                eprintln!("warning: PL_thread_destroy_engine failed");
            }
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
    id: c_int,
}

// SAFETY: If an `Engine` exists then an engine has been attached on the current
// thread (`Engine` is not `Send`) and a handle can be created. It's enough for
// `EngineHandle` not to be `Send` to ensure that is is always valid (for the
// duration of the thread).
impl From<&Engine> for EngineHandle {
    fn from(value: &Engine) -> Self {
        Self {
            _marker: Default::default(),
            id: value.id,
        }
    }
}

impl fmt::Debug for EngineHandle {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.id.fmt(f)
    }
}

/// A foreign frame - a contained environment for operating on the Prolog
/// stack
pub struct Frame<'p> {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
    _parent: PhantomData<&'p ()>,
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
            _parent: Default::default(),
            ptr: unsafe { pl::PL_open_foreign_frame() },
        }
    }

    fn new_term<'a>(&'a self) -> Term<'a> {
        Term::from_ptr(unsafe { pl::PL_new_term_ref() })
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

    fn call(&self, term: Term) -> bool {
        unsafe { pl::PL_call(term.ptr, std::ptr::null_mut()) != 0 }
    }

    fn assert(&self, term: Term, mode: Assert) {
        if unsafe { pl::PL_assert(term.ptr, std::ptr::null_mut(), mode as _) } == 0 {
            panic!("PL_assert failed");
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
        unsafe { pl::PL_register_atom(self.ptr) };
        Self { ..*self }
    }
}

impl Drop for Atom {
    fn drop(&mut self) {
        unsafe { pl::PL_unregister_atom(self.ptr) };
    }
}

impl Atom {
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
///   and args.
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
/// let t4 = term! { &engine => [{t1}, "bar", [3, 4]] };
///
/// assert_eq!(t1.to_string(), "foo");
/// assert_eq!(t2.to_string(), "foo(bar(foo),foo)");
/// assert_eq!(t4.to_string(), "[foo,\"bar\",[3,4]]");
/// ```
#[macro_export]
macro_rules! term {
    ($ctx:expr => {$term:expr}) => {
        $crate::ToTerm::to_term($term, $ctx)
    };
    ($ctx:expr => $value:literal) => {
        $crate::ToTerm::to_term($value, $ctx)
    };
    ($ctx:expr => $atom:ident) => {
        $ctx.new_term().put_atom_chars(stringify!($atom))
    };
    ($ctx:expr => _) => {
        $ctx.new_term()
    };
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
    (@args ($($($out:tt)+)?) $ctx:expr => [ $($args:tt)+ ] $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($ctx => [ $($args)+ ]))
            $($ctx => $($rest)+)?)
    };
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
            let engine = dbg!(SESSION.engine());
            let t = term! { &engine => foo(bar(baz), qux) };

            engine.assert(t, Default::default());
        });

        let engine = dbg!(SESSION.engine());
        let t = engine.new_term();
        let q = term! { &engine => foo(bar({t}), _) };

        assert!(engine.call(q));
        assert_eq!(t.atom_chars(), Some("baz"));
    }

    #[test]
    fn record() {
        let engine = dbg!(SESSION.engine());
        let t = term! { &engine => foo(bar) };
        let record = t.record();

        thread::spawn(move || {
            let engine = dbg!(SESSION.engine());
            let t = engine.new_term().put(&record);

            assert_eq!(t.to_string(), "foo(bar)");
        });
    }

    #[test]
    fn external_record() {
        let engine = dbg!(SESSION.engine());
        let t1 = term! { &engine => foo(bar(42), ["hello", "world"]) };

        let bytes = {
            let record = t1.record_external().unwrap();
            record.as_ref().to_owned()
        };

        let t2 = engine.new_term().put_recorded_external(&bytes);
        assert!(t1.unify_with(t2));
    }

    #[test]
    fn frames() {
        let mut engine = dbg!(SESSION.engine());
        {
            let frame = engine.frame();
            frame.assert(frame.new_term().put_atom_chars("foo"), Default::default());
        }
        assert!(engine.call(engine.new_term().put_atom_chars("foo")));
    }
}
