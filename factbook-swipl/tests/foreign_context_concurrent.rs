use factbook_swipl::blob::ScopedBlob;
use factbook_swipl::foreign::Semidet;
use factbook_swipl::{Atom, Context, Session, term};
use factbook_swipl_macros::{ScopedBlobData, predicate};
use std::collections::BTreeSet;
use std::sync::RwLock;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[derive(ScopedBlobData)]
struct State(RwLock<BTreeSet<i64>>);

#[predicate(my_predicate(ctx, value) semidet)]
struct MyPredicate;

impl Semidet for MyPredicate {
    fn call(_: &mut impl Context, [ctx, value]: Self::Args<'_>) -> bool {
        let ctx_atom = ctx.get::<Atom>().unwrap();
        let ctx = ctx_atom.scoped_blob::<State>().unwrap();

        if let Some(value) = value.get::<i64>() {
            ctx.0.write().unwrap().insert(value);
        }

        true
    }
}

#[test]
fn foreign_context_concurrent() {
    let session = Session::init(STATE).unwrap();

    let state = State(RwLock::new(BTreeSet::new()));
    let state_blob = ScopedBlob::new(state);

    std::thread::scope(|s| {
        let threads = [1, 2, 3].map(|i| {
            let state_blob = &state_blob;
            let session = &session;

            s.spawn(move || {
                let engine = session.engine();

                let context_t = engine.new_term().put(state_blob);
                let goal = term! { &engine => my_predicate({context_t}, {i}) };

                engine.register_predicate::<MyPredicate>();
                engine.call(goal, None).unwrap();

                println!("{context_t:?}");
            })
        });

        for thread in threads {
            thread.join().unwrap();
        }
    });

    let state = state_blob.into_inner().unwrap();
    let items = state.0.into_inner().unwrap();

    assert_eq!(items, BTreeSet::from([1, 2, 3]));
}
