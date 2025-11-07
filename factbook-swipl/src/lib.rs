use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{fmt, slice};
use swipl_fli::{self as pl, PL_ASSERTA, PL_ASSERTZ};

/// Global session handle which, when held, statically guarantees that the
/// Prolog runtime has been initialized. Parameterized by the lifetime of the
/// state passed to `get_or_init`
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
    _marker: PhantomData<*const ()>,
    id: c_int,
}

impl Drop for Engine {
    fn drop(&mut self) {
        if unsafe { pl::PL_thread_destroy_engine() } == 0 {
            eprintln!("warning: PL_thread_destroy_engine failed");
        }
    }
}

/// A handle/guard which, when held, statically guarantees that the current
/// thread has an attached engine
#[derive(Clone, Copy)]
pub struct EngineHandle {
    _marker: PhantomData<*const ()>,
    id: c_int,
}

impl EngineHandle {
    pub fn new_term(self) -> Term {
        Term {
            _marker: Default::default(),
            ptr: unsafe { pl::PL_new_term_ref() },
        }
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
        unsafe { pl::PL_call(term.ptr, std::ptr::null_mut()) != 0 }
    }

    pub fn assert(self, term: Term, mode: Assert) {
        if unsafe { pl::PL_assert(term.ptr, std::ptr::null_mut(), mode as _) } == 0 {
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

#[derive(Clone, Copy)]
pub struct Term {
    _marker: PhantomData<*const ()>,
    ptr: pl::term_t,
}

impl Term {
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
}

pub struct Atom {
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
    _marker: PhantomData<*const ()>,
    ptr: pl::functor_t,
}

#[derive(Default, Clone, Copy)]
#[repr(u32)]
pub enum Assert {
    #[default]
    Last = PL_ASSERTZ,
    First = PL_ASSERTA,
}

#[macro_export]
macro_rules! term {
    ($engine:expr => {$term:expr}) => {
        $term
    };
    ($engine:expr => $atom:ident) => {
        $engine.new_term().put_atom_chars(stringify!($atom))
    };
    ($engine:expr => $functor:ident($($arg:tt $($args:tt)?),+)) => {
        $engine.new_term().put_functor(
            $engine.functor(stringify!($functor)),
            [$(term!($engine => $arg $($args)?)),+]
        )
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
    static SESSION: LazyLock<Session<'static>> = LazyLock::new(|| Session::init(None).unwrap());

    #[test]
    fn threads() {
        thread::spawn(|| {
            let engine = dbg!(SESSION.engine());
            let t = term! { engine => foo(bar(baz)) };

            engine.assert(t, Default::default());
        });

        let engine = dbg!(SESSION.engine());
        let t = engine.new_term();
        let q = term! { engine => foo(bar({t})) };

        assert!(engine.call(q));
        assert_eq!(t.atom_chars(), "baz");
    }
}
