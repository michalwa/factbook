use crate::settings::SettingsExt;
use crate::window::WindowStateData;
use std::fs::File;
use std::path::PathBuf;
use std::sync::{LazyLock, RwLock};
use tauri::{App, AppHandle, Manager, Runtime};

mod api;
mod settings;
mod util;
mod window;

const DEFAULT_JOURNAL_PATH: &str = "welcome.json";

static SESSION: LazyLock<factbook_core::Session> =
    LazyLock::new(|| factbook_core::Session::new().expect("failed to initialize session"));

pub struct AppState {
    journal_path: Option<PathBuf>,
    journal: factbook_core::State<'static>,
    default: bool,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            journal_path: None,
            journal: factbook_core::State::new(&SESSION),
            default: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenMode {
    /// Load the journal and set it for writing
    Edit,
    /// Load the journal but don't set it for writing
    Read,
}

impl AppState {
    pub fn open(
        path: impl Into<PathBuf>,
        mode: OpenMode,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.into();
        log::info!("loading journal file: {}", path.display());

        let journal_state = factbook_core::State::new(&SESSION);

        match File::open(&path) {
            Ok(file) => {
                let journal = serde_json::from_reader(file)?;
                journal_state.load_journal(journal);
            },
            err => match mode {
                OpenMode::Edit => {
                    log::warn!("journal file doesn't exist, saving will create it")
                },
                OpenMode::Read => {
                    err?;
                },
            },
        }

        Ok(Self {
            journal_path: match mode {
                OpenMode::Edit => Some(path),
                OpenMode::Read => None,
            },
            journal: journal_state,
            default: false,
        })
    }

    pub fn open_default<M: Manager<R>, R: Runtime>(
        m: &M,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let default_journal_path = m
            .path()
            .resolve(DEFAULT_JOURNAL_PATH, tauri::path::BaseDirectory::Resource)
            .unwrap();

        let mut state = Self::open(default_journal_path, OpenMode::Read)?;
        state.default = true;
        Ok(state)
    }
}

impl<R: Runtime> WindowStateData<R> for RwLock<AppState> {
    fn cleanup(self, app: &AppHandle<R>) {
        if let Some(path) = self.into_inner().unwrap().journal_path {
            app.settings()
                .set_last_journal_path(path.to_str().expect("path is not valid unicode"));
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _, _| {
            if let Some(window) = app.webview_windows().values().next() {
                let _ = window.set_focus();
            }
        }));
    }

    builder
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            api::get_journal_path,
            api::create_journal,
            api::open_journal,
            api::save_journal,
            api::open_default_journal,
            api::get_views,
            api::create_view,
            api::remove_view,
            api::set_view_name,
            api::set_view_definition,
            api::get_entries,
            api::create_entry,
            api::remove_entry,
            api::set_entry_content,
            api::parse_entry_content,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri application");
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    match app.settings().last_journal_path() {
        Some(path) => window::open(app, RwLock::new(AppState::open(path, OpenMode::Edit)?)),
        None => window::open(app, RwLock::new(AppState::open_default(app)?)),
    };

    Ok(())
}
