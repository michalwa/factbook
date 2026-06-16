use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::ipc::{CommandArg, CommandItem, InvokeError};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_PATH: &str = "settings.json";
const SETTING_LAST_SAVED_JOURNAL: &str = "lastSavedJournal";

/// Type-safety & convenience wrapper around a [`Store`] for persistent user
/// settings
pub struct Settings<R: Runtime = tauri::Wry>(Arc<Store<R>>);

impl<'de, R: Runtime> CommandArg<'de, R> for Settings<R> {
    fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
        AppHandle::from_command(command).map(|app| app.settings())
    }
}

impl<R: Runtime> Settings<R> {
    pub fn last_saved_journal(&self) -> Option<PathBuf> {
        self.0
            .get(SETTING_LAST_SAVED_JOURNAL)
            .and_then(|v| v.as_str().map(PathBuf::from))
    }

    pub fn set_last_saved_journal(&self, path: impl AsRef<Path>) {
        self.0
            .set(SETTING_LAST_SAVED_JOURNAL, path.as_ref().to_str().unwrap());
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
