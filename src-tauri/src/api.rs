use crate::util::SerializeIterOnce;
use crate::window::WindowState;
use crate::{AppState, SETTING_OPEN_JOURNALS, SETTINGS_PATH};
use factbook_core::lang::{self, Span};
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
use std::fs::OpenOptions;
use std::sync::RwLock;
use tauri::{AppHandle, ipc};
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
pub fn get_state(state: WindowState<RwLock<AppState>>) -> &'static str {
    match *state.read().unwrap() {
        AppState::Start => "start",
        AppState::Journal { .. } => "journal",
    }
}

#[tauri::command]
pub fn get_journal_path(state: WindowState<RwLock<AppState>>) -> Option<String> {
    state.read().unwrap().journal_path().map(String::from)
}

#[tauri::command]
pub fn open_journal(app: AppHandle, state: WindowState<RwLock<AppState>>, path: &str) {
    state.write().unwrap().open_journal(path).unwrap();

    let store = app.store(SETTINGS_PATH).unwrap();
    let mut paths = store
        .get(SETTING_OPEN_JOURNALS)
        .and_then(|paths| paths.as_array().cloned())
        .unwrap_or_default();

    paths.push(path.into());
    store.set(SETTING_OPEN_JOURNALS, paths);
}

#[tauri::command]
pub fn close_journal(app: AppHandle, state: WindowState<RwLock<AppState>>) {
    let mut state = state.write().unwrap();
    let Some(path) = state.journal_path().map(|p| p.to_owned()) else {
        return;
    };
    *state = AppState::Start;

    let store = app.store(SETTINGS_PATH).unwrap();
    if let Some(mut paths) = store
        .get(SETTING_OPEN_JOURNALS)
        .and_then(|paths| paths.as_array().cloned())
    {
        if let Some(i) = paths.iter().position(|p| p == &path) {
            paths.remove(i);
        }
        store.set(SETTING_OPEN_JOURNALS, paths);
    }
}

#[tauri::command]
pub fn save_journal(state: WindowState<RwLock<AppState>>) {
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
pub fn get_views(state: WindowState<RwLock<AppState>>) -> ipc::Response {
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
pub fn create_view(state: WindowState<RwLock<AppState>>) -> ViewId {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().create()
}

#[tauri::command]
pub fn remove_view(state: WindowState<RwLock<AppState>>, id: ViewId) {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().remove(id);
}

#[tauri::command]
pub fn set_view_name(state: WindowState<RwLock<AppState>>, id: ViewId, name: String) {
    let mut state = state.write().unwrap();
    state.journal_mut().views_mut().set_name(id, name);
}

#[tauri::command]
pub fn set_view_definition(state: WindowState<RwLock<AppState>>, id: ViewId, definition: String) {
    let mut state = state.write().unwrap();
    state.journal_mut().set_view_definition(id, definition);
}

#[tauri::command]
pub fn get_entries(state: WindowState<RwLock<AppState>>, view: Option<ViewId>) -> ipc::Response {
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
pub fn create_entry(state: WindowState<RwLock<AppState>>) -> EntryId {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().create()
}

#[tauri::command]
pub fn remove_entry(state: WindowState<RwLock<AppState>>, id: EntryId) {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().remove(id);
}

#[tauri::command]
pub fn set_entry_content(
    state: WindowState<RwLock<AppState>>,
    id: EntryId,
    content: String,
) -> ipc::Response {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().set_content(id, content);

    let entries = state.journal().entries();
    ipc::Response::new(serde_json::to_string(entries.get(id)).unwrap())
}

/// A faster endpoint which allows parsing spans after every keystroke. Unlike
/// [`set_entry_content`] this does not instantiate any Prolog terms and so
/// should be faster and safe to call very frequently.
#[tauri::command]
pub fn parse_entry_content(content: &str) -> Vec<Span> {
    lang::parse_spans(content)
}
