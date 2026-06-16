use crate::settings::SettingsExt;
use std::fs::File;
use std::path::PathBuf;
use std::sync::LazyLock;
use tauri::{App, Manager};

mod api;
mod settings;
mod util;
mod window;

static SESSION: LazyLock<factbook_core::Session> =
    LazyLock::new(|| factbook_core::Session::new().expect("failed to initialize session"));

pub struct AppState {
    journal_path: Option<PathBuf>,
    journal: factbook_core::State<'static>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            journal_path: None,
            journal: factbook_core::State::new(&SESSION),
        }
    }
}

impl AppState {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self, Box<dyn std::error::Error>> {
        let path = path.into();
        let journal_state = factbook_core::State::new(&SESSION);

        if let Ok(file) = File::open(&path) {
            let journal = serde_json::from_reader(file)?;
            journal_state.load_journal(journal);
        } else {
            log::warn!("journal file doesn't exist, saving will create it");
        }

        Ok(Self {
            journal_path: Some(path),
            journal: journal_state,
        })
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
    if let Some(paths) = app.settings().open_journals() {
        for path in paths {
            log::info!("loading journal file: {path}");
            window::open(app, AppState::open(path)?);
        }
    } else {
        window::open(app, AppState::default());
    }

    Ok(())
}
