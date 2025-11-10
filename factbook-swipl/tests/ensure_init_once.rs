use factbook_swipl::Session;
use std::thread;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

#[test]
fn ensure_init_once() {
    let t1 = thread::spawn(|| Session::init(STATE));
    let t2 = thread::spawn(|| Session::init(STATE));

    assert!(t1.join().unwrap().xor(t2.join().unwrap()).is_some());
}
