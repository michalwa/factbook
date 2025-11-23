use crate::AppState;
use crate::model::{EntryId, ViewId};
use crate::prolog::predicates::EntryTags;
use crate::util::SerializeIterOnce;
use chrono::{DateTime, Local};
use factbook_swipl::Context;
use factbook_swipl::blob::ScopedBlob;
use factbook_swipl::query::open_query;
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
    let mut pl = engine.frame();

    let entry_tags = EntryTags::new(&cache.entry_tags);
    let entry_tags_blob = ScopedBlob::new(&entry_tags);

    let state = state.database.read().unwrap();

    let entries: Box<dyn Iterator<Item = (EntryId, &crate::model::Entry)>> =
        if let Some(view_id) = view {
            let mut entry_ids = Vec::new();
            let view = db.views.get(&view_id).unwrap();
            let module_name = format!("view_{view_id}");

            pl.load_module_from_str(&module_name, &view.definition)
                .unwrap();

            let query = open_query! { pl => {&module_name}:show({&entry_tags_blob}, _) }.unwrap();
            while let Some([_, entry_id]) = query.next_solution().unwrap() {
                // TODO: impl Iterator for Query
                entry_ids.push(entry_id.get::<EntryId>().unwrap());
            }

            Box::new(entry_ids.into_iter().map(|id| (id, &state.entries[&id])))
        } else {
            Box::new(state.entries.iter().map(|(id, entry)| (*id, entry)))
        };

    let entries = entries.map(|(id, entry)| Entry {
        id,
        created_at: entry.created_at,
        content: &entry.content,
    });

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(entries)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn set_entry_content(state: State<AppState>, entry_id: EntryId, content: &str) {
    let mut db = state.database.write().unwrap();

    db.entries.get_mut(&entry_id).unwrap().content = content.to_owned();

    let pl = state.swipl_session.engine();
    let mut cache = state.cache.write().unwrap();
    let tags = cache.entry_tags.get_mut(&entry_id).unwrap();
    tags.clear();
    tags.extend(crate::prolog::parse(content, &pl).map(|t| t.record()));

    log::debug!("updated entry {entry_id:?}");
}
