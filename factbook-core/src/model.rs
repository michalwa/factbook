use crate::lang::Span;
use chrono::{DateTime, Local};
use factbook_swipl::blob::{CopyBlob, CopyBlobData};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewId(pub(crate) usize);

impl fmt::Display for ViewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, CopyBlobData)]
#[serde(transparent)]
pub struct EntryId(pub(crate) sparse_tags::EntryId);

impl fmt::Debug for EntryId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl factbook_swipl::term::ToTerm for EntryId {
    fn put_in(self, term: factbook_swipl::term::Term) {
        term.put(CopyBlob(self));
    }
}

impl factbook_swipl::term::FromTerm for EntryId {
    fn from_term(term: factbook_swipl::term::Term) -> Option<Self> {
        Some(term.get::<CopyBlob<Self>>()?.0)
    }
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub name: String,
    pub definition: String,
    pub entry_count: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub created_at: DateTime<Local>,
    pub content: String,
    pub spans: Option<Vec<Span>>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            created_at: Local::now(),
            content: String::new(),
            spans: None,
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct PersistedView {
    pub name: String,
    pub definition: String,
}

impl From<&View> for PersistedView {
    fn from(value: &View) -> Self {
        let View {
            name, definition, ..
        } = value;

        Self {
            name: name.clone(),
            definition: definition.clone(),
        }
    }
}

impl From<PersistedView> for View {
    fn from(value: PersistedView) -> Self {
        let PersistedView { name, definition } = value;

        Self {
            name,
            definition,
            ..Default::default()
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(crate) struct PersistedEntry {
    pub created_at: DateTime<Local>,
    pub content: String,
}

impl From<&Entry> for PersistedEntry {
    fn from(value: &Entry) -> Self {
        let Entry {
            created_at,
            content,
            ..
        } = value;

        Self {
            created_at: *created_at,
            content: content.clone(),
        }
    }
}

impl From<PersistedEntry> for Entry {
    fn from(value: PersistedEntry) -> Self {
        let PersistedEntry {
            created_at,
            content,
        } = value;

        Self {
            created_at,
            content,
            ..Default::default()
        }
    }
}

/// Persistent save file
#[derive(Serialize, Deserialize)]
pub struct Journal {
    pub(crate) views: Vec<PersistedView>,
    pub(crate) entries: Vec<PersistedEntry>,
}
