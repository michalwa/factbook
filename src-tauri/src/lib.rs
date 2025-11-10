use crate::model::PersistentState;
use factbook_swipl::term;
use std::fs::File;
use std::io::BufReader;
use std::sync::RwLock;
use tauri::Manager;
use tauri_plugin_cli::CliExt;

mod api;
mod model;
mod util;

const SWIPL_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

struct AppState {
    persistent_state: RwLock<PersistentState>,
    swipl_session: factbook_swipl::Session<'static>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_cli::init())?;

            let mut persistent_state = None;

            if let Ok(matches) = app.cli().matches() {
                if let Some(help) = matches.args.get("help") {
                    println!("{}", help.value.as_str().unwrap());
                    app.handle().exit(0);
                }

                if let Some(file) = matches.args.get("file") {
                    let file = File::open(file.value.as_str().unwrap())?;
                    persistent_state = Some(serde_json::from_reader(BufReader::new(file))?);
                }
            }

            let Some(persistent_state) = persistent_state else {
                panic!("No journal file loaded");
            };

            let swipl_session = factbook_swipl::Session::init(SWIPL_STATE).unwrap();
            let pl = swipl_session.engine();
            pl.assert(term! { pl => foo(bar) }, Default::default());

            let state = AppState {
                persistent_state: RwLock::new(persistent_state),
                swipl_session,
            };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::get_views, api::get_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
