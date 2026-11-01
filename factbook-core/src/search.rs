use crate::model::{EntryId, ViewId, ViewItem};
use crate::{EntryStorage, State};
use factbook_swipl::blob::{CopyBlob, CopyBlobData, ScopedBlob, ScopedBlobData};
use factbook_swipl::query::open_query;
use factbook_swipl::term::{Term, TermKind};
use factbook_swipl::{Context, RawFunctor, Record};
use sparse_tags::Store;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Mutex;

/// Simplified representation of a tag type used to index and look up similar
/// tags
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TagKey {
    Functor(RawFunctor),
    Atom(String),
    String(String),
    Other,
}

impl From<&Term<'_>> for TagKey {
    fn from(value: &Term) -> Self {
        // The invariant of this mapping required for the search to be able to
        // find all valid matches is the following:
        //
        // For any two terms `t1` and `t2`, if `TagKey::from(t1) !=
        // TagKey::from(t2)` then `!t1.unify(t2)`. In other words,
        // `t1.unify(t2)` requires that `TagKey::from(t1) == TagKey::from(t2)`.
        match value.kind() {
            TermKind::Compound | TermKind::ListPair => {
                TagKey::Functor(value.get::<RawFunctor>().unwrap())
            },
            TermKind::Atom => TagKey::Atom(value.atom_chars().unwrap().into()),
            TermKind::String => TagKey::String(value.string_chars().unwrap().into()),
            _ => TagKey::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, CopyBlobData)]
pub struct SyntheticId(pub(crate) usize);

impl factbook_swipl::term::ToTerm for SyntheticId {
    fn put_in(self, term: factbook_swipl::term::Term) {
        term.put(CopyBlob(self));
    }

    fn unify_with(self, term: factbook_swipl::term::Term) -> bool {
        term.unify(CopyBlob(self))
    }
}

impl factbook_swipl::term::FromTerm for SyntheticId {
    fn from_term(term: factbook_swipl::term::Term) -> Option<Self> {
        Some(term.get::<CopyBlob<Self>>()?.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ViewItemId {
    Entry(EntryId),
    Synthetic(SyntheticId),
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ViewError {
    #[error("view does not exist")]
    NonexistentView,
    #[error("error parsing view: {0}")]
    Parse(String),
    #[error("prolog error: {0}")]
    Prolog(String),
}

impl State<'_> {
    pub fn for_each_view_entry<F>(&self, view_id: ViewId, mut f: F) -> Result<(), ViewError>
    where
        F: for<'a> FnMut(ViewItem<'a>),
    {
        let Some(view) = self.views.get(&view_id) else {
            return Err(ViewError::NonexistentView);
        };

        let mut pl = self.session.0.engine();

        let view_term = pl.new_term().put_parsed(&view.definition).map_err(|ex| {
            log::warn!("failed to parse view: {ex:?}");
            ViewError::Parse(ex.into_term().to_string())
        })?;

        let mut order_keys = BTreeMap::new();
        let mut synthetic_entries = Vec::new();

        let ctx_blob = ScopedBlob::new(ViewContext {
            entries: &self.entries,
            order_keys: Mutex::new(&mut order_keys),
            synthetic_entries: Mutex::new(&mut synthetic_entries),
        });

        let query = open_query! { pl => view_entry({&ctx_blob}, {view_term}, _) }.unwrap();
        let mut ids = BTreeSet::new();

        while let Some([_, _, entry_id]) = query.next_solution().map_err(|ex| {
            log::warn!("view returned an error: {ex:?}");
            ViewError::Prolog(ex.into_term().to_string())
        })? {
            if let Some(entry_id) = entry_id.get::<EntryId>() {
                ids.insert(ViewItemId::Entry(entry_id));
            } else if let Some(synthetic_id) = entry_id.get::<SyntheticId>() {
                ids.insert(ViewItemId::Synthetic(synthetic_id));
            } else if entry_id.kind() == TermKind::Variable {
                // If the entry ID is uninstantiated, the user probably used a
                // wildcard view like `_`, and we should iterate all entries
                log::debug!("query returned uninstantiated entry ID, returning all entries");

                ids.extend(self.entries.entry_ids().map(EntryId).map(ViewItemId::Entry));

                // Ignore other solutions
                break;
            }
        }

        query.close();
        drop(ctx_blob);

        let mut ids: Vec<_> = ids.into_iter().collect();

        if !order_keys.is_empty() {
            let pl = pl.frame();
            ids.sort_by_key(|id| order_keys.get(id).map(|key| pl.new_term().put(key)));
        }

        for id in ids {
            match id {
                ViewItemId::Entry(id) => f(ViewItem::Entry(id, self.entries.entry_data(id.0))),
                ViewItemId::Synthetic(id) => f(ViewItem::Synthetic(&synthetic_entries[id.0])),
            }
        }

        Ok(())
    }
}

/// The scoped blob passed to the Prolog execution for predicates to be able to
/// query the state
#[derive(ScopedBlobData)]
struct ViewContext<'a> {
    entries: &'a EntryStorage,
    order_keys: Mutex<&'a mut BTreeMap<ViewItemId, Record>>,
    synthetic_entries: Mutex<&'a mut Vec<String>>,
}

pub(crate) mod predicates {
    use crate::model::EntryId;
    use crate::search::{SyntheticId, TagKey, ViewContext, ViewItemId};
    use chrono::{Datelike, Timelike};
    use factbook_swipl::foreign::{Nondet, Semidet, predicate};
    use factbook_swipl::term::{Term, TermKind};
    use factbook_swipl::{Atom, Context, Record, Session, term};
    use sparse_tags::Store;

    pub(crate) fn register(session: Session) -> Session {
        session
            .register_predicate::<EntryTag>()
            .register_predicate::<SetEntryOrderKey>()
            .register_predicate::<EntryCreated>()
            .register_predicate::<PutSyntheticEntry>()
    }

    #[predicate(entry_tag(ctx, entry_id, tag) nondet)]
    pub(crate) struct EntryTag<'a> {
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
                    log::error!(
                        "entry_tag(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                    );
                    return false;
                };
                let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                    log::error!(
                        "entry_tag(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                    );
                    return false;
                };

                self.iter = Some(if let Some(entry_id) = entry_id.get::<EntryId>() {
                    // Prioritize scanning the entry if the ID is instantiated,
                    // since there's likely less tags on a specific entry than
                    // tags with the same functor
                    log::debug!("entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query by entry_id");

                    Box::new(
                        ctx.entries
                            .tags_by_entry(entry_id.0)
                            .map(move |(_, tag)| (entry_id, tag)),
                    )
                } else if tag.kind() != TermKind::Variable {
                    let key = TagKey::from(&tag);

                    log::debug!(
                        "entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query by tag {key:?}"
                    );

                    Box::new(
                        ctx.entries
                            .tags_by_key(&key)
                            .map(move |(entry_id, tag)| (EntryId(entry_id), tag)),
                    )
                } else {
                    log::debug!("entry_tag({ctx_arg:?}, {entry_id:?}, {tag:?}): query all");

                    Box::new(
                        ctx.entries
                            .tags()
                            .map(|(entry_id, _, tag)| (EntryId(entry_id), tag)),
                    )
                });
            }

            for (found_entry_id, found_tag) in self.iter.as_mut().unwrap() {
                let pl = pl.frame();

                if unify_entry_id(found_entry_id, entry_id) && tag.unify(found_tag) {
                    pl.close();
                    return true;
                }
            }

            false
        }
    }

    #[predicate(set_entry_order_key(ctx, entry_id, key) semidet)]
    pub(crate) struct SetEntryOrderKey;

    impl Semidet for SetEntryOrderKey {
        fn call(_: &mut impl Context, [ctx_arg, entry_id, key]: Self::Args<'_>) -> bool {
            let Some(ctx_atom) = ctx_arg.get::<Atom>() else {
                log::error!(
                    "set_entry_order_key(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };
            let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                log::error!(
                    "set_entry_order_key(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };

            let Some(id) = entry_id
                .get::<EntryId>()
                .map(ViewItemId::Entry)
                .or_else(|| entry_id.get::<SyntheticId>().map(ViewItemId::Synthetic))
            else {
                log::error!(
                    "set_entry_order_key(_, E, _): E must be of type EntryId or SyntheticId, got \
                     {entry_id:?}"
                );
                return false;
            };

            if ctx
                .order_keys
                .lock()
                .unwrap()
                .insert(id, key.record())
                .is_some()
            {
                log::warn!("redefinition of order key for {entry_id:?}");
            }

            true
        }
    }

    #[predicate(entry_created(ctx, entry_id, timestamp) nondet)]
    pub(crate) struct EntryCreated<'a> {
        iter: Option<Box<dyn Iterator<Item = EntryId> + 'a>>,
    }

    impl Nondet for EntryCreated<'_> {
        fn init(_: &impl Context) -> Self {
            Self { iter: None }
        }

        fn next(
            &mut self,
            pl: &mut impl Context,
            [ctx_arg, entry_id_arg, timestamp]: Self::Args<'_>,
        ) -> bool {
            let Some(ctx_atom) = ctx_arg.get::<Atom>() else {
                log::error!(
                    "entry_created(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };
            let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                log::error!(
                    "entry_created(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };

            let iter = self
                .iter
                .get_or_insert_with(|| match entry_id_arg.get::<EntryId>() {
                    Some(entry_id) => Box::new(std::iter::once(entry_id)),
                    None => Box::new(ctx.entries.entry_ids().map(EntryId)),
                });

            for entry_id in iter {
                let pl = pl.frame();

                if unify_entry_id(entry_id, entry_id_arg) {
                    let created_at = &ctx.entries.entry_data(entry_id.0).created_at;
                    let second = created_at.second() as f64
                        + created_at.nanosecond() as f64 / 1_000_000_000.0;

                    let date = term! {
                        &pl => date(
                            {created_at.year()},
                            {created_at.month()},
                            {created_at.day()},
                            {created_at.hour()},
                            {created_at.minute()},
                            {second},
                            _, _, _
                        )
                    };
                    let goal = term! { &pl => date_time_stamp({date}, {timestamp}) };

                    if pl.call(goal, None).unwrap() {
                        pl.close();
                        return true;
                    }
                }
            }

            false
        }
    }

    #[predicate(put_synthetic_entry(ctx, entry_id, content) semidet)]
    pub(crate) struct PutSyntheticEntry;

    impl Semidet for PutSyntheticEntry {
        fn call(_: &mut impl Context, [ctx_arg, entry_id, content]: Self::Args<'_>) -> bool {
            let Some(ctx_atom) = ctx_arg.get::<Atom>() else {
                log::error!(
                    "put_synthetic_entry(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };
            let Some(ctx) = ctx_atom.scoped_blob::<ViewContext>() else {
                log::error!(
                    "put_synthetic_entry(C, _, _): C must be of type ViewContext, got {ctx_arg:?}"
                );
                return false;
            };

            if entry_id.kind() != TermKind::Variable {
                log::error!(
                    "put_synthetic_entry(_, E, _): E must be uninstantiated, got {entry_id:?}"
                );
                return false;
            }

            let mut entries = ctx.synthetic_entries.lock().unwrap();
            let id = entries.len();
            entries.push(
                content
                    .string_chars()
                    .map(String::from)
                    .unwrap_or_else(|| content.to_string()),
            );

            entry_id.unify(SyntheticId(id))
        }
    }

    fn unify_entry_id(lhs: EntryId, rhs: Term<'_>) -> bool {
        // If `entry_id` is already instantiated, manually compare instead of
        // unifying terms, because unifying blobs like this will always fail,
        // even if they have the same contents
        match rhs.get::<EntryId>() {
            Some(entry_id) => entry_id == lhs,
            None => rhs.unify(lhs),
        }
    }
}
