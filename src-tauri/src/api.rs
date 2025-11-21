use crate::AppState;
use crate::model::{EntryId, ViewId};
use crate::prolog::predicates::EntryTags;
use crate::util::SerializeIterOnce;
use chrono::{DateTime, Local};
use factbook_swipl::blob::ScopedBlob;
use factbook_swipl::{Context, term};
use serde::Serialize;
use std::cell::RefCell;
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

    let entry_tags = EntryTags(RefCell::new(cache.entry_tags.iter()));
    let entry_tags_blob = ScopedBlob::new(&entry_tags);
    let t_entry_tags = pl.new_term().put(&entry_tags_blob);

    log::debug!("{t_entry_tags}");

    if let Some(view_id) = view {
        let view = db.views.get(&view_id).unwrap();
        let module_name = format!("view_{view_id}");

        pl.load_module_from_str(&module_name, &view.definition);

        if pl.predicate_defined::<2>("show", module_name.as_str()) {
            log::debug!("show/2 exists");
            pl.call_module(&module_name, term! { &pl => show({t_entry_tags}, 1) });
        } else {
            log::debug!("show/2 doesn't exist");
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
