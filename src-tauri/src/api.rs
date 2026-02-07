use crate::util::SerializeIterOnce;
use chrono::{DateTime, Local};
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
use tauri::{State, ipc};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View<'t> {
    id: ViewId,
    #[serde(flatten)]
    view: &'t model::View,
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
pub fn get_views(state: State<factbook_core::State>) -> ipc::Response {
    let db = state.database();
    let views = db.views.iter().map(|(&id, view)| View {
        id,
        view,
        entry_count: 0,
    });

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(views)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn create_view(state: State<factbook_core::State>) -> ViewId {
    let id = state.cache_mut().next_view_id();
    state.database_mut().views.insert(id, Default::default());
    id
}

#[tauri::command]
pub fn set_view_name(state: State<factbook_core::State>, id: ViewId, name: &str) {
    state.database_mut().views.get_mut(&id).unwrap().name = name.into();
}

#[tauri::command]
pub fn set_view_definition(state: State<factbook_core::State>, id: ViewId, definition: &str) {
    state.database_mut().views.get_mut(&id).unwrap().definition = definition.into();
}

#[tauri::command]
pub fn remove_view(state: State<factbook_core::State>, id: ViewId) {
    state.database_mut().views.remove(&id);
}

#[tauri::command]
pub fn get_entries(state: State<factbook_core::State>, view: Option<ViewId>) -> ipc::Response {
    let database = state.database();
    let cache = state.cache();

    let entries =
        factbook_core::search::get_entries(&database, &cache, &mut state.pl_engine(), view).map(
            |(id, entry)| Entry {
                id,
                created_at: entry.created_at,
                content: &entry.content,
            },
        );

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(entries)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn set_entry_content(state: State<factbook_core::State>, id: EntryId, content: &str) {
    state.database_mut().entries.get_mut(&id).unwrap().content = content.to_owned();

    let mut pl = state.pl_engine();
    state.cache_mut().update_entry(&mut pl, id, content);
}

#[tauri::command]
pub fn create_entry(state: State<factbook_core::State>) -> EntryId {
    let id = state.cache_mut().next_entry_id();
    state.database_mut().entries.insert(id, Default::default());
    id
}

#[tauri::command]
pub fn remove_entry(state: State<factbook_core::State>, id: EntryId) {
    state.database_mut().entries.remove(&id);
    state.cache_mut().entry_tags.remove(&id);
}
