use factbook_swipl::term::Term;
use factbook_swipl::{Context, Engine};
use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;
use serde::Serialize;

#[derive(Parser)]
#[grammar = "entry.pest"]
struct EntryParser;

pub struct ParseResult<'c> {
    pub tags: Vec<Term<'c>>,
    pub spans: Vec<Span>,
}

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
}

impl Span {
    fn from_pair(kind: SpanKind, pair: &pest::iterators::Pair<Rule>, input: &str) -> Self {
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

// Wrapper around `parse` to avoid having to reexport a `Context` type from
// `factbook_swipl`
pub fn parse_spans(input: &str) -> Vec<Span> {
    parse(input, None::<&Engine>).spans
}

pub fn parse<'c>(input: &str, ctx: Option<&'c impl Context>) -> ParseResult<'c> {
    let mut tags = Vec::new();
    let mut spans = Vec::new();

    // The parser should never fail, because it includes `Rule::text` which
    // matches arbitrary input
    let top = EntryParser::parse(Rule::entry, input)
        .unwrap()
        .exactly_one()
        .unwrap();

    assert_eq!(top.as_rule(), Rule::entry);

    for pair in top.into_inner() {
        match pair.as_rule() {
            Rule::tag => {
                let [_, tag] = pair
                    .clone()
                    .into_inner()
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();

                if let Some(ctx) = ctx
                    && let Ok(tag) = ctx
                        .new_term()
                        .put_parsed(tag.as_str())
                        .map_err(|e| log::warn!("failed to parse tag: {e:?}"))
                {
                    tags.push(tag);
                }

                for pair in pair.into_inner().flatten() {
                    use SpanKind as T;

                    if let Some(kind) = match pair.as_node_tag() {
                        Some("functor") => Some(T::Functor),
                        Some("argument") => Some(T::Argument),
                        _ => None,
                    } {
                        spans.push(Span::from_pair(kind, &pair, input));
                    }

                    if let Some(kind) = match pair.as_rule() {
                        Rule::at
                        | Rule::lparen
                        | Rule::rparen
                        | Rule::lbracket
                        | Rule::rbracket
                        | Rule::comma => Some(T::Punctuation),
                        Rule::ident | Rule::quoted => Some(T::Ident),
                        Rule::string => Some(T::String),
                        Rule::number => Some(T::Number),
                        Rule::variable => Some(T::Variable),
                        Rule::operator | Rule::operator_ex => Some(T::Operator),
                        _ => None,
                    } {
                        spans.push(Span::from_pair(kind, &pair, input));
                    }
                }
            },
            Rule::EOI => (),
            rule => panic!("parser returned unexpected rule: {rule:?}"),
        }
    }

    ParseResult { tags, spans }
}

#[cfg(test)]
mod test {
    use super::{Span, SpanKind as S};
    use factbook_swipl::Context;
    use pretty_assertions::assert_eq;
    use test_log::test;

    fn s(kind: S, start: usize, len: usize) -> Span {
        Span { kind, start, len }
    }

    fn parse(input: &str, ctx: &impl Context) -> (Vec<String>, Vec<Span>) {
        let result = super::parse(input, Some(ctx));
        (
            result.tags.into_iter().map(|t| t.to_string()).collect(),
            result.spans,
        )
    }

    #[test]
    fn parse_empty() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("", &engine);
        assert_eq!(tags, [] as [&str; _]);
        assert_eq!(spans, [] as [Span; _]);
    }

    #[test]
    fn parse_single_atom() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo", &engine);
        assert_eq!(tags, ["foo"]);
        assert_eq!(spans, [s(S::Punctuation, 0, 1), s(S::Ident, 1, 3)]);
    }

    #[test]
    fn parse_single_atom_with_surrounding_content() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("  bar  @foo  bar  ", &engine);
        assert_eq!(tags, ["foo"]);
        assert_eq!(spans, [s(S::Punctuation, 7, 1), s(S::Ident, 8, 3)]);
    }

    #[test]
    fn parse_single_atom_with_surrounding_line_breaks() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("\n@foo\n", &engine);
        assert_eq!(tags, ["foo"]);
        assert_eq!(spans, [s(S::Punctuation, 1, 1), s(S::Ident, 2, 3)]);
    }

    #[test]
    fn parse_two_atoms() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo @x", &engine);
        assert_eq!(tags, ["foo", "x"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 5, 1),
            s(S::Ident, 6, 1)
        ]);
    }

    #[test]
    fn parse_two_adjacent_atoms() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo@bar", &engine);
        assert_eq!(tags, [] as [&str; _]);
        assert_eq!(spans, [] as [Span; _]);
    }

    #[test]
    fn parse_single_compound() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo(bar)", &engine);
        assert_eq!(tags, ["foo(bar)"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 3),
            s(S::Ident, 5, 3),
            s(S::Punctuation, 8, 1)
        ]);
    }

    #[test]
    fn parse_two_compound() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo(bar) @bar( baz , 1 )", &engine);
        assert_eq!(tags, ["foo(bar)", "bar(baz,1)"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 3),
            s(S::Ident, 5, 3),
            s(S::Punctuation, 8, 1),
            s(S::Punctuation, 10, 1),
            s(S::Functor, 11, 3),
            s(S::Ident, 11, 3),
            s(S::Punctuation, 14, 1),
            // FIXME: `#argument` nodes consume trailing whitespace
            s(S::Argument, 16, 4),
            s(S::Ident, 16, 3),
            s(S::Punctuation, 20, 1),
            s(S::Argument, 22, 2),
            s(S::Number, 22, 1),
            s(S::Punctuation, 24, 1),
        ]);
    }

    #[test]
    fn parse_quoted() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@'foo bar'", &engine);
        assert_eq!(tags, ["'foo bar'"]);
        assert_eq!(spans, [s(S::Punctuation, 0, 1), s(S::Ident, 1, 9)]);
    }

    #[test]
    fn parse_string() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@\"foo bar\"", &engine);
        assert_eq!(tags, ["\"foo bar\""]);
        assert_eq!(spans, [s(S::Punctuation, 0, 1), s(S::String, 1, 9)]);
    }

    #[test]
    fn parse_compound_with_string_argument_with_spaces() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo(\"foo bar\")", &engine);
        assert_eq!(tags, ["foo(\"foo bar\")"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 9),
            s(S::String, 5, 9),
            s(S::Punctuation, 14, 1)
        ]);
    }

    #[test]
    fn parse_number() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@42", &engine);
        assert_eq!(tags, ["42"]);
        assert_eq!(spans, [s(S::Punctuation, 0, 1), s(S::Number, 1, 2)]);
    }

    #[test]
    fn parse_compound_with_numbers() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo(42, 12.34)", &engine);
        assert_eq!(tags, ["foo(42,12.34)"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 2),
            s(S::Number, 5, 2),
            s(S::Punctuation, 7, 1),
            s(S::Argument, 9, 5),
            s(S::Number, 9, 5),
            s(S::Punctuation, 14, 1)
        ]);
    }

    #[test]
    fn parse_compound_nested() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, _) = parse("@foo(bar(1, 2), 3, 'baz'(4, 5))", &engine);
        assert_eq!(tags, ["foo(bar(1,2),3,baz(4,5))"]);
    }

    #[test]
    fn parse_variables() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@Var @X @_ @_foo", &engine);
        assert_eq!(tags, ["_", "_", "_", "_"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Variable, 1, 3),
            s(S::Punctuation, 5, 1),
            s(S::Variable, 6, 1),
            s(S::Punctuation, 8, 1),
            s(S::Variable, 9, 1),
            s(S::Punctuation, 11, 1),
            s(S::Variable, 12, 4)
        ]);
    }

    #[test]
    fn parse_parens() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, _) = parse("@(42) @foo(((42)))", &engine);
        assert_eq!(tags, ["42", "foo(42)"]);
    }

    #[test]
    fn parse_operators() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo/bar @foo-bar-baz @-42 @+42", &engine);
        assert_eq!(tags, ["foo/bar", "foo-bar-baz", "-42", "+42"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Ident, 1, 3),
            s(S::Operator, 4, 1),
            s(S::Ident, 5, 3),
            s(S::Punctuation, 9, 1),
            s(S::Ident, 10, 3),
            s(S::Operator, 13, 1),
            s(S::Ident, 14, 3),
            s(S::Operator, 17, 1),
            s(S::Ident, 18, 3),
            s(S::Punctuation, 22, 1),
            s(S::Operator, 23, 1),
            s(S::Number, 24, 2),
            s(S::Punctuation, 27, 1),
            s(S::Operator, 28, 1),
            s(S::Number, 29, 2),
        ]);
    }

    #[test]
    fn parse_special_operators() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@(1,2) @(1=@=2)", &engine);
        assert_eq!(tags, ["1,2", "1=@=2"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Punctuation, 1, 1),
            s(S::Number, 2, 1),
            s(S::Operator, 3, 1),
            s(S::Number, 4, 1),
            s(S::Punctuation, 5, 1),
            s(S::Punctuation, 7, 1),
            s(S::Punctuation, 8, 1),
            s(S::Number, 9, 1),
            s(S::Operator, 10, 3),
            s(S::Number, 13, 1),
            s(S::Punctuation, 14, 1),
        ]);
    }

    #[test]
    fn parse_list() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, spans) = parse("@foo([1, 2, 3]) @[4, 5, 6] @[]", &engine);

        assert_eq!(tags, ["foo([1,2,3])", "[4,5,6]", "[]"]);
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 9),
            s(S::Punctuation, 5, 1),
            s(S::Argument, 6, 1),
            s(S::Number, 6, 1),
            s(S::Punctuation, 7, 1),
            s(S::Argument, 9, 1),
            s(S::Number, 9, 1),
            s(S::Punctuation, 10, 1),
            s(S::Argument, 12, 1),
            s(S::Number, 12, 1),
            s(S::Punctuation, 13, 1),
            s(S::Punctuation, 14, 1),
            s(S::Punctuation, 16, 1),
            s(S::Punctuation, 17, 1),
            s(S::Argument, 18, 1),
            s(S::Number, 18, 1),
            s(S::Punctuation, 19, 1),
            s(S::Argument, 21, 1),
            s(S::Number, 21, 1),
            s(S::Punctuation, 22, 1),
            s(S::Argument, 24, 1),
            s(S::Number, 24, 1),
            s(S::Punctuation, 25, 1),
            s(S::Punctuation, 27, 1),
            s(S::Punctuation, 28, 1),
            s(S::Punctuation, 29, 1),
        ]);
    }
}
