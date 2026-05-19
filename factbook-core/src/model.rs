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

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, CopyBlobData,
)]
#[serde(transparent)]
pub struct EntryId(pub(crate) sparse_tags::EntryId);

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

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub name: String,
    pub definition: String,
    pub entry_count: usize,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub created_at: DateTime<Local>,
    pub content: String,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            created_at: Local::now(),
            content: String::new(),
        }
    }
}
