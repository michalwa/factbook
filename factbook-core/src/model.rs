use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewId(pub(crate) u64);

impl ViewId {
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for ViewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntryId(pub(crate) u64);

impl EntryId {
    pub fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

impl factbook_swipl::term::ToTerm for EntryId {
    fn put_in(self, term: factbook_swipl::term::Term) {
        // TODO: Make it an opaque blob
        self.0.put_in(term);
    }
}

impl factbook_swipl::term::FromTerm for EntryId {
    fn from_term(term: factbook_swipl::term::Term) -> Option<Self> {
        term.get().map(Self)
    }
}

#[derive(Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct View {
    pub name: String,
    pub definition: String,
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
