use std::sync::RwLock;
use tauri::{Manager, State};
use tauri_plugin_cli::CliExt;

use crate::{model::Entry, persistence::read_journal_file};

mod model;
mod persistence;

#[derive(Default)]
struct AppState {
    entries: Vec<Entry>,
}

#[tauri::command]
fn get_all_entries(state: State<'_, RwLock<AppState>>) -> Vec<Entry> {
    state.read().unwrap().entries.clone()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_cli::init())?;

            app.manage(RwLock::new(AppState::default()));

            match app.cli().matches() {
                Ok(matches) if matches.args.contains_key("help") => {
                    println!("{}", matches.args["help"].value.as_str().unwrap());
                    app.handle().exit(0);
                }
                Ok(matches) if matches.args.contains_key("file") => {
                    let state = app.state::<RwLock<AppState>>();
                    state.write().unwrap().entries =
                        read_journal_file(matches.args["file"].value.as_str().unwrap())?;
                }
                _ => {}
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_all_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
