use factbook_swipl::query::open_query;
use factbook_swipl::{Context, Session};
use std::sync::LazyLock;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

static SESSION: LazyLock<Session<'static>> = LazyLock::new(|| Session::init(STATE).unwrap());

#[test]
fn query() {
    let mut engine = SESSION.engine();
    let mut solutions = Vec::new();

    let query = open_query! { engine => lists:member(_, [1, 2]) }.unwrap();

    while let Some([member, _]) = query.next_solution().unwrap() {
        solutions.push(member.get::<i64>().unwrap());
    }

    assert_eq!(solutions, [1, 2]);
}
