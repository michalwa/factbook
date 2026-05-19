use crate::model::{Entry, EntryId, View, ViewId};
use factbook_swipl::{Context, RawFunctor, Record};
use sparse_tags::{IndexedStore, Store};
use stable_vec::StableVec;
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

pub mod lang;
pub mod model;
mod search;

const SWIPL_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

pub struct Session(factbook_swipl::Session<'static>);

impl Session {
    pub fn new() -> Option<Self> {
        factbook_swipl::Session::init(SWIPL_STATE).map(Self)
    }
}

pub struct State<'s> {
    views: RwLock<ViewStorage>,
    entries: RwLock<EntryStorage>,
    session: &'s Session,
}

type ViewStorage = StableVec<View>;
type EntryStorage = IndexedStore<RawFunctor, Record, Entry>;

impl<'a> State<'a> {
    pub fn new(session: &'a Session) -> Self {
        Self {
            views: Default::default(),
            entries: Default::default(),
            session,
        }
    }

    pub fn views(&self) -> Views<'_> {
        Views(self.views.read().unwrap())
    }

    pub fn views_mut(&self) -> ViewsMut<'_> {
        ViewsMut(self.views.write().unwrap())
    }

    pub fn entries(&self) -> Entries<'_> {
        Entries(self.entries.read().unwrap())
    }

    pub fn entries_mut(&self) -> EntriesMut<'_> {
        EntriesMut {
            store: self.entries.write().unwrap(),
            session: self.session,
        }
    }
}

pub struct Views<'a>(RwLockReadGuard<'a, ViewStorage>);

impl<'a> Views<'a> {
    pub fn iter(&'a self) -> impl Iterator<Item = (ViewId, &'a View)> {
        self.0.iter().map(|(id, view)| (ViewId(id), view))
    }
}

pub struct ViewsMut<'a>(RwLockWriteGuard<'a, ViewStorage>);

impl ViewsMut<'_> {
    pub fn create(&mut self) -> ViewId {
        ViewId(self.0.push(Default::default()))
    }

    pub fn remove(&mut self, id: ViewId) -> View {
        self.0.remove(id.0).unwrap()
    }

    pub fn set_name(&mut self, id: ViewId, name: String) {
        self.0[id.0].name = name;
    }

    pub fn set_definition(&mut self, id: ViewId, definition: String) {
        self.0[id.0].definition = definition;

        // TODO: Recompute `entry_count`
    }
}

pub struct Entries<'a>(RwLockReadGuard<'a, EntryStorage>);

impl<'a> Entries<'a> {
    pub fn iter(&'a self) -> impl Iterator<Item = (EntryId, &'a Entry)> {
        self.0.entries().map(|(id, entry)| (EntryId(id), entry))
    }
}

pub struct EntriesMut<'a> {
    session: &'a Session,
    store: RwLockWriteGuard<'a, EntryStorage>,
}

impl<'a> EntriesMut<'a> {
    pub fn create(&mut self) -> EntryId {
        EntryId(self.store.insert_entry(Default::default()))
    }

    pub fn remove(&mut self, id: EntryId) -> Entry {
        self.store.remove_entry(id.0)
    }

    pub fn set_content(&mut self, id: EntryId, content: String) {
        self.store.clear_entry(id.0);

        let mut engine = self.session.0.engine();
        let pl = engine.frame();

        for tag in lang::parse(&content, &pl) {
            let key = tag.get::<RawFunctor>().unwrap();
            self.store.insert_tag(id.0, key, tag.record());
        }

        // let debug_tags = self
        //     .store
        //     .tags_by_entry(id.0)
        //     .map(|(k, _)| k)
        //     .collect::<Vec<_>>();
        // log::debug!("set entry {id:?} to {content:?}, parsed tags: {debug_tags:?}");

        self.store.entry_data_mut(id.0).content = content;
    }
}

#[cfg(test)]
mod test {
    use crate::Session;
    use std::sync::LazyLock;

    pub(crate) static SESSION: LazyLock<Session> = LazyLock::new(|| Session::new().unwrap());
}
