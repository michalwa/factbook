//! Parsers for entry and view languages
//!
//! These parsers are only used to extract highlighting spans and, in the case
//! of entries, spans corresponding to tags which are extracted and passed to
//! Prolog to be properly parsed into terms.

use pest::RuleType;
use serde::Serialize;

pub mod entry;
pub mod view;

/// Span of entry content or view definition with an attached semantic tag
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Span {
    pub kind: SpanKind,
    /// Character offset from the start of the input string
    pub start: usize,
    /// Length of the span in characters
    pub len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SpanKind {
    Punctuation,
    Ident,
    Number,
    String,
    Variable,
    Operator,
    Functor,
    Argument,
    Comment,
}

impl Span {
    fn from_pair<R: RuleType>(
        kind: SpanKind,
        pair: &pest::iterators::Pair<R>,
        input: &str,
    ) -> Self {
        let mut char_indices = input
            .char_indices()
            .map(|(i, _)| Some(i))
            .chain(std::iter::once(None));

        let [start, len] = [pair.as_span().start(), pair.as_span().end()].map(|pos| {
            char_indices
                .position(|i| i.is_none_or(|i| i == pos))
                .unwrap()
        });

        Self {
            kind,
            start,
            len: len + 1,
        }
    }
}

#[cfg(test)]
fn span(kind: SpanKind, start: usize, len: usize) -> Span {
    Span { kind, start, len }
}
