use crate::AppState;
use crate::model::{EntryId, ViewId};
use crate::util::SerializeIterOnce;
use chrono::{DateTime, Local};
use factbook_swipl::{Context, term};
use serde::Serialize;
use tauri::{State, ipc};

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
    let db = state.database.read().unwrap();
    let views = db.views.iter().map(|(&id, view)| View {
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
    let db = state.database.read().unwrap();
    let cache = state.cache.read().unwrap();

    let mut engine = state.swipl_session.engine();
    let pl = engine.frame();

    if let Some(view_id) = view {
        let view = db.views.get(&view_id).unwrap();
        let module_name = format!("view_{view_id}");

        pl.load_module_from_str(&module_name, &view.definition);

        if pl.predicate_defined::<1>("show", module_name.as_str()) {
            log::debug!("show/1 exists");
        } else {
            log::debug!("show/1 doesn't exist");
        }
    }

    let state = state.database.read().unwrap();
    let entries = state
        .entries
        .iter()
        .inspect(|(id, _)| {
            for tag in cache.entry_tags.get(id).unwrap() {
                log::debug!("entry {id:?} has the tag: {}", pl.new_term().put(tag));
            }
        })
        .map(|(&id, entry)| Entry {
            id,
            created_at: entry.created_at,
            content: &entry.content,
        });

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(entries)).unwrap();
    ipc::Response::new(response)
}
