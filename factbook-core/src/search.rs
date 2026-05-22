use crate::model::{Entry, EntryId, ViewId};
use crate::{EntryStorage, State};
use factbook_swipl::Context;
use factbook_swipl::blob::{CopyBlob, ScopedBlob, ScopedBlobData};
use factbook_swipl::query::open_query;
use sparse_tags::Store;
use std::collections::BTreeSet;

impl State<'_> {
    pub fn for_each_view_entry<F>(&self, view: ViewId, mut f: F)
    where
        F: FnMut(EntryId, &Entry),
    {
        let mut engine = self.session.0.engine();
        let pl = engine.frame();
        pl.register_predicate::<predicates::EntryTag>();

        let views = self.views.read().unwrap();
        let view_definition = &views[view.0].definition;

        let view_term = match pl.new_term().put_parsed(view_definition) {
            Ok(view_term) => view_term,
            Err(ex) => {
                log::warn!("failed to parse view: {ex:?}");
                return;
            },
        };

        let entries = self.entries.read().unwrap();
        let mut ctx = ViewContext { entries: &entries };
        let ctx_blob = ScopedBlob::new(&mut ctx);

        let query = open_query! { pl => view_entry({&ctx_blob}, {view_term}, _) }.unwrap();
        let mut visited = BTreeSet::new();

        while let Some([_, _, entry_id]) = query.next_solution().unwrap() {
            if let Some(CopyBlob(entry_id)) = entry_id.get::<CopyBlob<EntryId>>() {
                if visited.contains(&entry_id) {
                    continue;
                }

                let entry = entries.entry_data(entry_id.0);

                f(entry_id, entry);

                visited.insert(entry_id);
            } else {
                // If the entry ID is uninstantiated, the user probably used a wildcard
                // view like `any`, and we should iterate all entries
                log::debug!("query returned uninstantiated entry ID, returning all entries");

                for (entry_id, entry) in entries.entries() {
                    f(EntryId(entry_id), entry);
                }

                // Ignore other solutions
                break;
            }
        }

        query.close();
    }
}

/// The scoped blob passed to the Prolog execution for predicates to be able to
/// query the state
#[derive(ScopedBlobData)]
struct ViewContext<'a> {
    entries: &'a EntryStorage,
}

mod predicates {
    use crate::model::EntryId;
    use crate::search::ViewContext;
    use factbook_swipl::foreign::{Nondet, predicate};
    use factbook_swipl::{Atom, Context, RawFunctor, Record};
    use sparse_tags::Store;

    #[predicate(entry_tag(ctx, entry_id, tag) nondet)]
    pub(super) struct EntryTag<'a> {
        iter: Option<Box<dyn Iterator<Item = (EntryId, &'a Record)> + 'a>>,
    }

    impl Nondet for EntryTag<'_> {
        fn init(_: &impl Context) -> Self {
            Self { iter: None }
        }

        fn next(
            &mut self,
            pl: &mut impl Context,
            [ctx_arg, entry_id, tag]: Self::Args<'_>,
        ) -> bool {
            if self.iter.is_none() {
                let Some(ctx_atom) = ctx_arg.get::<Atom>() else {
                    return false;
                };
                let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                    return false;
                };

                self.iter = if let Some(entry_id) = entry_id.get::<EntryId>() {
                    // Prioritize scanning the entry if the ID is instantiated,
                    // since there's likely less tags on a specific entry than
                    // tags with the same functor
                    log::debug!("entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query by entry_id");

                    Some(Box::new(
                        ctx.entries
                            .tags_by_entry(entry_id.0)
                            .map(move |(_, tag)| (entry_id, tag)),
                    ))
                } else if let Some(functor) = tag.get::<RawFunctor>() {
                    log::debug!("entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query by functor");

                    Some(Box::new(
                        ctx.entries
                            .tags_by_key(&functor)
                            .map(move |(entry_id, tag)| (EntryId(entry_id), tag)),
                    ))
                } else {
                    log::debug!("entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query all");

                    Some(Box::new(
                        ctx.entries
                            .tags()
                            .map(|(entry_id, _, tag)| (EntryId(entry_id), tag)),
                    ))
                };
            }

            for (found_entry_id, found_tag) in self.iter.as_mut().unwrap() {
                let pl = pl.frame();
                let found_tag = pl.new_term().put(found_tag);

                // If `entry_id` is already instantiated, manually compare instead of
                // unifying terms, because unifying blobs like this will always fail,
                // even if they have the same contents
                let entry_matched = match entry_id.get::<EntryId>() {
                    Some(entry_id) => entry_id == found_entry_id,
                    None => {
                        let found_entry_id = pl.new_term().put(found_entry_id);
                        entry_id.unify_with(found_entry_id)
                    },
                };

                if entry_matched && tag.unify_with(found_tag) {
                    pl.close();
                    return true;
                }
            }

            false
        }
    }
}
