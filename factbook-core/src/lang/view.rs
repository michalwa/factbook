use crate::lang::Span;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "lang/view.pest"]
struct ViewParser;

pub fn parse_spans(input: &str) -> Option<Vec<Span>> {
    let mut spans = Vec::new();

    let top = ViewParser::parse(Rule::program, input).ok()?;

    for pair in top.flatten() {
        use crate::lang::SpanKind as S;

        if let Some(kind) = match pair.as_node_tag() {
            Some("functor") => Some(S::Functor),
            Some("argument") => Some(S::Argument),
            _ => None,
        } {
            spans.push(Span::from_pair(kind, &pair, input));
        }

        if let Some(kind) = match pair.as_rule() {
            Rule::at
            | Rule::lparen
            | Rule::rparen
            | Rule::lsquare
            | Rule::rsquare
            | Rule::lcurly
            | Rule::rcurly
            | Rule::comma
            | Rule::semi => Some(S::Punctuation),
            Rule::ident | Rule::quoted => Some(S::Ident),
            Rule::string => Some(S::String),
            Rule::number => Some(S::Number),
            Rule::variable => Some(S::Variable),
            Rule::operator_other => Some(S::Operator),
            Rule::COMMENT => Some(S::Comment),
            _ => None,
        } {
            spans.push(Span::from_pair(kind, &pair, input));
        }
    }

    Some(spans)
}

#[cfg(test)]
mod test {
    use super::parse_spans;
    use crate::lang::{Span, SpanKind as S, span as s};
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_empty() {
        let spans = parse_spans("").unwrap();
        assert_eq!(spans, [] as [Span; _]);
    }

    #[test]
    fn parse_single_tag() {
        let spans = parse_spans("@foo").unwrap();
        assert_eq!(spans, [s(S::Punctuation, 0, 1), s(S::Ident, 1, 3)]);
    }

    #[test]
    fn parse_two_compound_tags() {
        let spans = parse_spans("@foo(1, 2), @bar(3)").unwrap();
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Functor, 1, 3),
            s(S::Ident, 1, 3),
            s(S::Punctuation, 4, 1),
            s(S::Argument, 5, 1),
            s(S::Number, 5, 1),
            s(S::Punctuation, 6, 1),
            s(S::Argument, 8, 1),
            s(S::Number, 8, 1),
            s(S::Punctuation, 9, 1),
            s(S::Punctuation, 10, 1),
            s(S::Punctuation, 12, 1),
            s(S::Functor, 13, 3),
            s(S::Ident, 13, 3),
            s(S::Punctuation, 16, 1),
            s(S::Argument, 17, 1),
            s(S::Number, 17, 1),
            s(S::Punctuation, 18, 1),
        ]);
    }

    #[test]
    fn parse_comments() {
        let spans = parse_spans("% comment\n@foo % comment").unwrap();
        assert_eq!(spans, [
            s(S::Comment, 0, 9),
            s(S::Punctuation, 10, 1),
            s(S::Ident, 11, 3),
            s(S::Comment, 15, 9),
        ]);
    }

    #[test]
    fn parse_comment_inside_brackets() {
        let spans = parse_spans("(1\n% comment\n)").unwrap();
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Number, 1, 1),
            s(S::Comment, 3, 9),
            s(S::Punctuation, 13, 1)
        ]);
    }

    #[test]
    fn parse_brackets_and_semicolon() {
        let spans = parse_spans("(1), [2]; {3}").unwrap();
        assert_eq!(spans, [
            s(S::Punctuation, 0, 1),
            s(S::Number, 1, 1),
            s(S::Punctuation, 2, 1),
            s(S::Punctuation, 3, 1),
            s(S::Punctuation, 5, 1),
            s(S::Argument, 6, 1),
            s(S::Number, 6, 1),
            s(S::Punctuation, 7, 1),
            s(S::Punctuation, 8, 1),
            s(S::Punctuation, 10, 1),
            s(S::Number, 11, 1),
            s(S::Punctuation, 12, 1),
        ]);
    }

    #[test]
    fn parse_operator_with_at() {
        let spans = parse_spans("@@foo").unwrap();
        assert_eq!(spans, [s(S::Operator, 0, 2), s(S::Ident, 2, 3)]);
    }
}
