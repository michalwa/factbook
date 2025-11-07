use serde::{Serialize, Serializer};
use std::cell::Cell;

pub struct SerializeIterOnce<I>(Cell<Option<I>>);

impl<I> SerializeIterOnce<I> {
    pub fn new(iter: impl IntoIterator<IntoIter = I>) -> Self {
        Self(Cell::new(Some(iter.into_iter())))
    }
}

impl<I> Serialize for SerializeIterOnce<I>
where
    I: Iterator,
    I::Item: Serialize,
{
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.collect_seq(
            self.0
                .take()
                .expect("`SerializeIterOnce` may only be serialized once"),
        )
    }
}
