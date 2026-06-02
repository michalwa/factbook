use factbook_swipl::blob::{Blob, BlobData, CopyBlob, CopyBlobData, ScopedBlob};
use factbook_swipl::foreign::Semidet;
use factbook_swipl::{Atom, Context, Session, assert_unify, term};
use factbook_swipl_macros::{ScopedBlobData, predicate};
use std::cell::RefCell;
use std::sync::LazyLock;

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

static SESSION: LazyLock<Session<'static>> = LazyLock::new(|| {
    Session::init(STATE)
        .unwrap()
        .register_predicate::<CustomPredicate>()
});

#[derive(ScopedBlobData)]
struct MyVec(RefCell<Vec<i32>>);

#[predicate(custom_predicate(t_blob) semidet)]
struct CustomPredicate;

impl Semidet for CustomPredicate {
    fn call(_: &mut impl Context, [t_blob]: Self::Args<'_>) -> bool {
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

#[test]
fn blob() {
    #[derive(Debug, BlobData, PartialEq)]
    struct FooBlob {
        text: String,
        number: i32,
    }

    #[derive(Debug, BlobData, PartialEq)]
    struct BarBlob {
        number: f32,
    }

    let engine = SESSION.engine();
    let [t1, t2, t3] = engine.new_terms().into();

    t1.put(Blob::new(FooBlob {
        text: "Hello".into(),
        number: 42,
    }));
    assert_eq!(t1.to_string(), "<FooBlob { text: \"Hello\", number: 42 }>");

    assert_unify!(t1, t2);
    t3.put(Blob::new(BarBlob { number: 1.0 }));

    let a2 = t2.get::<Atom>().unwrap();
    let a3 = t3.get::<Atom>().unwrap();

    assert_eq!(*a2.blob::<FooBlob>().unwrap(), FooBlob {
        text: "Hello".into(),
        number: 42,
    });
    assert_eq!(a3.blob::<FooBlob>(), None);
}

#[test]
fn copy_blob() {
    #[derive(Debug, Clone, Copy, CopyBlobData, PartialEq)]
    struct Vec2i {
        x: i32,
        y: i32,
    }

    #[derive(Debug, Clone, Copy, CopyBlobData, PartialEq)]
    struct Vec3i {
        x: i32,
        y: i32,
        z: i32,
    }

    let engine = SESSION.engine();
    let [t1, t2, t3, t4] = engine.new_terms().into();

    t1.put(CopyBlob(Vec2i { x: 1, y: 2 }));
    assert_eq!(t1.to_string(), "<Vec2i { x: 1, y: 2 }>");

    assert_unify!(t1, t2);
    t3.put(CopyBlob(Vec3i { x: 1, y: 2, z: 3 }));

    assert_eq!(t2.get::<CopyBlob<Vec2i>>().unwrap().0, Vec2i { x: 1, y: 2 });
    assert_eq!(t3.get::<CopyBlob<Vec2i>>(), None);
    assert_eq!(t4.get::<CopyBlob<Vec2i>>(), None);
}

#[test]
fn scoped_blob_with_foreign_predicate() {
    let engine = SESSION.engine();

    let xs = MyVec(RefCell::new(Vec::<i32>::new()));
    let t = engine.new_term();

    let xs = {
        let blob = ScopedBlob::new(xs);
        t.put(&blob);

        let goal = term! { &engine => custom_predicate({t}) };
        assert!(engine.call(goal, None).unwrap());
        assert_eq!(t.to_string(), "<MyVec>");

        blob.into_inner().unwrap()
    };

    assert_eq!(xs.0.into_inner(), [1]);

    // Blob is dropped and references cannot be obtained anymore
    let goal = term! { &engine => custom_predicate({t}) };
    assert!(!engine.call(goal, None).unwrap());
    assert_eq!(t.to_string(), "<MyVec (invalid)>");
}
