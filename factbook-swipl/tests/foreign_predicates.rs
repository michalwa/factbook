use factbook_swipl::foreign::{Nondet, Predicate, Semidet, predicate};
use factbook_swipl::{Context, Session, assert_unify, term};

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[predicate(my_semidet_pred(t1, t2) semidet)]
struct MySemidetPred;

#[predicate(my_nondet_pred(t1) nondet)]
struct MyNondetPred {
    i: i32,
}

impl Semidet for MySemidetPred {
    fn call(ctx: &impl Context, [t1, t2]: Self::Args<'_>) -> bool {
        t1.unify_with(ctx.new_term().put(1)) && t2.unify_with(ctx.new_term().put(2))
    }
}

impl Nondet for MyNondetPred {
    fn init(_: &impl Context) -> Self {
        Self { i: 1 }
    }

    fn next(&mut self, ctx: &impl Context, [t1]: Self::Args<'_>) -> bool {
        if self.i <= 3 {
            let t = ctx.new_term().put(self.i);
            self.i += 1;
            t1.unify_with(t)
        } else {
            false
        }
    }
}

#[test]
fn foreign_nondet() {
    let session = Session::init(STATE).unwrap();
    let engine = session.engine();

    engine.register_predicate::<MySemidetPred>();
    engine.register_predicate::<MyNondetPred>();

    assert!(engine.predicate_defined::<2>(MySemidetPred::NAME.to_str().unwrap(), None));
    assert!(engine.predicate_defined::<1>(MyNondetPred::NAME.to_str().unwrap(), None));

    let [t1, t2] = engine.new_terms();
    let solutions = engine.new_term();
    let goal =
        term! { &engine => findall("-"({t1}, {t2}), my_semidet_pred({t1}, {t2}), {solutions}) };
    assert!(engine.call(goal, None).unwrap());
    assert_unify!(solutions, term! { &engine => ["-"(1, 2)] });

    let t = engine.new_term();
    let solutions = engine.new_term();
    let goal = term! { &engine => findall({t}, my_nondet_pred({t}), {solutions}) };
    assert!(engine.call(goal, None).unwrap());
    assert_unify!(solutions, term! { &engine => [1, 2, 3] });
}
