use chrono::{DateTime, Local};
use serde::Serialize;
use tauri::{ipc, State};

use crate::model::{EntryId, ViewId};
use crate::util::SerializeIterOnce;
use crate::AppState;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View<'t> {
    id: ViewId,
    name: &'t str,
    entry_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry<'t> {
    id: EntryId,
    created_at: DateTime<Local>,
    content: &'t str,
}

#[tauri::command]
pub fn get_views(state: State<AppState>) -> ipc::Response {
    let state = state.persistent_state.read().unwrap();
    let views = state.views.iter().map(|(&id, view)| View {
        id,
        name: &view.name,
        entry_count: 0,
    });

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(views)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn get_entries(state: State<AppState>, view: Option<ViewId>) -> ipc::Response {
    use swipl::prelude::*;

    log::debug!("get_all_entries({view:?})");

    let swipl_context = state.swipl_context.lock().unwrap();
    let var = swipl_context.new_term_ref();
    swipl_context.call_once(pred!(foo / 1), [&var]).unwrap();
    var.get_atom_name(|result| log::debug!("{}", result.unwrap())).unwrap();

    let state = state.persistent_state.read().unwrap();
    let entries = state.entries.iter().map(|(&id, entry)| Entry {
        id,
        created_at: entry.created_at,
        content: &entry.content,
    });

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(entries)).unwrap();
    ipc::Response::new(response)
}
