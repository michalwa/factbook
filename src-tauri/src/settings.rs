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
        let value = self.0.get(SETTING_OPEN_JOURNALS)?;
        value.as_array().map(|paths| {
            paths
                .iter()
                .filter_map(|p| p.as_str().map(String::from))
                .collect()
        })
    }

    /// Appends a path to the persisted list of open journals
    pub fn open_journal(&self, path: String) {
        let mut paths = self.open_journals().unwrap_or_default();
        paths.push(path);
        self.0.set(SETTING_OPEN_JOURNALS, paths);
    }

    /// Removes a path from the persisted list of open journals, ensuring at
    /// least one path is left
    pub fn close_journal(&self, path: &str) {
        let mut paths = self.open_journals().unwrap_or_default();
        if paths.len() > 1 {
            paths.retain(|p| p != path);
            self.0.set(SETTING_OPEN_JOURNALS, paths);
        }
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
