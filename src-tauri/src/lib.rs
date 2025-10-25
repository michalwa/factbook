use std::fs::File;
use std::io::BufReader;
use std::sync::RwLock;
use tauri::Manager;
use tauri_plugin_cli::CliExt;

use crate::model::PersistentState;

mod api;
mod model;
mod util;

#[derive(Default)]
struct AppState {
    persistent_state: PersistentState,
}

type AppStateRef = RwLock<AppState>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_cli::init())?;

            app.manage(AppStateRef::default());

            if let Ok(matches) = app.cli().matches() {
                if let Some(help) = matches.args.get("help") {
                    println!("{}", help.value.as_str().unwrap());
                    app.handle().exit(0);
                }

                if let Some(file) = matches.args.get("file") {
                    let state = app.state::<AppStateRef>();
                    let file = File::open(file.value.as_str().unwrap())?;
                    state.write().unwrap().persistent_state =
                        serde_json::from_reader(BufReader::new(file))?;
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            api::get_views,
            api::get_entries
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
