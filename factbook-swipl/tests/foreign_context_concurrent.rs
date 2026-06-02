use factbook_swipl::blob::ScopedBlob;
use factbook_swipl::foreign::Nondet;
use factbook_swipl::{Atom, Context, Session};
use factbook_swipl_macros::{ScopedBlobData, open_query, predicate};

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[derive(ScopedBlobData)]
struct MyContext {
    start: i64,
}

#[predicate(my_predicate(ctx, value) nondet)]
struct MyPredicate {
    iter: Option<Box<dyn Iterator<Item = i64>>>,
}

impl Nondet for MyPredicate {
    fn init(_: &impl Context) -> Self {
        Self { iter: None }
    }

    fn next(&mut self, _: &mut impl Context, [ctx, value]: Self::Args<'_>) -> bool {
        if self.iter.is_none() {
            let ctx_atom = ctx.get::<Atom>().unwrap();
            let ctx = ctx_atom.scoped_blob::<MyContext>().unwrap();

            self.iter = Some(Box::new(ctx.start..))
        }

        value.unify(self.iter.as_mut().unwrap().next().unwrap())
    }
}

#[test]
fn foreign_concurrent() {
    let session = Session::init(STATE)
        .unwrap()
        .register_predicate::<MyPredicate>();

    let ctx_blob = ScopedBlob::new(MyContext { start: 7 });

    std::thread::scope(|s| {
        let threads: [_; 8] = std::array::from_fn(|_| {
            let session = &session;
            let ctx_blob = &ctx_blob;

            s.spawn(move || {
                let engine = session.engine();
                let query = open_query! { engine => my_predicate({ctx_blob}, _) }.unwrap();

                let solutions: [_; 3] = std::array::from_fn(|_| {
                    let [_, value] = query.next_solution().unwrap().unwrap();
                    value.get::<i64>().unwrap()
                });

                solutions
            })
        });

        for thread in threads {
            assert_eq!(thread.join().unwrap(), [7, 8, 9]);
        }
    });
}
