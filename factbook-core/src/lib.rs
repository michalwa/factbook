use crate::model::{
    CommonTag, CommonTagCount, Entry, EntryId, Journal, PersistedEntry, PersistedView, View, ViewId,
};
use crate::search::TagKey;
use factbook_swipl::{Context, Record};
use sparse_tags::{IndexedStore, Store};
use std::collections::{BTreeMap, HashMap};

pub mod lang;
pub mod model;
mod search;

const SWIPL_STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

pub struct Session(factbook_swipl::Session<'static>);

impl Session {
    pub fn new() -> Option<Self> {
        factbook_swipl::Session::init(SWIPL_STATE)
            .map(crate::search::predicates::register)
            .map(Self)
    }
}

pub struct State<'s> {
    views: BTreeMap<ViewId, View>,
    entries: EntryStorage,
    last_view_id: ViewId,
    session: &'s Session,
}

pub(crate) type EntryStorage = IndexedStore<TagKey, Record, Entry>;

impl<'s> State<'s> {
    pub fn new(session: &'s Session) -> Self {
        Self {
            views: Default::default(),
            entries: Default::default(),
            last_view_id: ViewId(0),
            session,
        }
    }

    pub fn views(&self) -> impl Iterator<Item = (ViewId, &View)> {
        self.views.iter().map(|(id, view)| (*id, view))
    }

    pub fn create_view(&mut self) -> ViewId {
        self.last_view_id.0 += 1;
        self.views.insert(self.last_view_id, Default::default());
        self.last_view_id
    }

    pub fn remove_view(&mut self, id: ViewId) -> Option<View> {
        self.views.remove(&id)
    }

    pub fn set_view_name(&mut self, id: ViewId, name: String) {
        self.views.get_mut(&id).unwrap().name = name;
    }

    pub fn set_view_definition(&mut self, id: ViewId, definition: String) {
        self.views.get_mut(&id).unwrap().definition = definition;
        self.update_view(id);
    }

    pub fn entries(&self) -> impl Iterator<Item = (EntryId, &Entry)> {
        self.entries
            .entries()
            .map(|(id, entry)| (EntryId(id), entry))
    }

    pub fn entry(&self, id: EntryId) -> Option<&Entry> {
        self.entries
            .entry_exists(id.0)
            .then(|| self.entries.entry_data(id.0))
    }

    pub fn create_entry(&mut self) -> EntryId {
        EntryId(self.entries.insert_entry(Default::default()))
    }

    pub fn remove_entry(&mut self, id: EntryId) -> Option<Entry> {
        self.entries
            .entry_exists(id.0)
            .then(|| self.entries.remove_entry(id.0))
    }

    pub fn set_entry_content(&mut self, id: EntryId, content: String) {
        if self.entries.entry_exists(id.0) {
            self.entries.entry_data_mut(id.0).content = content;
            self.update_entry(id);
        }
    }

    pub fn common_tags(&self) -> impl Iterator<Item = CommonTag<'_>> {
        self.entries
            .tags()
            .filter_map(move |(_, key, _)| CommonTag::from_key(key))
    }

    pub fn common_tag_counts(&self) -> impl Iterator<Item = CommonTagCount<'_>> {
        let mut counts = HashMap::new();

        for tag in self.common_tags() {
            *counts.entry(tag).or_default() += 1;
        }

        counts
            .into_iter()
            .map(|(tag, count)| CommonTagCount { tag, count })
    }

    pub fn load_journal(&mut self, journal: Journal) {
        for entry in journal.entries {
            let id = self.entries.insert_entry(entry.into());
            self.update_entry(EntryId(id));
        }

        for view in journal.views {
            match view.id {
                Some(id) => {
                    self.views.insert(id, view.into());
                    self.last_view_id = self.last_view_id.max(id);
                },
                None => {
                    self.last_view_id.0 += 1;
                    self.views.insert(self.last_view_id, view.into());
                },
            }

            self.update_view(self.last_view_id);
        }
    }

    pub fn to_journal(&self) -> Journal {
        let entries = self
            .entries
            .entries()
            .map(|(_, e)| PersistedEntry::new(e))
            .collect::<Vec<_>>();

        let views = self
            .views
            .iter()
            .map(|(id, view)| PersistedView::new(*id, view))
            .collect::<Vec<_>>();

        Journal { entries, views }
    }

    fn update_view(&mut self, id: ViewId) {
        let mut entry_count = 0;
        self.for_each_view_entry(id, |_, _| entry_count += 1);
        self.views.get_mut(&id).unwrap().entry_count = entry_count;
    }

    fn update_entry(&mut self, id: EntryId) {
        self.entries.clear_entry(id.0);

        let content = &self.entries.entry_data(id.0).content;

        let mut engine = self.session.0.engine();
        let pl = engine.frame();

        let parsed = lang::parse(content, Some(&pl));

        for tag in parsed.tags {
            let key = TagKey::from(&tag);
            self.entries.insert_tag(id.0, key, tag.record());
        }

        self.entries.entry_data_mut(id.0).spans =
            (!parsed.spans.is_empty()).then_some(parsed.spans);
    }
}

#[cfg(test)]
mod test {
    use crate::model::{CommonTag, EntryId, ViewId};
    use crate::{Session, State};
    use chrono::{Local, TimeZone, Timelike};
    use factbook_swipl::{Context, term};
    use pretty_assertions::assert_eq;
    use sparse_tags::Store;
    use std::collections::HashSet;
    use std::sync::LazyLock;
    use test_log::test;

    pub(crate) static SESSION: LazyLock<Session> = LazyLock::new(|| Session::new().unwrap());

    fn generate_fixtures() -> (State<'static>, Vec<EntryId>) {
        let mut state = State::new(&SESSION);

        let entry_ids = {
            [
                "",
                "@foo",
                "@foo(1, 2)",
                "@bar some other text",
                "@foo @bar",
                "@foo(1, 1)",
                "@bar(3)",
                "@bar(1)",
                "@bar(2)",
                "@baz(1, 2)",
                "@baz(2, 1)",
                "@42",
                r#"@"string""#,
            ]
            .into_iter()
            .map(|content| {
                let entry = state.create_entry();
                state.set_entry_content(entry, content.into());
                entry
            })
            .collect()
        };

        (state, entry_ids)
    }

    fn create_view(state: &mut State, definition: impl Into<String>) -> ViewId {
        let view = state.create_view();
        state.set_view_definition(view, definition.into());
        view
    }

    #[test]
    fn prolog_tests() {
        let pl = SESSION.0.engine();
        let goal = term! { &pl => run_tests(_, [format(log)]) };
        assert!(pl.call(goal, None).unwrap());
    }

    #[test]
    fn view_empty() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |id, _| matches.push(id));
        assert_eq!(matches, []);
    }

    #[test]
    fn view_any() {
        let (mut state, entry_ids) = generate_fixtures();
        let view = create_view(&mut state, "_");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |id, _| matches.push(id));
        assert_eq!(matches, entry_ids);
    }

    #[test]
    fn view_single_tag() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@foo");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo", "@foo @bar"]);
    }

    #[test]
    fn view_single_tag_with_wildcard_args() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@foo(_, _)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo(1, 2)", "@foo(1, 1)"]);
    }

    #[test]
    fn view_conjunction() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@foo, @bar");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo @bar"]);
    }

    #[test]
    fn view_disjunction() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@foo; @bar");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        matches.sort();
        assert_eq!(matches, ["@bar some other text", "@foo", "@foo @bar"]);
    }

    #[test]
    fn view_single_tag_with_unified_args() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@foo(X, X)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@foo(1, 1)"]);
    }

    #[test]
    fn view_existence() {
        let (mut state, _) = generate_fixtures();
        // @bar(X), such that there exists a an entry with @foo(X, _)
        let view = create_view(&mut state, "@bar(X), _: @foo(X, _)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@bar(1)"]);
    }

    #[test]
    fn view_existence_mapping() {
        let (mut state, _) = generate_fixtures();
        // @bar(X), such that there exists a an entry with @baz(1, X)
        let view = create_view(&mut state, "@bar(X), _: @baz(1, X)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@bar(2)"]);
    }

    #[test]
    fn view_non_functor_tags() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, r#"@42; @"string""#);

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@42", r#"@"string""#]);
    }

    #[test]
    fn view_order() {
        let (mut state, _) = generate_fixtures();
        let view = create_view(&mut state, "@bar(X), order(X)");

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |_, e| matches.push(e.content.clone()));
        assert_eq!(matches, ["@bar(1)", "@bar(2)", "@bar(3)"]);
    }

    #[test]
    fn view_created() {
        let created_at = Local
            .with_ymd_and_hms(2021, 2, 3, 13, 14, 15)
            .unwrap()
            .with_nanosecond(123_000_000)
            .unwrap();

        let (mut state, entry_ids) = generate_fixtures();

        state.entries.entry_data_mut(entry_ids[0].0).created_at = created_at;

        let view = create_view(
            &mut state,
            "
            created(T),
            {
                stamp_date_time(T, date(2021, 2, 3, 13, 14, S, _, _, _), local),
                abs(S - 15.123) < 0.0001 % expected deviation in sub-second precision
            }
            ",
        );

        let mut matches = Vec::new();
        state.for_each_view_entry(view, |id, _| matches.push(id));
        assert_eq!(matches, [entry_ids[0]]);
    }

    #[test]
    fn get_tags() {
        use crate::model::CommonTagKind as T;

        let (state, _) = generate_fixtures();

        let mut tags = state
            .common_tags()
            .collect::<HashSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        tags.sort();

        assert_eq!(tags, [
            CommonTag {
                name: "bar".into(),
                kind: T::Atom,
            },
            CommonTag {
                name: "bar".into(),
                kind: T::Functor { arity: 1 },
            },
            CommonTag {
                name: "baz".into(),
                kind: T::Functor { arity: 2 },
            },
            CommonTag {
                name: "foo".into(),
                kind: T::Atom,
            },
            CommonTag {
                name: "foo".into(),
                kind: T::Functor { arity: 2 },
            },
            CommonTag {
                name: "string".into(),
                kind: T::String,
            },
        ]);
    }
}
