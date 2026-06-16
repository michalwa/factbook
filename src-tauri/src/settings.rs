use std::sync::Arc;
use tauri::ipc::{CommandArg, CommandItem, InvokeError};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_PATH: &str = "settings.json";
const SETTING_OPEN_JOURNALS: &str = "openJournals";

/// Type-safety & convenience wrapper around a [`Store`] for persistent user
/// settings
pub struct Settings<R: Runtime = tauri::Wry>(Arc<Store<R>>);

impl<'de, R: Runtime> CommandArg<'de, R> for Settings<R> {
    fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
        AppHandle::from_command(command).map(|app| app.settings())
    }
}

impl<R: Runtime> Settings<R> {
    pub fn open_journals(&self) -> Option<Vec<String>> {
        self.0
            .get(SETTING_OPEN_JOURNALS)
            .and_then(|paths| paths.as_array().cloned())
            .map(|paths| {
                paths
                    .into_iter()
                    .filter_map(|path| path.as_str().map(|p| p.into()))
                    .collect()
            })
    }

    pub fn set_open_journals(&self, value: Vec<String>) {
        self.0.set(SETTING_OPEN_JOURNALS, value);
    }
}

pub trait SettingsExt<R: Runtime> {
    fn settings(&self) -> Settings<R>;
}

impl<S: StoreExt<R>, R: Runtime> SettingsExt<R> for S {
    fn settings(&self) -> Settings<R> {
        Settings(self.store(SETTINGS_PATH).unwrap())
    }
}
