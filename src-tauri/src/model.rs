use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

/// Holds persistent data that is saved across sessions
#[derive(Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Database {
    pub views: BTreeMap<ViewId, View>,
    pub entries: BTreeMap<EntryId, Entry>,
}

/// Holds non-persistent data for the duration of the current session
pub struct Cache {
    pub entry_tags: BTreeMap<EntryId, Vec<factbook_swipl::Record>>,
    next_entry_id: EntryId,
    next_view_id: ViewId,
}

impl Cache {
    pub fn init_from(state: &Database, pl: &mut impl factbook_swipl::Context) -> Self {
        let mut cache = Self {
            entry_tags: Default::default(),
            next_entry_id: state
                .entries
                .iter()
                .map(|(&id, _)| id)
                .max()
                .map(EntryId::next)
                .unwrap_or_default(),
            next_view_id: state
                .views
                .iter()
                .map(|(&id, _)| id)
                .max()
                .map(ViewId::next)
                .unwrap_or_default(),
        };

        for (id, entry) in &state.entries {
            cache.update_entry(pl, *id, &entry.content);
        }

        cache
    }

    pub fn update_entry(
        &mut self,
        pl: &mut impl factbook_swipl::Context,
        id: EntryId,
        content: &str,
    ) {
        let tags = self.entry_tags.entry(id).or_default();
        tags.clear();
        tags.extend(crate::prolog::parse(content, pl).map(|t| t.record()));
    }

    pub fn next_entry_id(&mut self) -> EntryId {
        let id = self.next_entry_id;
        self.next_entry_id = self.next_entry_id.next();
        id
    }

    pub fn next_view_id(&mut self) -> ViewId {
        let id = self.next_view_id;
        self.next_view_id = self.next_view_id.next();
        id
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ViewId(u64);

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
pub struct EntryId(u64);

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

impl Entry {
    pub fn new() -> Self {
        Self {
            created_at: Local::now(),
            content: String::new(),
        }
    }
}
