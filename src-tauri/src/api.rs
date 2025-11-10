use crate::model::{EntryId, ViewId};
use crate::util::SerializeIterOnce;
use crate::AppState;
use chrono::{DateTime, Local};
use factbook_swipl::term;
use serde::Serialize;
use tauri::{ipc, State};

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
    log::debug!("get_all_entries({view:?})");

    let pl = state.swipl_session.engine();
    let var = pl.new_term();
    pl.call(term! { pl => foo({var}) });
    log::debug!("{}", var.atom_chars().unwrap());

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
