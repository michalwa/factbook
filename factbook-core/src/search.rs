use crate::model::{Entry, EntryId, ViewId};
use crate::{Cache, Database};
use factbook_swipl::blob::ScopedBlob;
use factbook_swipl::query::open_query;
use factbook_swipl::{Context, EngineHandle};
use std::collections::BTreeSet;

pub mod predicates {
    use crate::model::EntryId;
    use factbook_swipl::blob::ScopedBlobData;
    use factbook_swipl::foreign::{Nondet, predicate};
    use factbook_swipl::{Atom, Context};
    use std::collections::BTreeMap;

    #[derive(ScopedBlobData)]
    pub struct EntryTagContext<'a>(pub(crate) &'a BTreeMap<EntryId, Vec<factbook_swipl::Record>>);

    #[predicate(entry_tag(context, entry, tag) nondet)]
    pub struct EntryTag<'a> {
        iter: Option<Box<dyn Iterator<Item = (EntryId, &'a factbook_swipl::Record)> + 'a>>,
    }

    impl Nondet for EntryTag<'_> {
        fn init(_: &impl Context) -> Self {
            Self { iter: None }
        }

        fn next(
            &mut self,
            pl: &mut impl Context,
            [entry_tags, arg_entry, arg_tag]: Self::Args<'_>,
        ) -> bool {
            if self.iter.is_none() {
                let Some(entry_tags_atom) = entry_tags.get::<Atom>() else {
                    return false;
                };
                let Some(entry_tags) = entry_tags_atom.scoped_blob::<EntryTagContext>() else {
                    return false;
                };
                self.iter = Some(Box::new(
                    entry_tags
                        .0
                        .iter()
                        .flat_map(|(id, ts)| ts.iter().map(|t| (*id, t))),
                ));
            };

            for (entry_id, tag) in self.iter.as_mut().unwrap() {
                let pl = pl.frame();
                let entry_id = pl.new_term().put(entry_id);
                let tag = pl.new_term().put(tag);

                if arg_entry.unify_with(entry_id) && arg_tag.unify_with(tag) {
                    pl.close();
                    return true;
                }
            }

            false
        }
    }
}

pub fn get_entries<'d>(
    database: &'d Database,
    cache: &'_ Cache,
    pl: &mut EngineHandle,
    view: Option<ViewId>,
) -> Box<dyn Iterator<Item = (EntryId, &'d Entry)> + 'd> {
    let pl = pl.frame();

    let context = predicates::EntryTagContext(&cache.entry_tags);
    let context_blob = ScopedBlob::new(&context);

    if let Some(view_id) = view {
        let mut entry_ids = BTreeSet::new();
        let view = database.views.get(&view_id).unwrap();
        let view_term = pl.new_term().put_parsed(&view.definition).unwrap();

        // `view_entry(Context: EntryTagContext, View, EntryId)` defined in
        // ../prelude.pl
        let query = open_query! { pl => view_entry({&context_blob}, {view_term}, _) }.unwrap();
        while let Some([_, _, entry_id]) = query.next_solution().unwrap() {
            // TODO: impl Iterator for Query
            entry_ids.insert(entry_id.get::<EntryId>().unwrap());
        }

        Box::new(entry_ids.into_iter().map(|id| (id, &database.entries[&id])))
    } else {
        Box::new(database.entries.iter().map(|(id, entry)| (*id, entry)))
    }
}

#[cfg(test)]
mod test {
    use crate::model::{Entry, EntryId, View, ViewId};
    use crate::{Cache, Database};
    use factbook_swipl::Context;
    use std::sync::LazyLock;
    use test_log::test;

    static FIXTURE_DATABASE: LazyLock<Database> = LazyLock::new(|| {
        let mut database = Database::default();

        database.entries.insert(EntryId(0), Entry {
            content: "@foo".into(),
            ..Default::default()
        });
        database.entries.insert(EntryId(1), Entry {
            content: "@bar".into(),
            ..Default::default()
        });
        database.entries.insert(EntryId(2), Entry {
            content: "@foo @bar".into(),
            ..Default::default()
        });

        database.views.insert(ViewId(0), View {
            name: "foo".into(),
            definition: "@foo".into(),
        });
        database.views.insert(ViewId(1), View {
            name: "bar".into(),
            definition: "@bar".into(),
        });
        database.views.insert(ViewId(2), View {
            name: "foo and bar".into(),
            definition: "@foo, @bar".into(),
        });
        database.views.insert(ViewId(3), View {
            name: "foo or bar".into(),
            definition: "@foo; @bar".into(),
        });
        database.views.insert(ViewId(4), View {
            name: "any".into(),
            definition: "any".into(),
        });
        database.views.insert(ViewId(5), View {
            name: "empty".into(),
            definition: "".into(),
        });
        database.views.insert(ViewId(6), View {
            name: "invalid".into(),
            definition: "ajksdhkajshd".into(),
        });

        database
    });

    #[test]
    fn get_entries_all() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, None)
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(0), EntryId(1), EntryId(2)]);
    }

    #[test]
    fn get_entries_single_tag() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(0)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(0), EntryId(2)]);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(1)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(1), EntryId(2)]);
    }

    #[test]
    fn get_entries_conjunction() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(2)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(2)]);
    }

    #[test]
    fn get_entries_disjunction() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(3)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(0), EntryId(1), EntryId(2)]);
    }

    #[test]
    fn get_entries_any() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(4)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, [EntryId(0), EntryId(1), EntryId(2)]);
    }

    #[test]
    fn get_entries_empty() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(5)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, []);
    }

    #[test]
    fn get_entries_invalid() {
        let mut pl = crate::test::SESSION.engine();
        pl.register_predicate::<super::predicates::EntryTag>();
        let cache = Cache::init_from(&FIXTURE_DATABASE, &mut pl);

        let entries = super::get_entries(&FIXTURE_DATABASE, &cache, &mut pl, Some(ViewId(6)))
            .map(|(id, _)| id)
            .collect::<Vec<_>>();
        assert_eq!(entries, []);
    }
}
