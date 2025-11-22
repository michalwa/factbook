use crate::prolog;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub views: BTreeMap<ViewId, View>,
    pub entries: BTreeMap<EntryId, Entry>,
}

pub struct Cache {
    pub entry_tags: BTreeMap<EntryId, Vec<factbook_swipl::Record>>,
}

impl Cache {
    pub fn init_from(state: &Database, pl: &mut impl factbook_swipl::Context) -> Self {
        let pl = pl.frame();
        let entry_tags = state
            .entries
            .iter()
            .map(|(k, v)| {
                (
                    *k,
                    prolog::parse(&v.content, &pl).map(|t| t.record()).collect(),
                )
            })
            .collect();

        Self { entry_tags }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewId(u64);

impl fmt::Display for ViewId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EntryId(u64);

#[derive(Clone, Serialize, Deserialize)]
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
