use crate::AppState;
use crate::util::SerializeIterOnce;
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
use std::fs::File;
use std::sync::RwLock;
use tauri::{State, ipc};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct View<'t> {
    id: ViewId,
    #[serde(flatten)]
    view: &'t model::View,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Entry<'t> {
    id: EntryId,
    #[serde(flatten)]
    entry: &'t model::Entry,
}

#[tauri::command]
pub fn get_state(state: State<RwLock<AppState>>) -> &'static str {
    match *state.read().unwrap() {
        AppState::Start => "start",
        AppState::Journal(_) => "journal",
    }
}

#[tauri::command]
pub fn open_journal(state: State<RwLock<AppState>>, path: &str) {
    let journal_state = factbook_core::State::new(&crate::SESSION);

    let file = File::open(path).unwrap();
    let journal = serde_json::from_reader(file).unwrap();
    journal_state.load_journal(journal);

    let mut state = state.write().unwrap();
    *state = AppState::Journal(journal_state);
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

    if let Some(view) = view {
        let mut json = Vec::new();
        let mut serializer = serde_json::Serializer::new(&mut json);
        let mut seq = serializer.serialize_seq(None).unwrap();

        state.for_each_view_entry(view, |id, entry| {
            seq.serialize_element(&Entry { id, entry }).unwrap();
        });

        seq.end().unwrap();

        // SAFETY: `serde_json::Serializer` does not emit invalid UTF-8
        ipc::Response::new(unsafe { String::from_utf8_unchecked(json) })
    } else {
        let entries = state.entries();
        let entries = entries
            .iter()
            .map(|(id, entry)| Entry { id, entry })
            .collect::<Vec<_>>();

        // Return an `ipc::Response` directly to avoid allocations
        let response = serde_json::to_string(&SerializeIterOnce::new(entries)).unwrap();
        ipc::Response::new(response)
    }
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
pub fn set_entry_content(state: State<RwLock<AppState>>, id: EntryId, content: String) {
    let mut state = state.write().unwrap();
    state.journal_mut().entries_mut().set_content(id, content);
}
