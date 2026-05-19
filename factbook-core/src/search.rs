use crate::model::{Entry, EntryId, ViewId};
use crate::{EntryStorage, State};
use factbook_swipl::Context;
use factbook_swipl::blob::{CopyBlob, ScopedBlob, ScopedBlobData};
use factbook_swipl::query::open_query;
use sparse_tags::Store;

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
        let view_term = pl
            .new_term()
            .put_parsed(view_definition)
            .expect("invalid view");

        let entries = self.entries.read().unwrap();
        let ctx = ViewContext { entries: &entries };
        let ctx_blob = ScopedBlob::new(&ctx);

        let query = open_query! { pl => view_entry({&ctx_blob}, {view_term}, _) }.unwrap();

        while let Some([_, _, entry_id]) = query.next_solution().unwrap() {
            let CopyBlob(entry_id) = entry_id.get::<CopyBlob<EntryId>>().unwrap();
            let entry = entries.entry_data(entry_id.0);

            f(entry_id, entry);
        }
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

        fn next(&mut self, pl: &mut impl Context, [ctx, entry_id, tag]: Self::Args<'_>) -> bool {
            if self.iter.is_none() {
                let Some(ctx_atom) = ctx.get::<Atom>() else {
                    return false;
                };
                let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                    return false;
                };

                self.iter = if let Some(entry_id) = entry_id.get::<EntryId>() {
                    // Prioritize scanning the entry if the ID is instantiated,
                    // since there's likely less tags on a specific entry than
                    // tags with the same functor
                    log::debug!("querying tags by entry_id: {entry_id:?}");

                    Some(Box::new(
                        ctx.entries
                            .tags_by_entry(entry_id.0)
                            .map(move |(_, tag)| (entry_id, tag)),
                    ))
                } else if let Some(functor) = tag.get::<RawFunctor>() {
                    log::debug!("querying tags by functor: {tag:?}");

                    Some(Box::new(
                        ctx.entries
                            .tags_by_key(&functor)
                            .map(move |(entry_id, tag)| (EntryId(entry_id), tag)),
                    ))
                } else {
                    log::debug!("querying all tags");

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
