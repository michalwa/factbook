use crate::prolog;
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
