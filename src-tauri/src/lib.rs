use std::fs::File;
use std::sync::{LazyLock, RwLock};
use tauri::{App, Manager};
use tauri_plugin_store::StoreExt;

mod api;
mod util;

static SESSION: LazyLock<factbook_core::Session> =
    LazyLock::new(|| factbook_core::Session::new().expect("failed to initialize session"));

const SETTINGS_PATH: &str = "settings.json";
const SETTING_JOURNAL_PATH: &str = "journal_path";

#[derive(Default)]
pub enum AppState {
    /// Initial screen with journal file picker
    #[default]
    Start,
    /// Main state with entries and views
    Journal(factbook_core::State<'static>),
}

impl AppState {
    pub fn journal(&self) -> &factbook_core::State<'static> {
        match self {
            Self::Journal(state) => state,
            _ => panic!("expected AppState to be in Journal state"),
        }
    }

    pub fn journal_mut(&mut self) -> &mut factbook_core::State<'static> {
        match self {
            Self::Journal(state) => state,
            _ => panic!("expected AppState to be in Journal state"),
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
            api::get_state,
            api::open_journal,
            api::close_journal,
            api::get_views,
            api::create_view,
            api::remove_view,
            api::set_view_name,
            api::set_view_definition,
            api::get_entries,
            api::create_entry,
            api::remove_entry,
            api::set_entry_content,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    let state = app
        .store(SETTINGS_PATH)?
        .get(SETTING_JOURNAL_PATH)
        .and_then(|path| {
            let file = File::open(path.as_str()?).unwrap();
            let journal = serde_json::from_reader(file).unwrap();

            let journal_state = factbook_core::State::new(&SESSION);
            journal_state.load_journal(journal);
            Some(AppState::Journal(journal_state))
        })
        .unwrap_or_default();

    app.manage(RwLock::new(state));

    Ok(())
}
