use crate::util::SerializeIterOnce;
use factbook_core::model::{self, EntryId, ViewId};
use serde::Serialize;
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
pub fn get_views(state: State<factbook_core::State>) -> ipc::Response {
    let views = state.views();
    let views = views
        .iter()
        .map(|(id, view)| View { id, view })
        .collect::<Vec<_>>();

    // Return an `ipc::Response` directly to avoid allocations
    let response = serde_json::to_string(&SerializeIterOnce::new(views)).unwrap();
    ipc::Response::new(response)
}

#[tauri::command]
pub fn create_view(state: State<factbook_core::State>) -> ViewId {
    state.views_mut().create()
}

#[tauri::command]
pub fn remove_view(state: State<factbook_core::State>, id: ViewId) {
    state.views_mut().remove(id);
}

#[tauri::command]
pub fn set_view_name(state: State<factbook_core::State>, id: ViewId, name: String) {
    state.views_mut().set_name(id, name);
}

#[tauri::command]
pub fn set_view_definition(state: State<factbook_core::State>, id: ViewId, definition: String) {
    state.views_mut().set_definition(id, definition);
}

#[tauri::command]
pub fn get_entries(state: State<factbook_core::State>, view: Option<ViewId>) -> ipc::Response {
    use serde::Serializer;
    use serde::ser::SerializeSeq;

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
pub fn create_entry(state: State<factbook_core::State>) -> EntryId {
    state.entries_mut().create()
}

#[tauri::command]
pub fn remove_entry(state: State<factbook_core::State>, id: EntryId) {
    state.entries_mut().remove(id);
}

#[tauri::command]
pub fn set_entry_content(state: State<factbook_core::State>, id: EntryId, content: String) {
    state.entries_mut().set_content(id, content);
}
