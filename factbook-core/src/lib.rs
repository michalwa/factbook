use crate::model::{Entry, EntryId, View, ViewId};
use factbook_swipl::{Context, EngineHandle};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod lang;
pub mod model;
pub mod search;

const SWIPL_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

pub struct State {
    database: RwLock<Database>,
    cache: RwLock<Cache>,
    swipl_session: factbook_swipl::Session<'static>,
}

impl State {
    pub fn init_with(database: Database) -> Self {
        let swipl_session = factbook_swipl::Session::init(SWIPL_STATE).unwrap();

        let mut pl = swipl_session.engine();
        pl.register_predicate::<crate::search::predicates::EntryTag>();

        let cache = Cache::init_from(&database, &mut pl);

        Self {
            database: RwLock::new(database),
            cache: RwLock::new(cache),
            swipl_session,
        }
    }

    pub fn database<'a>(&'a self) -> RwLockReadGuard<'a, Database> {
        self.database.read().unwrap()
    }

    pub fn database_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Database> {
        self.database.write().unwrap()
    }

    pub fn cache<'a>(&'a self) -> RwLockReadGuard<'a, Cache> {
        self.cache.read().unwrap()
    }

    pub fn cache_mut<'a>(&'a self) -> RwLockWriteGuard<'a, Cache> {
        self.cache.write().unwrap()
    }

    pub fn pl_engine(&self) -> EngineHandle {
        self.swipl_session.engine()
    }
}

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
    pub fn init_from(database: &Database, pl: &mut impl factbook_swipl::Context) -> Self {
        let mut cache = Self {
            entry_tags: Default::default(),
            next_entry_id: database
                .entries
                .iter()
                .map(|(&id, _)| id)
                .max()
                .map(EntryId::next)
                .unwrap_or_default(),
            next_view_id: database
                .views
                .iter()
                .map(|(&id, _)| id)
                .max()
                .map(ViewId::next)
                .unwrap_or_default(),
        };

        for (id, entry) in &database.entries {
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
        tags.extend(crate::lang::parse(content, pl).map(|t| t.record()));
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

#[cfg(test)]
mod test {
    use factbook_swipl::Session;
    use std::sync::LazyLock;

    pub(crate) static SESSION: LazyLock<Session<'static>> =
        LazyLock::new(|| Session::init(crate::SWIPL_STATE).unwrap());
}
