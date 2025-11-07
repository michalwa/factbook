use std::cell::RefCell;
use std::marker::PhantomData;
use std::os::raw::c_int;
use std::sync::OnceLock;
use std::{fmt, slice};
use swipl_fli as pl;

static SESSION: OnceLock<Session> = OnceLock::new();

thread_local! {
    static ENGINE: RefCell<Option<Engine>> = const { RefCell::new(None) };
}

/// Global session handle which, when held, statically guarantees that the
/// Prolog runtime has been initialized
pub struct Session(());

impl Session {
    pub fn get_or_init<'s>(state: impl Into<Option<&'s [u8]>>) -> &'static Self {
        SESSION.get_or_init(|| {
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

            Self(())
        })
    }

    pub fn engine(&self) -> EngineHandle {
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
            -1 => panic!("failed to create Prolog engine"),
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

impl Drop for Session {
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
pub struct EngineHandle {
    _marker: PhantomData<*const ()>,
    id: c_int,
}

impl EngineHandle {
    pub fn new_term<'e>(&'e self) -> Term<'e> {
        Term {
            _engine: Default::default(),
            ptr: unsafe { pl::PL_new_term_ref() },
        }
    }

    pub fn call(&self, term: &Term) -> bool {
        unsafe { pl::PL_call(term.ptr, std::ptr::null_mut()) != 0 }
    }

    pub fn assert(&self, term: &Term) {
        if unsafe { pl::PL_assert(term.ptr, std::ptr::null_mut(), pl::PL_ASSERTZ as _) } == 0 {
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
pub struct Term<'e> {
    _engine: PhantomData<&'e EngineHandle>,
    ptr: pl::term_t,
}

impl Term<'_> {
    pub fn put_atom_chars(&self, chars: &str) -> Self {
        if unsafe { pl::PL_put_atom_nchars(self.ptr, chars.len(), chars.as_ptr() as _) } == 0 {
            panic!("PL_put_atom_nchars failed");
        }

        *self
    }

    pub fn get_atom_chars(&self) -> &str {
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

impl fmt::Debug for Term<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_fmt(format_args!("term@{:?}", self.ptr))
    }
}
