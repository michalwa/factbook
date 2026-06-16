use crate::AppState;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock, RwLockReadGuard};
use tauri::ipc::{CommandArg, CommandItem, InvokeError};
use tauri::{App, Manager, Runtime, WebviewUrl, WebviewWindow, WebviewWindowBuilder, Window};

/// Manages instances of a state type per window
struct WindowStateManager<T> {
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
            .and_then(|manager| {
                let manager = Arc::clone(&*manager);

                // SAFETY: Guard is valid as long as the `RwLock` is valid,
                // which is kept valid by the `Arc`
                let guard = unsafe {
                    std::mem::transmute::<
                        RwLockReadGuard<'_, WindowStates<T>>,
                        RwLockReadGuard<'static, WindowStates<T>>,
                    >(manager.states.read().unwrap())
                };

                // SAFETY: Reference is valid as long as `guard` is valid, the `HashMap`
                // is ensured to be immutable while `guard` is held
                let state = unsafe { &*(guard.get(window.label())? as *const T) };

                Some(WindowState {
                    state,
                    _guard: guard,
                    _manager: manager,
                })
            })
            .ok_or_else(|| {
                InvokeError::from(format!(
                    "window state not managed for field `{}` on command `{}`",
                    command.key, command.name
                ))
            })
    }
}

trait ManageScopedInternal {
    fn label(&self) -> &str;
}

pub trait ManageScoped<R: Runtime> {
    fn manage_scoped<T: Send + Sync + 'static>(&self, state: T) -> bool;
}

impl<R: Runtime, M: Manager<R> + ManageScopedInternal> ManageScoped<R> for M {
    fn manage_scoped<T: Send + Sync + 'static>(&self, state: T) -> bool {
        self.manage::<Arc<WindowStateManager<T>>>(Default::default());
        self.state::<Arc<WindowStateManager<T>>>()
            .states
            .write()
            .unwrap()
            .insert(self.label().into(), state)
            .is_some()
    }
}

impl ManageScopedInternal for Window {
    fn label(&self) -> &str {
        self.label()
    }
}

impl ManageScopedInternal for WebviewWindow {
    fn label(&self) -> &str {
        self.label()
    }
}

#[derive(Default)]
struct WindowCounter(AtomicUsize);

pub fn open(app: &App, state: AppState) {
    app.manage(WindowCounter::default());

    let id = app
        .state::<WindowCounter>()
        .0
        .fetch_add(1, Ordering::SeqCst);

    let window = WebviewWindowBuilder::new(app, format!("main{id}"), WebviewUrl::default())
        .title("factbook")
        .inner_size(800.0, 600.0)
        .on_document_title_changed(|window, title| {
            window.set_title(&title).unwrap();
        })
        .build()
        .unwrap();

    window.manage_scoped(RwLock::new(state));
}
