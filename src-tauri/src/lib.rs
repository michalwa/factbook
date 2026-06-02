use std::sync::{LazyLock, RwLock};

mod api;
mod util;

static SESSION: LazyLock<factbook_core::Session> =
    LazyLock::new(|| factbook_core::Session::new().expect("failed to initialize session"));

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
        .manage(RwLock::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            api::get_state,
            api::open_journal,
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
