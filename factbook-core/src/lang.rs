use factbook_swipl::Context;
use factbook_swipl::term::Term;
use itertools::Itertools;
use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "entry.pest"]
struct EntryParser;

pub struct ParseResult<'c> {
    pub tags: Vec<Term<'c>>,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    /// Character offset from the start of the input string
    pub start: usize,
    /// Length of the token in characters
    pub len: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Punctuation,
    Ident,
    Number,
    String,
    Variable,
    Operator,
}

impl Token {
    fn from_pair(kind: TokenKind, pair: pest::iterators::Pair<Rule>) -> Self {
        Self {
            kind,
            start: pair.line_col().1 - 1,
            len: pair.as_str().chars().count(),
        }
    }
}

pub fn parse<'c>(input: &str, ctx: &'c impl Context) -> ParseResult<'c> {
    let mut tags = Vec::new();
    let mut tokens = Vec::new();

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

                if let Ok(tag) = ctx
                    .new_term()
                    .put_parsed(tag.as_str())
                    .map_err(|e| log::warn!("failed to parse tag: {e:?}"))
                {
                    tags.push(tag);
                }

                for pair in pair.into_inner().flatten() {
                    use TokenKind as T;

                    match pair.as_rule() {
                        Rule::at | Rule::lparen | Rule::rparen | Rule::comma => {
                            tokens.push(Token::from_pair(T::Punctuation, pair))
                        },
                        Rule::ident | Rule::quoted => tokens.push(Token::from_pair(T::Ident, pair)),
                        Rule::string => tokens.push(Token::from_pair(T::String, pair)),
                        Rule::number => tokens.push(Token::from_pair(T::Number, pair)),
                        Rule::variable => tokens.push(Token::from_pair(T::Variable, pair)),
                        Rule::operator | Rule::operator_ex => {
                            tokens.push(Token::from_pair(T::Operator, pair))
                        },
                        _ => (),
                    }
                }
            },
            Rule::EOI => (),
            rule => panic!("parser returned unexpected rule: {rule:?}"),
        }
    }

    ParseResult { tags, tokens }
}

#[cfg(test)]
mod test {
    use super::{Token, TokenKind as T};
    use factbook_swipl::Context;
    use pretty_assertions::assert_eq;
    use test_log::test;

    fn t(kind: T, start: usize, len: usize) -> Token {
        Token { kind, start, len }
    }

    fn parse(input: &str, ctx: &impl Context) -> (Vec<String>, Vec<Token>) {
        let result = super::parse(input, ctx);
        (
            result.tags.into_iter().map(|t| t.to_string()).collect(),
            result.tokens,
        )
    }

    #[test]
    fn parse_empty() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("", &engine);
        assert_eq!(tags, [] as [&str; _]);
        assert_eq!(tokens, [] as [Token; _]);
    }

    #[test]
    fn parse_single_atom() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo", &engine);
        assert_eq!(tags, ["foo"]);
        assert_eq!(tokens, [t(T::Punctuation, 0, 1), t(T::Ident, 1, 3)]);
    }

    #[test]
    fn parse_single_atom_with_surrounding_content() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("bar @foo bar", &engine);
        assert_eq!(tags, ["foo"]);
        assert_eq!(tokens, [t(T::Punctuation, 4, 1), t(T::Ident, 5, 3)]);
    }

    #[test]
    fn parse_two_atoms() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo @x", &engine);
        assert_eq!(tags, ["foo", "x"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Punctuation, 5, 1),
            t(T::Ident, 6, 1)
        ]);
    }

    #[test]
    fn parse_two_adjacent_atoms() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo@bar", &engine);
        assert_eq!(tags, [] as [&str; _]);
        assert_eq!(tokens, [] as [Token; _]);
    }

    #[test]
    fn parse_single_compound() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo(bar)", &engine);
        assert_eq!(tags, ["foo(bar)"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Punctuation, 4, 1),
            t(T::Ident, 5, 3),
            t(T::Punctuation, 8, 1)
        ]);
    }

    #[test]
    fn parse_two_compound() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo(bar) @bar( baz , 1 )", &engine);
        assert_eq!(tags, ["foo(bar)", "bar(baz,1)"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Punctuation, 4, 1),
            t(T::Ident, 5, 3),
            t(T::Punctuation, 8, 1),
            t(T::Punctuation, 10, 1),
            t(T::Ident, 11, 3),
            t(T::Punctuation, 14, 1),
            t(T::Ident, 16, 3),
            t(T::Punctuation, 20, 1),
            t(T::Number, 22, 1),
            t(T::Punctuation, 24, 1),
        ]);
    }

    #[test]
    fn parse_quoted() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@'foo bar'", &engine);
        assert_eq!(tags, ["'foo bar'"]);
        assert_eq!(tokens, [t(T::Punctuation, 0, 1), t(T::Ident, 1, 9)]);
    }

    #[test]
    fn parse_string() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@\"foo bar\"", &engine);
        assert_eq!(tags, ["\"foo bar\""]);
        assert_eq!(tokens, [t(T::Punctuation, 0, 1), t(T::String, 1, 9)]);
    }

    #[test]
    fn parse_compound_with_string_argument_with_spaces() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo(\"foo bar\")", &engine);
        assert_eq!(tags, ["foo(\"foo bar\")"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Punctuation, 4, 1),
            t(T::String, 5, 9),
            t(T::Punctuation, 14, 1)
        ]);
    }

    #[test]
    fn parse_number() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@42", &engine);
        assert_eq!(tags, ["42"]);
        assert_eq!(tokens, [t(T::Punctuation, 0, 1), t(T::Number, 1, 2)]);
    }

    #[test]
    fn parse_compound_with_numbers() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@foo(42, 12.34)", &engine);
        assert_eq!(tags, ["foo(42,12.34)"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Punctuation, 4, 1),
            t(T::Number, 5, 2),
            t(T::Punctuation, 7, 1),
            t(T::Number, 9, 5),
            t(T::Punctuation, 14, 1)
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
        let (tags, tokens) = parse("@Var @X @_ @_foo", &engine);
        assert_eq!(tags, ["_", "_", "_", "_"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Variable, 1, 3),
            t(T::Punctuation, 5, 1),
            t(T::Variable, 6, 1),
            t(T::Punctuation, 8, 1),
            t(T::Variable, 9, 1),
            t(T::Punctuation, 11, 1),
            t(T::Variable, 12, 4)
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
        let (tags, tokens) = parse("@foo/bar @foo-bar-baz @-42 @+42", &engine);
        assert_eq!(tags, ["foo/bar", "foo-bar-baz", "-42", "+42"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Ident, 1, 3),
            t(T::Operator, 4, 1),
            t(T::Ident, 5, 3),
            t(T::Punctuation, 9, 1),
            t(T::Ident, 10, 3),
            t(T::Operator, 13, 1),
            t(T::Ident, 14, 3),
            t(T::Operator, 17, 1),
            t(T::Ident, 18, 3),
            t(T::Punctuation, 22, 1),
            t(T::Operator, 23, 1),
            t(T::Number, 24, 2),
            t(T::Punctuation, 27, 1),
            t(T::Operator, 28, 1),
            t(T::Number, 29, 2),
        ]);
    }

    #[test]
    fn parse_special_operators() {
        let engine = crate::test::SESSION.0.engine();
        let (tags, tokens) = parse("@(1,2) @(1=@=2)", &engine);
        assert_eq!(tags, ["1,2", "1=@=2"]);
        assert_eq!(tokens, [
            t(T::Punctuation, 0, 1),
            t(T::Punctuation, 1, 1),
            t(T::Number, 2, 1),
            t(T::Operator, 3, 1),
            t(T::Number, 4, 1),
            t(T::Punctuation, 5, 1),
            t(T::Punctuation, 7, 1),
            t(T::Punctuation, 8, 1),
            t(T::Number, 9, 1),
            t(T::Operator, 10, 3),
            t(T::Number, 13, 1),
            t(T::Punctuation, 14, 1),
        ]);
    }
}
