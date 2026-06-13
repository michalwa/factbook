use crate::util::SerializeIterOnce;
use crate::{AppState, SETTING_JOURNAL_PATH, SETTINGS_PATH};
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
use std::fs::OpenOptions;
use std::sync::RwLock;
use tauri::{AppHandle, Manager, State, ipc};
use tauri_plugin_store::StoreExt;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View<'a> {
    id: ViewId,
    #[serde(flatten)]
    view: &'a model::View,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry<'a> {
    id: EntryId,
    #[serde(flatten)]
    entry: &'a model::Entry,
}

#[tauri::command]
pub fn get_state(state: State<RwLock<AppState>>) -> &'static str {
    match *state.read().unwrap() {
        AppState::Start => "start",
        AppState::Journal { .. } => "journal",
    }
}

#[tauri::command]
pub fn get_journal_path(state: State<RwLock<AppState>>) -> Option<String> {
    state.read().unwrap().journal_path().map(String::from)
}

#[tauri::command]
pub fn open_journal(app: AppHandle, path: &str) {
    let state = app.state::<RwLock<AppState>>();
    state.write().unwrap().open_journal(path).unwrap();

    let store = app.store(SETTINGS_PATH).unwrap();
    store.set(SETTING_JOURNAL_PATH, path);
}

#[tauri::command]
pub fn close_journal(app: AppHandle) {
    let state = app.state::<RwLock<AppState>>();
    *state.write().unwrap() = AppState::Start;

    let store = app.store(SETTINGS_PATH).unwrap();
    store.delete(SETTING_JOURNAL_PATH);
}

#[tauri::command]
pub fn save_journal(state: State<RwLock<AppState>>) {
    let state = state.read().unwrap();
    let path = state.journal_path().unwrap();
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();

    serde_json::to_writer_pretty(file, &state.journal().to_journal()).unwrap();
}

#[tauri::command]
pub fn get_views(state: State<RwLock<AppState>>) -> ipc::Response {
    let state = state.read().unwrap();
    let views = state.journal().views();
    let views = views
        .iter()
        .map(|(id, view)| View { id, view })
        .collect::<Vec<_>>();

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(views)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn create_view(state: State<RwLock<AppState>>) -> ViewId {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().create()
}

#[tauri::command]
pub fn remove_view(state: State<RwLock<AppState>>, id: ViewId) {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().remove(id);
}

#[tauri::command]
pub fn set_view_name(state: State<RwLock<AppState>>, id: ViewId, name: String) {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().set_name(id, name);
}

#[tauri::command]
pub fn set_view_definition(state: State<RwLock<AppState>>, id: ViewId, definition: String) {
    let mut state = state.write().unwrap();
    state.journal_mut().set_view_definition(id, definition);
}

#[tauri::command]
pub fn get_entries(state: State<RwLock<AppState>>, view: Option<ViewId>) -> ipc::Response {
    use serde::Serializer;
    use serde::ser::SerializeSeq;

    let state = state.read().unwrap();
    let state = state.journal();

    let mut json = Vec::new();
    let mut serializer = serde_json::Serializer::new(&mut json);
    let mut seq = serializer.serialize_seq(None).unwrap();

    if let Some(view) = view {
        state.for_each_view_entry(view, |id, entry| {
            seq.serialize_element(&Entry { id, entry }).unwrap()
        });
    } else {
        state
            .entries()
            .iter()
            .for_each(|(id, entry)| seq.serialize_element(&Entry { id, entry }).unwrap());
    }

    seq.end().unwrap();

    // SAFETY: `serde_json::Serializer` does not emit invalid UTF-8
    ipc::Response::new(unsafe { String::from_utf8_unchecked(json) })
}

#[tauri::command]
pub fn create_entry(state: State<RwLock<AppState>>) -> EntryId {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().create()
}

#[tauri::command]
pub fn remove_entry(state: State<RwLock<AppState>>, id: EntryId) {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().remove(id);
}

#[tauri::command]
pub fn set_entry_content(
    state: State<RwLock<AppState>>,
    id: EntryId,
    content: String,
) -> ipc::Response {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().set_content(id, content);

    let entries = state.journal().entries();
    ipc::Response::new(serde_json::to_string(entries.get(id)).unwrap())
}
