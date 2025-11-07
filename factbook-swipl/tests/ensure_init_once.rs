use factbook_swipl::Session;
use std::thread;

#[test]
fn ensure_init_once() {
    let t1 = thread::spawn(|| Session::init(None));
    let t2 = thread::spawn(|| Session::init(None));

    assert!(t1.join().unwrap().xor(t2.join().unwrap()).is_some());
}
