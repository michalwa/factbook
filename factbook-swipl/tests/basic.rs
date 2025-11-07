use factbook_swipl::*;
use std::thread;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[test]
fn threads() {
    let session = Session::get_or_init(STATE);

    thread::spawn(|| {
        let engine = dbg!(session.engine());
        let t = engine.new_term().put_atom_chars("foo");

        engine.assert(&t);
    })
    .join()
    .unwrap();

    let engine = dbg!(session.engine());
    let t = engine.new_term().put_atom_chars("foo");

    assert!(engine.call(&t));
}
