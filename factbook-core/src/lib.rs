use crate::model::{Entry, EntryId, Journal, View, ViewId};
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
        Some(Self(
            factbook_swipl::Session::init(SWIPL_STATE)?
                .register_predicate::<search::predicates::EntryTag>(),
        ))
    }
}

pub struct State<'s> {
    // FIXME: Due to the interior `RwLock`s, you can currently introduce a
    // deadlock by borrowing something mutably from the state and then calling
    // a method which requires another lock
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

    pub fn set_view_definition(&self, id: ViewId, definition: String) {
        let mut views = self.views.write().unwrap();
        views[id.0].definition = definition;
        drop(views); // can't hold `views` while calling `for_each_view_entry`

        self.update_view(id);
    }

    pub fn load_journal(&self, journal: Journal) {
        for entry in journal.entries {
            self.entries_mut().insert(entry);
        }

        for view in journal.views {
            self.insert_view(view);
        }
    }

    fn insert_view(&self, view: View) {
        let mut views = self.views_mut();
        let id = ViewId(views.0.push(view));
        drop(views);

        self.update_view(id);
    }

    fn update_view(&self, id: ViewId) {
        let mut entry_count = 0;
        self.for_each_view_entry(id, |_, _| entry_count += 1);

        let mut views = self.views.write().unwrap();
        views[id.0].entry_count = entry_count;
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
        self.store.entry_data_mut(id.0).content = content;
        self.update_entry(id);
    }

    fn insert(&mut self, entry: Entry) {
        let id = EntryId(self.store.insert_entry(entry));
        self.update_entry(id);
    }

    fn update_entry(&mut self, id: EntryId) {
        self.store.clear_entry(id.0);

        let content = &self.store.entry_data(id.0).content;

        let mut engine = self.session.0.engine();
        let pl = engine.frame();

        for tag in lang::parse(content, &pl).collect::<Vec<_>>() {
            let key = tag.get::<RawFunctor>().unwrap();
            self.store.insert_tag(id.0, key, tag.record());
        }
    }
}

#[cfg(test)]
mod test {
    use crate::model::{EntryId, ViewId};
    use crate::{Session, State};
    use std::sync::LazyLock;
    use test_log::test;

    pub(crate) static SESSION: LazyLock<Session> = LazyLock::new(|| Session::new().unwrap());

    static FIXTURES: LazyLock<(State<'static>, Vec<EntryId>)> = LazyLock::new(generate_fixtures);

    fn generate_fixtures() -> (State<'static>, Vec<EntryId>) {
        let state = State::new(&SESSION);

        let entry_ids = {
            let mut entries = state.entries_mut();

            [
                "",
                "@foo",
                "@foo(1, 2)",
                "@bar some other text",
                "@foo @bar",
                "@foo(1, 1)",
                "@bar(1)",
                "@bar(2)",
                "@baz(1, 2)",
                "@baz(2, 1)",
            ]
            .into_iter()
            .map(|content| {
                let entry = entries.create();
                entries.set_content(entry, content.into());
                entry
            })
            .collect()
        };

        (state, entry_ids)
    }

    fn create_view(state: &State, definition: impl Into<String>) -> ViewId {
        let view = state.views_mut().create();
        state.set_view_definition(view, definition.into());
        view
    }

    #[test]
    fn view_empty() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |id, _| matches.push(id));
        assert_eq!(matches, []);
    }

    #[test]
    fn view_any() {
        let (state, entry_ids) = &*FIXTURES;
        let view = create_view(state, "any");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |id, _| matches.push(id));
        assert_eq!(&matches, entry_ids);
    }

    #[test]
    fn view_single_tag() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "@foo");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo", "@foo @bar"]);
    }

    #[test]
    fn view_single_tag_with_wildcard_args() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "@foo(_, _)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo(1, 2)", "@foo(1, 1)"]);
    }

    #[test]
    fn view_conjunction() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "@foo, @bar");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo @bar"]);
    }

    #[test]
    fn view_disjunction() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "@foo; @bar");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        matches.sort();
        assert_eq!(matches, ["@bar some other text", "@foo", "@foo @bar"]);
    }

    #[test]
    fn view_single_tag_with_unified_args() {
        let (state, _) = &*FIXTURES;
        let view = create_view(state, "@foo(X, X)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo(1, 1)"]);
    }

    #[test]
    fn view_existence() {
        let (state, _) = &*FIXTURES;
        // @bar(X), such that there exists a an entry with @foo(X, _)
        let view = create_view(state, "@bar(X), _: @foo(X, _)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@bar(1)"]);
    }

    #[test]
    fn view_existence_mapping() {
        let (state, _) = &*FIXTURES;
        // @bar(X), such that there exists a an entry with @baz(1, X)
        let view = create_view(state, "@bar(X), _: @baz(1, X)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@bar(2)"]);
    }
}
