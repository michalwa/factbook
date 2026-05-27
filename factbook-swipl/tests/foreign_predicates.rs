use factbook_swipl::foreign::{Nondet, Predicate, Semidet, predicate};
use factbook_swipl::{Context, Session, assert_unify, term};
use std::sync::atomic::{AtomicUsize, Ordering};

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[predicate(my_semidet_pred(t1, t2) semidet)]
struct MySemidetPred;

#[predicate(my_nondet_pred(t1) nondet)]
struct MyNondetPred {
    i: i32,
}

impl Semidet for MySemidetPred {
    fn call(ctx: &mut impl Context, [t1, t2]: Self::Args<'_>) -> bool {
        t1.unify_with(ctx.new_term().put(1)) && t2.unify_with(ctx.new_term().put(2))
    }
}

impl Nondet for MyNondetPred {
    fn init(_: &impl Context) -> Self {
        Self { i: 1 }
    }

    fn next(&mut self, ctx: &mut impl Context, [t1]: Self::Args<'_>) -> bool {
        if self.i <= 3 {
            let t = ctx.new_term().put(self.i);
            self.i += 1;
            t1.unify_with(t)
        } else {
            false
        }
    }
}

static MY_NONDET_PRED_DROP_COUNT: AtomicUsize = AtomicUsize::new(0);

impl Drop for MyNondetPred {
    fn drop(&mut self) {
        MY_NONDET_PRED_DROP_COUNT.fetch_add(1, Ordering::SeqCst);
    }
}

#[test]
fn foreign_nondet() {
    let session = Session::init(STATE)
        .unwrap()
        .register_predicate::<MySemidetPred>()
        .register_predicate::<MyNondetPred>();

    let engine = session.engine();

    assert!(engine.predicate_defined::<2>(MySemidetPred::NAME.to_str().unwrap(), None));
    assert!(engine.predicate_defined::<1>(MyNondetPred::NAME.to_str().unwrap(), None));

    let [t1, t2] = engine.new_terms().into();
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

    assert_eq!(MY_NONDET_PRED_DROP_COUNT.swap(0, Ordering::SeqCst), 1);

    // Each invocation of a foreign predicate should be its own instance
    solutions.put_variable();
    let goal = term! { &engine => findall("-"({t1}, {t2}), ","(my_nondet_pred({t1}), my_nondet_pred({t2})), {solutions}) };

    assert!(engine.call(goal, None).unwrap());
    assert_unify!(solutions, term! { &engine => [
        "-"(1, 1), "-"(1, 2), "-"(1, 3),
        "-"(2, 1), "-"(2, 2), "-"(2, 3),
        "-"(3, 1), "-"(3, 2), "-"(3, 3)
    ] });

    // 1 (first invocation) + 3 (second invocation for each solution of the first)
    assert_eq!(MY_NONDET_PRED_DROP_COUNT.swap(0, Ordering::SeqCst), 4);
}
