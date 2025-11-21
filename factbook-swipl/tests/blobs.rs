use factbook_swipl::blob::{Blob, BlobData, BlobRef, CopyBlob, CopyBlobData, ScopedBlob};
use factbook_swipl::foreign::Semidet;
use factbook_swipl::{Atom, Context, Session, term};
use factbook_swipl_macros::{ScopedBlobData, predicate};
use std::cell::RefCell;
use std::sync::LazyLock;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

pub(crate) static SESSION: LazyLock<Session<'static>> =
    LazyLock::new(|| Session::init(STATE).unwrap());

#[test]
fn blobs() {
    #[derive(Debug, BlobData, PartialEq)]
    struct MyBlob {
        text: String,
        number: i32,
    }

    #[derive(Debug, Clone, Copy, CopyBlobData, PartialEq)]
    struct Vec2i {
        x: i32,
        y: i32,
    }

    let engine = SESSION.engine();
    let [t1, t2, t3, t4] = engine.new_terms();

    t1.put(Blob::new(MyBlob {
        text: "Hello".into(),
        number: 42,
    }));
    t2.put(CopyBlob(Vec2i { x: 1, y: 2 }));

    assert_eq!(t1.to_string(), "MyBlob { text: \"Hello\", number: 42 }");
    assert_eq!(t2.to_string(), "Vec2i { x: 1, y: 2 }");

    assert!(t3.unify_with(t1));
    assert_eq!(*t3.get::<BlobRef<MyBlob>>().unwrap(), MyBlob {
        text: "Hello".into(),
        number: 42,
    });

    assert!(t4.unify_with(t2));
    assert_eq!(t4.get::<CopyBlob<Vec2i>>().unwrap().0, Vec2i { x: 1, y: 2 });
}

#[test]
fn scoped_blob_with_foreign_predicate() {
    #[derive(ScopedBlobData)]
    struct MyVec(RefCell<Vec<i32>>);

    #[predicate(custom_predicate(t_blob) semidet)]
    struct CustomPredicate;

    impl Semidet for CustomPredicate {
        fn call(_: &impl Context, [t_blob]: Self::Args<'_>) -> bool {
            let Some(atom) = t_blob.get::<Atom>() else {
                return false;
            };
            let Some(xs) = atom.scoped_blob::<MyVec>() else {
                return false;
            };
            xs.0.borrow_mut().push(1);
            true
        }
    }

    let engine = SESSION.engine();

    engine.register_predicate::<CustomPredicate>();

    let xs = MyVec(RefCell::new(Vec::<i32>::new()));
    let t = engine.new_term();

    {
        let blob = ScopedBlob::new(&xs);
        t.put(&blob);
        assert!(engine.call(term! { &engine => custom_predicate({t}) }));
    }

    assert_eq!(*xs.0.borrow(), [1]);

    // Blob is dropped and references cannot be obtained anymore
    assert!(!engine.call(term! { &engine => custom_predicate({t}) }));
}
