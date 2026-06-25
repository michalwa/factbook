use crate::OpenMode;
use crate::util::SerializeIterOnce;
use crate::window::{self, WindowScopedManager, WindowState, WindowStatesExt};
use factbook_core::lang::{self, Span};
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::sync::RwLock;
use tauri::{AppHandle, Manager, Runtime, Window, ipc};
use tauri_plugin_dialog::{DialogExt, FileDialogBuilder};

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

type AppState<'a> = WindowState<'a, RwLock<crate::AppState>>;

#[tauri::command]
pub fn get_journal_path(state: AppState) -> Option<PathBuf> {
    state.read().unwrap().journal_path.clone()
}

// Commands which create windows need to be async
// https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html#known-issues
#[tauri::command]
pub async fn create_journal(app: AppHandle) {
    window::open(&app, RwLock::new(crate::AppState::default()));
}

#[tauri::command]
pub async fn open_journal(window: Window) {
    if let Some(path) = journal_file_picker(&window)
        .set_title("Open journal file")
        .blocking_pick_file()
    {
        let path = path.into_path().unwrap();
        let state = crate::AppState::open(path, OpenMode::Edit).unwrap();
        window::open(window.app_handle(), RwLock::new(state));
    }
}

#[derive(Serialize)]
pub struct SaveJournalResponse {
    success: bool,
}

#[tauri::command]
pub async fn save_journal(window: Window) -> Result<SaveJournalResponse, ()> {
    let state = window.state_window_scoped::<RwLock<crate::AppState>>();
    let mut state = state.write().unwrap();

    let path = match state.journal_path {
        Some(ref path) => path,
        None => {
            let Some(path) = journal_file_picker(&window)
                .set_title("Save journal file")
                .blocking_save_file()
            else {
                return Ok(SaveJournalResponse { success: false });
            };

            state.default = false;
            state.journal_path = Some(path.into_path().unwrap());
            state.journal_path.as_ref().unwrap()
        },
    };

    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .unwrap();

    serde_json::to_writer_pretty(file, &state.journal.to_journal()).unwrap();

    Ok(SaveJournalResponse { success: true })
}

#[tauri::command]
pub async fn open_default_journal(app: AppHandle) {
    let windows = app.window_states::<RwLock<crate::AppState>>();

    if let Some((label, _)) = windows
        .states()
        .iter()
        .find(|(_, state)| state.read().unwrap().default)
    {
        app.get_webview_window(label)
            .and_then(|w| w.set_focus().ok());
    } else {
        window::open(
            &app,
            RwLock::new(crate::AppState::open_default(&app).unwrap()),
        );
    }
}

#[tauri::command]
pub fn get_views(state: AppState) -> ipc::Response {
    let state = state.read().unwrap();
    let views = state.journal.views();
    let views = views
        .iter()
        .map(|(id, view)| View { id, view })
        .collect::<Vec<_>>();

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(views)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn create_view(state: AppState) -> ViewId {
    let state = state.write().unwrap();
    state.journal.views_mut().create()
}

#[tauri::command]
pub fn remove_view(state: AppState, id: ViewId) {
    let state = state.write().unwrap();
    state.journal.views_mut().remove(id);
}

#[tauri::command]
pub fn set_view_name(state: AppState, id: ViewId, name: String) {
    let state = state.write().unwrap();
    state.journal.views_mut().set_name(id, name);
}

#[tauri::command]
pub fn set_view_definition(state: AppState, id: ViewId, definition: String) {
    let state = state.write().unwrap();
    state.journal.set_view_definition(id, definition);
}

#[tauri::command]
pub fn get_entries(state: AppState, view: Option<ViewId>) -> ipc::Response {
    use serde::Serializer;
    use serde::ser::SerializeSeq;

    let state = state.read().unwrap();

    let mut json = Vec::new();
    let mut serializer = serde_json::Serializer::new(&mut json);
    let mut seq = serializer.serialize_seq(None).unwrap();

    if let Some(view) = view {
        state.journal.for_each_view_entry(view, |id, entry| {
            seq.serialize_element(&Entry { id, entry }).unwrap()
        });
    } else {
        state
            .journal
            .entries()
            .iter()
            .for_each(|(id, entry)| seq.serialize_element(&Entry { id, entry }).unwrap());
    }

    seq.end().unwrap();

    // SAFETY: `serde_json::Serializer` does not emit invalid UTF-8
    ipc::Response::new(unsafe { String::from_utf8_unchecked(json) })
}

#[tauri::command]
pub fn create_entry(state: AppState) -> EntryId {
    let state = state.write().unwrap();
    state.journal.entries_mut().create()
}

#[tauri::command]
pub fn remove_entry(state: AppState, id: EntryId) {
    let state = state.write().unwrap();
    state.journal.entries_mut().remove(id);
}

#[tauri::command]
pub fn set_entry_content(state: AppState, id: EntryId, content: String) -> ipc::Response {
    let state = state.write().unwrap();
    state.journal.entries_mut().set_content(id, content);

    let entries = state.journal.entries();
    ipc::Response::new(serde_json::to_string(&entries.get(id)).unwrap())
}

/// A faster endpoint which allows parsing spans after every keystroke. Unlike
/// [`set_entry_content`] this does not instantiate any Prolog terms and so
/// should be faster and safe to call very frequently.
#[tauri::command]
pub fn parse_entry_content(content: &str) -> Vec<Span> {
    lang::parse_spans(content)
}

#[tauri::command]
pub fn get_tags(state: AppState) -> ipc::Response {
    let state = state.read().unwrap();
    let entries = state.journal.entries();

    let counts = entries.common_tag_counts();
    let response = serde_json::to_string(&SerializeIterOnce::new(counts)).unwrap();
    ipc::Response::new(response)
}

fn journal_file_picker<R: Runtime>(window: &Window<R>) -> FileDialogBuilder<R> {
    window
        .app_handle()
        .dialog()
        .file()
        .set_parent(window)
        .add_filter("Journal file", &["json"])
}
