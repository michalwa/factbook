use std::sync::Arc;
use tauri::ipc::{CommandArg, CommandItem, InvokeError};
use tauri::{AppHandle, Runtime};
use tauri_plugin_store::{Store, StoreExt};

const SETTINGS_PATH: &str = "settings.json";
const SETTING_LAST_JOURNAL_PATH: &str = "last_journal_path";

/// Type-safety & convenience wrapper around a [`Store`] for persistent user
/// settings
pub struct Settings<R: Runtime = tauri::Wry>(Arc<Store<R>>);

impl<'de, R: Runtime> CommandArg<'de, R> for Settings<R> {
    fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
        AppHandle::from_command(command).map(|app| app.settings())
    }
}

impl<R: Runtime> Settings<R> {
    pub fn last_journal_path(&self) -> Option<String> {
        Some(self.0.get(SETTING_LAST_JOURNAL_PATH)?.as_str()?.to_owned())
    }

    pub fn set_last_journal_path(&self, path: impl Into<String>) {
        self.0.set(SETTING_LAST_JOURNAL_PATH, path.into());
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
