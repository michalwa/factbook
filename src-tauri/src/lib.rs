use crate::model::{Cache, Database};
use factbook_swipl::Context;
use std::fs::File;
use std::io::BufReader;
use std::sync::RwLock;
use tauri::{App, Manager};
use tauri_plugin_cli::CliExt;

mod api;
mod model;
mod prolog;
mod util;

const SWIPL_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

struct AppState {
    database: RwLock<Database>,
    cache: RwLock<Cache>,
    swipl_session: factbook_swipl::Session<'static>,
}

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
            api::set_view_name,
            api::set_view_definition,
            api::remove_view,
            api::get_entries,
            api::set_entry_content,
            api::create_entry,
            api::remove_entry,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn setup(app: &mut App) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(desktop)]
    app.handle().plugin(tauri_plugin_cli::init())?;

    let mut database = None;

    if let Ok(matches) = app.cli().matches() {
        if let Some(help) = matches.args.get("help") {
            println!("{}", help.value.as_str().unwrap());
            app.handle().exit(0);
        }

        if let Some(file) = matches.args.get("file") {
            let file = File::open(file.value.as_str().unwrap())?;
            database = Some(serde_json::from_reader(BufReader::new(file))?);
        }
    }

    let Some(database) = database else {
        panic!("No journal file loaded");
    };

    let swipl_session = factbook_swipl::Session::init(SWIPL_STATE).unwrap();
    let mut pl = swipl_session.engine();
    pl.register_predicate::<prolog::predicates::Tag>();
    let cache = Cache::init_from(&database, &mut pl.frame());

    let state = AppState {
        database: RwLock::new(database),
        cache: RwLock::new(cache),
        swipl_session,
    };

    app.manage(state);

    Ok(())
}
