use std::sync::LazyLock;
use tauri::{App, Manager};

mod api;
mod util;

static SESSION: LazyLock<factbook_core::Session> =
    LazyLock::new(|| factbook_core::Session::new().expect("failed to initialize session"));

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_prevent_default::debug())
        .setup(setup)
        .invoke_handler(tauri::generate_handler![
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
    let state = factbook_core::State::new(&SESSION);

    // TODO: Load journal file
    {
        let mut entries = state.entries_mut();
        let e1 = entries.create();
        entries.set_content(e1, "@todo walk the dog @due(today)".into());
    }

    app.manage(state);

    Ok(())
}
