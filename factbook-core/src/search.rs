use crate::model::{Entry, EntryId, ViewId};
use crate::{EntryStorage, State};
use factbook_swipl::blob::{CopyBlob, ScopedBlob, ScopedBlobData};
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
        // For any two terms `t1` and `t2`, if `TagKey::from(t1) != TagKey::from(t2)`
        // then `!t1.unify(t2)`. In other words, `t1.unify(t2)` requires that
        // `TagKey::from(t1) == TagKey::from(t2)`.
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
        F: FnMut(EntryId, &Entry),
    {
        let Some(view) = self.views.get(&view_id) else {
            return Err(ViewError::NonexistentView);
        };

        let mut pl = self.session.0.engine();

        let view_term = match pl.new_term().put_parsed(&view.definition) {
            Ok(view_term) => view_term,
            Err(ex) => {
                log::warn!("failed to parse view: {ex:?}");
                return Err(ViewError::Parse(ex.into_term().to_string()));
            },
        };

        let mut order_keys = BTreeMap::new();

        let ctx_blob = ScopedBlob::new(ViewContext {
            entries: &self.entries,
            order_keys: Mutex::new(&mut order_keys),
        });

        let query = open_query! { pl => view_entry({&ctx_blob}, {view_term}, _) }.unwrap();
        let mut visited = BTreeSet::new();
        let mut collected = Vec::new();

        while let Some([_, _, entry_id]) = query
            .next_solution()
            .map_err(|ex| ViewError::Prolog(ex.into_term().to_string()))?
        {
            if let Some(CopyBlob(entry_id)) = entry_id.get::<CopyBlob<EntryId>>() {
                visited.insert(entry_id);
            } else {
                // If the entry ID is uninstantiated, the user probably used a wildcard
                // view like `_`, and we should iterate all entries
                log::debug!("query returned uninstantiated entry ID, returning all entries");

                collected.extend(
                    self.entries
                        .entries()
                        .map(|(id, entry)| (EntryId(id), entry)),
                );

                // Ignore other solutions
                break;
            }
        }

        query.close();
        drop(ctx_blob);

        if !visited.is_empty() {
            collected.extend(
                visited
                    .into_iter()
                    .map(|id| (id, self.entries.entry_data(id.0))),
            );
        }

        {
            let pl = pl.frame();

            // Sort by ID first, then rely on stable sort to preserve ID-based order
            // for equal keys
            collected.sort_by_key(|(id, _)| *id);
            collected.sort_by_key(|(id, _)| order_keys.get(id).map(|key| pl.new_term().put(key)));
        }

        for (entry_id, entry) in collected {
            f(entry_id, entry);
        }

        Ok(())
    }
}

/// The scoped blob passed to the Prolog execution for predicates to be able to
/// query the state
#[derive(ScopedBlobData)]
struct ViewContext<'a> {
    entries: &'a EntryStorage,
    order_keys: Mutex<&'a mut BTreeMap<EntryId, Record>>,
}

pub(crate) mod predicates {
    use crate::model::EntryId;
    use crate::search::{TagKey, ViewContext};
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
            let Some(entry_id) = entry_id.get::<EntryId>() else {
                log::error!(
                    "set_entry_order_key(_, E, _): E must be of type EntryId, got {entry_id:?}"
                );
                return false;
            };

            if ctx
                .order_keys
                .lock()
                .unwrap()
                .insert(entry_id, key.record())
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
