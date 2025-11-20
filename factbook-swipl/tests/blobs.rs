use factbook_swipl::blob::{Blob, BlobData, BlobRef, CopyBlob, CopyBlobData};
use factbook_swipl::{Context, Session};

const STATE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/state"));

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

#[test]
fn blobs() {
    let session = Session::init(STATE).unwrap();
    let engine = session.engine();

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
