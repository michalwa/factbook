use std::cell::RefCell;
use std::fmt;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use swipl_fli as pl;
pub use term::Term;

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
#[derive(Clone, Copy)]
pub struct EngineHandle {
    // Not `Send` because it's only valid in the context of the current thread engine
    _marker: PhantomData<*const ()>,
    id: c_int,
}

impl EngineHandle {
    pub fn new_term(self) -> Term {
        Term::from_ptr(unsafe { pl::PL_new_term_ref() })
    }

    pub fn atom(self, chars: &str) -> Atom {
        Atom {
            _marker: Default::default(),
            ptr: unsafe { pl::PL_new_atom_nchars(chars.len(), chars.as_ptr() as _) },
        }
    }

    pub fn functor<const ARITY: usize>(self, name: &str) -> Functor<ARITY> {
        self.atom(name).to_functor()
    }

    pub fn call(self, term: Term) -> bool {
        unsafe { pl::PL_call(term.ptr(), std::ptr::null_mut()) != 0 }
    }

    pub fn assert(self, term: Term, mode: Assert) {
        if unsafe { pl::PL_assert(term.ptr(), std::ptr::null_mut(), mode as _) } == 0 {
            panic!("PL_assert failed");
        }
    }
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

pub struct Record {
    ptr: pl::record_t,
}

unsafe impl Send for Record {}

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
/// * `#term` will include the expression `term` verbatim in the term
///   construction - useful for nesting variables within compound terms.
/// * `term` will construct an atom `term`.
/// * `f(a, b, ...)` will construct a compound term with the given functor `f`
///   and args
/// * `_` will construct an empty term (variable).
///
/// ```
/// use factbook_swipl::*;
///
/// let session = Session::init(None).unwrap();
/// let engine = session.engine();
///
/// let t1 = term! { engine => foo };
/// let t2 = term! { engine => foo(bar, {t1}) };
///
/// assert_eq!(t1.to_string(), "foo");
/// assert_eq!(t2.to_string(), "foo(bar,foo)");
/// ```
#[macro_export]
macro_rules! term {
    ($engine:expr => {$term:expr}) => {
        $term
    };
    ($engine:expr => $atom:ident) => {
        $engine.new_term().put_atom_chars(stringify!($atom))
    };
    ($engine:expr => _) => {
        $engine.new_term()
    };
    ($engine:expr => $functor:ident ( $($args:tt)+ )) => {{
        let engine = $engine;
        engine.new_term().put_functor(
            engine.functor(stringify!($functor)),
            term!(@args () engine => $($args)+),
        )
    }};
    // Recursively builds nested terms. The base case without arguments wraps
    // the resulting expressions in an array, because macro invocations cannot
    // yield bare comma-separated expressions. The `$($out:tt)*` group is used
    // as an accumulator for the constructed expressions.
    //
    // term!(@args ()           engine => arg0, arg1, arg2)
    // term!(@args (out0)       engine => arg1, arg2)
    // term!(@args (out0, out1) engine => arg2)
    // term!(@args (out0, out1, out2))
    //
    (@args ($($out:tt)*)) => {
        [$($out)*]
    };
    (@args ($($($out:tt)+)?) $engine:expr => $term:tt $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($engine => $term))
            $($engine => $($rest)+)?)
    };
    (@args ($($($out:tt)+)?) $engine:expr => $functor:ident ( $($args:tt)+ ) $(, $($rest:tt)+)?) => {
        term!(@args
            ($($($out)* ,)? term!($engine => $functor ( $($args)+ )))
            $($engine => $($rest)+)?)
    };
}

#[cfg(test)]
mod test {
    use super::*;
    use std::sync::LazyLock;
    use std::thread;

    // Normally you would store the session somewhere in the state of your program.
    // The reason why the library doesn't provide a `LazyLock` out of the box
    // is to allow initializing the session with a non-static state memory area.
    pub(crate) static SESSION: LazyLock<Session<'static>> =
        LazyLock::new(|| Session::init(None).unwrap());

    #[test]
    fn threads() {
        thread::spawn(|| {
            let engine = dbg!(SESSION.engine());
            let t = term! { engine => foo(bar(baz), qux) };

            engine.assert(t, Default::default());
        });

        let engine = dbg!(SESSION.engine());
        let t = engine.new_term();
        let q = term! { engine => foo(bar({t}), _) };

        assert!(engine.call(q));
        assert_eq!(t.atom_chars(), "baz");
    }

    #[test]
    fn record() {
        let engine = dbg!(SESSION.engine());
        let t = term! { engine => foo(bar) };
        let record = t.record();

        thread::spawn(move || {
            let engine = dbg!(SESSION.engine());
            let t = engine.new_term().put_recorded(&record);

            assert_eq!(t.to_string(), "foo(bar)");
        });
    }
}
