use std::any::type_name;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tauri::ipc::{CommandArg, CommandItem, InvokeError};
use tauri::{
    AppHandle, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window,
    WindowEvent,
};

pub trait WindowStateData<R: Runtime>: Send + Sync + 'static {
    /// Called on managed window-scoped state when the window is destroyed
    fn cleanup(self, app: &AppHandle<R>);
}

/// Manages instances of a state type per window
pub struct WindowStateManager<T> {
    states: RwLock<WindowStates<T>>,
}

type WindowStates<T> = HashMap<String, T>;

impl<T> Default for WindowStateManager<T> {
    fn default() -> Self {
        Self {
            states: Default::default(),
        }
    }
}

impl<T: Send + Sync + 'static> WindowStateManager<T> {
    pub fn states(&self) -> RwLockReadGuard<'_, WindowStates<T>> {
        self.states.read().unwrap()
    }

    fn get(self: &Arc<Self>, key: &str) -> Option<WindowState<'static, T>> {
        let manager = Arc::clone(self);

        // SAFETY: Guard is valid as long as the `RwLock` is valid, which is kept
        // valid by the `Arc`
        let guard = unsafe {
            std::mem::transmute::<
                RwLockReadGuard<'_, WindowStates<T>>,
                RwLockReadGuard<'static, WindowStates<T>>,
            >(manager.states.read().unwrap())
        };

        // SAFETY: Reference is valid as long as `guard` is valid, the `HashMap`
        // is ensured to be immutable while `guard` is held
        let state = unsafe { &*(guard.get(key)? as *const T) };

        Some(WindowState {
            state,
            _guard: guard,
            _manager: manager,
        })
    }
}

/// Managed state scoped to a single window
pub struct WindowState<'a, T: Send + Sync + 'static> {
    state: &'a T,
    // SAFETY: `_guard` must be declared before `_manager` in order to ensure
    // the `RwLock` is still valid on `drop`
    _guard: RwLockReadGuard<'static, WindowStates<T>>,
    _manager: Arc<WindowStateManager<T>>,
}

impl<T: Send + Sync + 'static> Deref for WindowState<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.state
    }
}

impl<'r, 'de: 'r, T: Send + Sync + 'static, R: Runtime> CommandArg<'de, R> for WindowState<'r, T> {
    fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
        let window = command.message.webview_ref().window();

        command
            .message
            .state_ref()
            .try_get::<Arc<WindowStateManager<T>>>()
            .and_then(|manager| manager.get(window.label()))
            .ok_or_else(|| {
                InvokeError::from(format!(
                    "window state not managed for field `{}` on command `{}`",
                    command.key, command.name
                ))
            })
    }
}

trait AnyWindow {
    fn label(&self) -> &str;
    fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F);
}

pub trait WindowScopedManager<R: Runtime> {
    /// Registers state to be managed in the scope of the window.
    fn manage_window_scoped<T: WindowStateData<R>>(&self, state: T) -> bool;
    /// Fetches state managed in the scope of the window.
    /// In commands, prefer using a [`WindowState`] argument.
    #[allow(unused)] // Used in tests and added for consistency with [`tauri::Manager::state`]
    fn state_window_scoped<T: Send + Sync + 'static>(&self) -> WindowState<'_, T>;
}

impl<R: Runtime, M: Manager<R> + AnyWindow> WindowScopedManager<R> for M {
    fn manage_window_scoped<T: WindowStateData<R>>(&self, state: T) -> bool {
        self.manage::<Arc<WindowStateManager<T>>>(Default::default());

        let replaced = self
            .state::<Arc<WindowStateManager<T>>>()
            .states
            .write()
            .unwrap()
            .insert(self.label().into(), state)
            .is_some();

        log::debug!("attached state to window {:?}", self.label());

        if !replaced {
            let handle = self.app_handle().clone();
            let label = self.label().to_owned();

            self.on_window_event(move |e| match e {
                WindowEvent::Destroyed => {
                    log::debug!("window {label:?} destroyed, dropping state");

                    if let Some(manager) = handle.try_state::<Arc<WindowStateManager<T>>>()
                        && let Some(data) = manager.states.write().unwrap().remove(&label)
                    {
                        data.cleanup(&handle);
                    }
                },
                _ => (),
            });
        }

        replaced
    }

    fn state_window_scoped<T: Send + Sync + 'static>(&self) -> WindowState<'_, T> {
        self.state::<Arc<WindowStateManager<T>>>()
            .get(self.label())
            .unwrap_or_else(|| {
                panic!(
                    "state_window_scoped() called before manage_window_scoped() for {}",
                    type_name::<T>()
                )
            })
    }
}

impl<R: Runtime> AnyWindow for Window<R> {
    fn label(&self) -> &str {
        self.label()
    }

    fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
        self.on_window_event(f);
    }
}

impl<R: Runtime> AnyWindow for WebviewWindow<R> {
    fn label(&self) -> &str {
        self.label()
    }

    fn on_window_event<F: Fn(&WindowEvent) + Send + 'static>(&self, f: F) {
        self.on_window_event(f);
    }
}

pub trait WindowStatesExt<R: Runtime> {
    /// Returns a reference to the storage of all window-scoped states of type
    /// `T`
    fn window_states<T: Send + Sync + 'static>(&self) -> Arc<WindowStateManager<T>>;
}

impl<R: Runtime, M: Manager<R>> WindowStatesExt<R> for M {
    fn window_states<T: Send + Sync + 'static>(&self) -> Arc<WindowStateManager<T>> {
        Arc::clone(&*self.state::<Arc<WindowStateManager<T>>>())
    }
}

#[derive(Default)]
struct WindowCounter(AtomicUsize);

/// Opens a new app window with the given window-scoped state attached
///
/// NOTE: Commands calling this function must be `async` to avoid deadlocks on
/// Windows!
/// https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html#known-issues
pub fn open<R, M, S>(app: &M, state: S) -> WebviewWindow<R>
where
    R: Runtime,
    M: Manager<R>,
    S: WindowStateData<R>,
{
    app.manage(WindowCounter::default());

    let id = app
        .state::<WindowCounter>()
        .0
        .fetch_add(1, Ordering::SeqCst);

    let label = format!("main{id}");
    let window = WebviewWindowBuilder::new(app, &label, WebviewUrl::default())
        .title("factbook")
        .inner_size(800.0, 600.0)
        .on_document_title_changed(|window, title| {
            window.set_title(&title).unwrap();
        })
        .build()
        .unwrap();

    window.manage_window_scoped(state);
    window
}
