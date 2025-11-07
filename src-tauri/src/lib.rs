use crate::model::PersistentState;
use std::fs::File;
use std::io::BufReader;
use std::ops::Deref;
use std::sync::{Mutex, RwLock};
use swipl::prelude::{
    initialize_swipl_with_state, term, ActivatedEngine as SwiplActivatedEngine,
    Context as SwiplContext,
};
use tauri::Manager;
use tauri_plugin_cli::CliExt;

mod api;
mod model;
mod util;

const PROLOG_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

struct SendSwiplContext(Mutex<SwiplContext<'static, SwiplActivatedEngine<'static>>>);

// SAFETY: Everything is behind a `Mutex`
unsafe impl Sync for SendSwiplContext {}
unsafe impl Send for SendSwiplContext {}

impl Deref for SendSwiplContext {
    type Target = Mutex<SwiplContext<'static, SwiplActivatedEngine<'static>>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct AppState {
    persistent_state: RwLock<PersistentState>,
    swipl_context: SendSwiplContext,
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

            let swipl_context: SwiplContext<_> = initialize_swipl_with_state(PROLOG_STATE)
                .expect("failed to initialize SWI-Prolog")
                .into();

            swipl_context
                .assert(
                    &term! {swipl_context: foo(bar)}.unwrap(),
                    Default::default(),
                )
                .unwrap();

            let state = AppState {
                persistent_state: RwLock::new(persistent_state),
                swipl_context: SendSwiplContext(Mutex::new(swipl_context)),
            };

            app.manage(state);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![api::get_views, api::get_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
