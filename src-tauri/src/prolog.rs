use factbook_swipl::Context;
use factbook_swipl::term::Term;
use std::str::Chars;

pub mod predicates {
    use crate::model::EntryId;
    use factbook_swipl::{Atom, Context};
    use factbook_swipl::blob::ScopedBlobData;
    use factbook_swipl::foreign::{Nondet, predicate};
    use std::cell::RefCell;
    use std::collections::btree_map;

    #[derive(ScopedBlobData)]
    pub struct EntryTags<'a>(pub RefCell<btree_map::Iter<'a, EntryId, Vec<factbook_swipl::Record>>>);

    #[predicate(tag(entry_tags, entry, tag) nondet)]
    pub struct Tag;

    impl Nondet for Tag {
        fn init(_: &impl Context) -> Self {
            Self
        }

        fn next(&mut self, _: &impl Context, [entry_tags, entry, tag]: Self::Args<'_>) -> bool {
            log::debug!("tag/3 called");

            let Some(entry_tags_atom) = entry_tags.get::<Atom>() else { return false };
            let Some(entry_tags) = entry_tags_atom.scoped_blob::<EntryTags>() else { return false };

            if let Some((entry_id, tags)) = entry_tags.0.borrow_mut().next() {
                log::debug!("  {entry_id:?}");
                true
            } else {
                false
            }
        }
    }
}

struct Parse<'i, 'c, C: Context> {
    input: Chars<'i>,
    ctx: &'c C,
}

impl<'c, C: Context> Iterator for Parse<'_, 'c, C> {
    type Item = Term<'c>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_was_space = true;

        while let Some(c) = self.input.next() {
            if c == '@'
                && last_was_space
                && let Some(term) = self.consume_term()
            {
                return Some(term);
            }

            last_was_space = c.is_whitespace();
        }

        None
    }
}

impl<'c, C: Context> Parse<'_, 'c, C> {
    fn consume_term(&mut self) -> Option<Term<'c>> {
        let mut term = String::new();
        let mut parens = 0u32;

        while let Some(c) = self.input.next() {
            match c {
                '@' => return None,
                '"' => term.push_str(&self.consume_quoted('"')?),
                '\'' => term.push_str(&self.consume_quoted('\'')?),
                '(' => {
                    term.push('(');
                    parens += 1;
                },
                ')' => {
                    term.push(')');
                    parens -= 1;
                },
                c if c.is_whitespace() && parens == 0 => break,
                c => term.push(c),
            }
        }

        self.ctx
            .new_term()
            .put_parsed(&term)
            .map_err(|e| log::warn!("failed to parse Prolog term: {e:?}"))
            .ok()
    }

    fn consume_quoted(&mut self, delimiter: char) -> Option<String> {
        let mut escape = false;
        let mut quoted = [delimiter].into_iter().collect::<String>();

        for c in self.input.by_ref() {
            match c {
                c if c == delimiter && !escape => {
                    quoted.push(c);
                    return Some(quoted);
                },
                '\\' => {
                    escape = !escape;
                },
                c => {
                    quoted.push(c);
                    escape = false;
                },
            }
        }

        None
    }
}

pub fn parse<'i, 'c: 'i>(
    input: &'i str,
    ctx: &'c impl Context,
) -> impl Iterator<Item = Term<'c>> + 'i {
    Parse {
        input: input.chars(),
        ctx,
    }
}

#[cfg(test)]
mod test {
    use crate::SWIPL_STATE;
    use factbook_swipl::{Context, Session};
    use std::sync::LazyLock;
    use test_log::test;

    pub(crate) static SESSION: LazyLock<Session<'static>> =
        LazyLock::new(|| Session::init(SWIPL_STATE).unwrap());

    fn parse(input: &str, ctx: &impl Context) -> Vec<String> {
        super::parse(input, ctx).map(|t| t.to_string()).collect()
    }

    #[test]
    fn parse_empty() {
        let engine = SESSION.engine();
        assert_eq!(parse("", &engine), [] as [&str; _]);
    }

    #[test]
    fn parse_single_atom() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo", &engine), ["foo"]);
    }

    #[test]
    fn parse_single_atom_with_surrounding_content() {
        let engine = SESSION.engine();
        assert_eq!(parse("bar @foo bar", &engine), ["foo"]);
    }

    #[test]
    fn parse_two_atoms() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo @bar", &engine), ["foo", "bar"]);
    }

    #[test]
    fn parse_two_adjacent_atoms() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo@bar", &engine), [] as [&str; _]);
    }

    #[test]
    fn parse_single_compound() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo(bar)", &engine), ["foo(bar)"]);
    }

    #[test]
    fn parse_two_compound() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo(bar) @bar(baz)", &engine), [
            "foo(bar)", "bar(baz)"
        ]);
    }

    #[test]
    fn parse_quoted() {
        let engine = SESSION.engine();
        assert_eq!(parse("@\"foo bar\"", &engine), ["\"foo bar\""]);
        assert_eq!(parse("@'foo bar'", &engine), ["'foo bar'"]);
    }

    #[test]
    fn parse_compound_with_string_argument_with_spaces() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo(\"foo bar\")", &engine), ["foo(\"foo bar\")"]);
    }

    #[test]
    fn parse_compound_with_numbers() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo(1, 2.3)", &engine), ["foo(1,2.3)"]);
    }

    #[test]
    fn parse_compound_nested() {
        let engine = SESSION.engine();
        assert_eq!(parse("@foo(bar(1, 2), 3, baz(4, 5))", &engine), [
            "foo(bar(1,2),3,baz(4,5))"
        ]);
    }
}
