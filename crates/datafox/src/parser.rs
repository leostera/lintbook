use crate::ast::{Atom, Clause, Query};
use crate::diagnostic::{Diagnostic, Span};
use crate::error::{Error, Result};
use crate::{Term, Value};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TokenKind {
    Identifier(String),
    Integer(i64),
    String(String),
    Quoted(String),
    Underscore,
    Bang,
    Equal,
    Comma,
    Semicolon,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    LeftParen,
    RightParen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Token {
    kind: TokenKind,
    span: Span,
}

pub fn parse_query(source: &str) -> Result<Query> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        source,
        tokens,
        cursor: 0,
    };
    parser.parse_query()
}

pub fn parse_queries(source: &str) -> Result<Vec<Query>> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        source,
        tokens,
        cursor: 0,
    };
    parser.parse_queries()
}

fn lex(source: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = source.char_indices().peekable();

    while let Some((start, ch)) = chars.next() {
        if ch.is_whitespace() {
            continue;
        }

        let token = match ch {
            '!' => Token {
                kind: TokenKind::Bang,
                span: Span::new(start, start + ch.len_utf8()),
            },
            '=' => Token {
                kind: TokenKind::Equal,
                span: Span::new(start, start + ch.len_utf8()),
            },
            ',' => Token {
                kind: TokenKind::Comma,
                span: Span::new(start, start + ch.len_utf8()),
            },
            ';' => Token {
                kind: TokenKind::Semicolon,
                span: Span::new(start, start + ch.len_utf8()),
            },
            '<' => {
                if let Some((index, '=')) = chars.peek().copied() {
                    chars.next();
                    Token {
                        kind: TokenKind::LessThanOrEqual,
                        span: Span::new(start, index + '='.len_utf8()),
                    }
                } else {
                    Token {
                        kind: TokenKind::LessThan,
                        span: Span::new(start, start + ch.len_utf8()),
                    }
                }
            }
            '>' => {
                if let Some((index, '=')) = chars.peek().copied() {
                    chars.next();
                    Token {
                        kind: TokenKind::GreaterThanOrEqual,
                        span: Span::new(start, index + '='.len_utf8()),
                    }
                } else {
                    Token {
                        kind: TokenKind::GreaterThan,
                        span: Span::new(start, start + ch.len_utf8()),
                    }
                }
            }
            '(' => Token {
                kind: TokenKind::LeftParen,
                span: Span::new(start, start + ch.len_utf8()),
            },
            ')' => Token {
                kind: TokenKind::RightParen,
                span: Span::new(start, start + ch.len_utf8()),
            },
            '_' => Token {
                kind: TokenKind::Underscore,
                span: Span::new(start, start + ch.len_utf8()),
            },
            '"' => {
                let mut end = start + ch.len_utf8();
                let mut value = String::new();
                let mut closed = false;

                while let Some((index, next)) = chars.next() {
                    end = index + next.len_utf8();
                    if next == '"' {
                        closed = true;
                        break;
                    }
                    value.push(next);
                }

                if !closed {
                    return Err(Error::Parse {
                        diagnostics: vec![
                            Diagnostic::new("unterminated string literal")
                                .with_span(Span::new(start, end)),
                        ],
                    });
                }

                Token {
                    kind: TokenKind::String(value),
                    span: Span::new(start, end),
                }
            }
            '\'' => {
                let mut end = start + ch.len_utf8();
                let mut value = String::new();
                let mut closed = false;

                while let Some((index, next)) = chars.next() {
                    end = index + next.len_utf8();
                    if next == '\'' {
                        closed = true;
                        break;
                    }
                    value.push(next);
                }

                if !closed {
                    return Err(Error::Parse {
                        diagnostics: vec![
                            Diagnostic::new("unterminated quoted identifier")
                                .with_span(Span::new(start, end)),
                        ],
                    });
                }

                Token {
                    kind: TokenKind::Quoted(value),
                    span: Span::new(start, end),
                }
            }
            '-' | '0'..='9' => {
                let mut end = start + ch.len_utf8();
                let mut text = String::from(ch);

                while let Some((index, next)) = chars.peek().copied() {
                    if !next.is_ascii_digit() {
                        break;
                    }
                    chars.next();
                    text.push(next);
                    end = index + next.len_utf8();
                }

                match text.parse::<i64>() {
                    Ok(value) => Token {
                        kind: TokenKind::Integer(value),
                        span: Span::new(start, end),
                    },
                    Err(_) => {
                        return Err(Error::Parse {
                            diagnostics: vec![
                                Diagnostic::new("invalid integer literal")
                                    .with_span(Span::new(start, end))
                                    .with_found(text),
                            ],
                        });
                    }
                }
            }
            ch if is_identifier_start(ch) => {
                let mut end = start + ch.len_utf8();
                let mut text = String::from(ch);

                while let Some((index, next)) = chars.peek().copied() {
                    if !is_identifier_continue(next) {
                        break;
                    }
                    chars.next();
                    text.push(next);
                    end = index + next.len_utf8();
                }

                Token {
                    kind: TokenKind::Identifier(text),
                    span: Span::new(start, end),
                }
            }
            other => {
                return Err(Error::Parse {
                    diagnostics: vec![
                        Diagnostic::new("unexpected character")
                            .with_span(Span::new(start, start + other.len_utf8()))
                            .with_found(other.to_string()),
                    ],
                });
            }
        };

        tokens.push(token);
    }

    Ok(tokens)
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_ascii_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, ':' | '_' | '-' | '?')
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    cursor: usize,
}

impl<'a> Parser<'a> {
    fn parse_query(&mut self) -> Result<Query> {
        if self.tokens.is_empty() {
            return Err(Error::Parse {
                diagnostics: vec![Diagnostic::new("expected a query, found end of input")],
            });
        }

        let query = self.parse_query_body()?;

        if let Some(token) = self.peek() {
            return Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new("unexpected trailing input")
                        .with_span(token.span)
                        .with_found(self.token_text(token)),
                ],
            });
        }

        Ok(query)
    }

    fn parse_queries(&mut self) -> Result<Vec<Query>> {
        if self.tokens.is_empty() {
            return Err(Error::Parse {
                diagnostics: vec![Diagnostic::new("expected a query, found end of input")],
            });
        }

        let mut queries = Vec::new();
        loop {
            queries.push(self.parse_query_body()?);

            if self
                .match_kind(|kind| matches!(kind, TokenKind::Semicolon))
                .is_none()
            {
                break;
            }

            if self.peek().is_none() {
                break;
            }
        }

        if let Some(token) = self.peek() {
            return Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new("unexpected trailing input")
                        .with_span(token.span)
                        .with_found(self.token_text(token)),
                ],
            });
        }

        Ok(queries)
    }

    fn parse_query_body(&mut self) -> Result<Query> {
        if let Some(token) = self
            .peek()
            .filter(|token| matches!(token.kind, TokenKind::Semicolon))
        {
            return Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new("expected a query")
                        .with_span(token.span)
                        .with_found(self.token_text(token)),
                ],
            });
        }

        let mut clauses = vec![self.parse_clause()?];
        while self
            .match_kind(|kind| matches!(kind, TokenKind::Comma))
            .is_some()
        {
            clauses.push(self.parse_clause()?);
        }

        if clauses.len() == 1 {
            match clauses.pop().expect("one clause") {
                Clause::Atom(atom) => Ok(Query::single(atom)),
                _ => Err(Error::InvalidSingleQueryShape),
            }
        } else {
            Query::multi(clauses)
        }
    }

    fn parse_clause(&mut self) -> Result<Clause> {
        if self
            .match_kind(|kind| matches!(kind, TokenKind::Bang))
            .is_some()
        {
            return Ok(Clause::negated(self.parse_atom()?));
        }

        if self.is_infix_builtin_start() {
            return self.parse_infix_builtin();
        }

        let atom = self.parse_atom()?;
        if is_named_builtin(&atom.predicate) {
            Ok(Clause::builtin(atom.predicate, atom.args))
        } else {
            Ok(Clause::atom(atom))
        }
    }

    fn parse_infix_builtin(&mut self) -> Result<Clause> {
        let left = self.parse_term()?;
        let operator = self.next().ok_or_else(|| Error::Parse {
            diagnostics: vec![Diagnostic::new(
                "expected an infix operator, found end of input",
            )],
        })?;
        let name = match operator.kind {
            TokenKind::GreaterThan => "gt",
            TokenKind::GreaterThanOrEqual => "gte",
            TokenKind::LessThan => "lt",
            TokenKind::LessThanOrEqual => "lte",
            TokenKind::Equal => "eq",
            _ => {
                return Err(Error::Parse {
                    diagnostics: vec![
                        Diagnostic::new("expected an infix operator")
                            .with_span(operator.span)
                            .with_found(self.token_text(&operator)),
                    ],
                });
            }
        };
        let right = self.parse_term()?;

        Ok(Clause::builtin(name, vec![left, right]))
    }

    fn parse_atom(&mut self) -> Result<Atom> {
        let predicate = self.parse_predicate()?;
        self.expect_kind(
            |kind| matches!(kind, TokenKind::LeftParen),
            "expected `(` after predicate",
        )?;

        let mut args = Vec::new();
        if self
            .match_kind(|kind| matches!(kind, TokenKind::RightParen))
            .is_none()
        {
            loop {
                args.push(self.parse_term()?);

                if self
                    .match_kind(|kind| matches!(kind, TokenKind::RightParen))
                    .is_some()
                {
                    break;
                }

                self.expect_kind(
                    |kind| matches!(kind, TokenKind::Comma),
                    "expected `,` or `)` after term",
                )?;
            }
        }

        Atom::new(predicate, args)
    }

    fn parse_term(&mut self) -> Result<Term> {
        let token = self.next().ok_or_else(|| Error::Parse {
            diagnostics: vec![Diagnostic::new("expected a term, found end of input")],
        })?;

        match token.kind {
            TokenKind::Underscore => Ok(Term::wildcard()),
            TokenKind::String(value) | TokenKind::Quoted(value) => {
                Ok(Term::constant(Value::string(value)))
            }
            TokenKind::Integer(value) => Ok(Term::constant(Value::integer(value))),
            TokenKind::Identifier(value) => {
                if value
                    .chars()
                    .next()
                    .is_some_and(|ch| ch.is_ascii_uppercase())
                {
                    Term::variable(value)
                } else {
                    Ok(Term::constant(Value::string(value)))
                }
            }
            _ => Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new("expected a term")
                        .with_span(token.span)
                        .with_found(self.token_text(&token)),
                ],
            }),
        }
    }

    fn parse_predicate(&mut self) -> Result<String> {
        let token = self.next().ok_or_else(|| Error::Parse {
            diagnostics: vec![Diagnostic::new(
                "expected a predicate name, found end of input",
            )],
        })?;

        match token.kind {
            TokenKind::Identifier(value) => Ok(value),
            TokenKind::Quoted(value) => Ok(value),
            _ => Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new("expected a predicate name")
                        .with_span(token.span)
                        .with_found(self.token_text(&token)),
                ],
            }),
        }
    }

    fn expect_kind(
        &mut self,
        predicate: impl FnOnce(&TokenKind) -> bool,
        message: &str,
    ) -> Result<Token> {
        let token = self.next().ok_or_else(|| Error::Parse {
            diagnostics: vec![Diagnostic::new(message)],
        })?;

        if predicate(&token.kind) {
            Ok(token)
        } else {
            Err(Error::Parse {
                diagnostics: vec![
                    Diagnostic::new(message)
                        .with_span(token.span)
                        .with_found(self.token_text(&token)),
                ],
            })
        }
    }

    fn match_kind(&mut self, predicate: impl FnOnce(&TokenKind) -> bool) -> Option<Token> {
        let token = self.peek()?.clone();
        if predicate(&token.kind) {
            self.cursor += 1;
            Some(token)
        } else {
            None
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.cursor)
    }

    fn peek_n(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.cursor + offset)
    }

    fn next(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.cursor).cloned()?;
        self.cursor += 1;
        Some(token)
    }

    fn is_infix_builtin_start(&self) -> bool {
        let Some(left) = self.peek() else {
            return false;
        };
        if !matches!(
            left.kind,
            TokenKind::Identifier(_)
                | TokenKind::Integer(_)
                | TokenKind::String(_)
                | TokenKind::Quoted(_)
                | TokenKind::Underscore
        ) {
            return false;
        }

        self.peek_n(1).is_some_and(|token| {
            matches!(
                token.kind,
                TokenKind::GreaterThan
                    | TokenKind::GreaterThanOrEqual
                    | TokenKind::LessThan
                    | TokenKind::LessThanOrEqual
                    | TokenKind::Equal
            )
        })
    }

    fn token_text(&self, token: &Token) -> String {
        self.source[token.span.start..token.span.end].to_string()
    }
}

fn is_named_builtin(name: &str) -> bool {
    matches!(
        name,
        "startsWith"
            | "endsWith"
            | "contains"
            | "matchesRegex"
            | "notStartsWith"
            | "notEndsWith"
            | "notContains"
            | "notMatchesRegex"
            | "before"
            | "after"
    )
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use crate::{
        Atom, Clause, Diagnostic, Error, Query, Span, Term, Value, parse_queries, parse_query,
    };

    #[test]
    fn parses_single_goal_queries() {
        let query = parse_query("spotify:displayName(Album, \"2112\")").expect("query");

        assert_eq!(
            query,
            Query::single(
                Atom::new(
                    "spotify:displayName",
                    vec![
                        Term::variable("Album").expect("variable"),
                        Term::constant(Value::string("2112")),
                    ],
                )
                .expect("atom"),
            )
        );
    }

    #[test]
    fn parses_multi_goal_queries() {
        let query =
            parse_query("spotify:byArtist(Album, Artist), spotify:displayName(Artist, \"Rush\")")
                .expect("query");

        assert_eq!(
            query,
            Query::multi(vec![
                Clause::atom(
                    Atom::new(
                        "spotify:byArtist",
                        vec![
                            Term::variable("Album").expect("variable"),
                            Term::variable("Artist").expect("variable"),
                        ],
                    )
                    .expect("atom"),
                ),
                Clause::atom(
                    Atom::new(
                        "spotify:displayName",
                        vec![
                            Term::variable("Artist").expect("variable"),
                            Term::constant(Value::string("Rush")),
                        ],
                    )
                    .expect("atom"),
                ),
            ])
            .expect("multi"),
        );
    }

    #[test]
    fn parses_semicolon_separated_query_sets() {
        let queries = parse_queries(r#"node(Node, "call"); text(Node, "dbg!")"#).expect("queries");

        assert_eq!(queries.len(), 2);
        assert!(matches!(&queries[0], Query::Single(_)));
        assert!(matches!(&queries[1], Query::Single(_)));
    }

    #[test]
    fn parses_negated_atoms() {
        let query = parse_query("!edge(X, Y), edge(Y, Z)").expect("query");

        assert_eq!(
            query,
            Query::multi(vec![
                Clause::negated(
                    Atom::new(
                        "edge",
                        vec![
                            Term::variable("X").expect("variable"),
                            Term::variable("Y").expect("variable"),
                        ],
                    )
                    .expect("atom"),
                ),
                Clause::atom(
                    Atom::new(
                        "edge",
                        vec![
                            Term::variable("Y").expect("variable"),
                            Term::variable("Z").expect("variable"),
                        ],
                    )
                    .expect("atom"),
                ),
            ])
            .expect("multi"),
        );
    }

    #[test]
    fn parses_infix_comparison_builtins() {
        let query = parse_query(
            "gcal:startedAt(Event, Start), Start < \"2026-01-01 21:12:00\", Start > \"2026-01-02\"",
        )
        .expect("query");

        assert_eq!(
            query,
            Query::multi(vec![
                Clause::atom(
                    Atom::new(
                        "gcal:startedAt",
                        vec![
                            Term::variable("Event").expect("variable"),
                            Term::variable("Start").expect("variable"),
                        ],
                    )
                    .expect("atom"),
                ),
                Clause::builtin(
                    "lt",
                    vec![
                        Term::variable("Start").expect("variable"),
                        Term::constant(Value::string("2026-01-01 21:12:00")),
                    ],
                ),
                Clause::builtin(
                    "gt",
                    vec![
                        Term::variable("Start").expect("variable"),
                        Term::constant(Value::string("2026-01-02")),
                    ],
                ),
            ])
            .expect("multi"),
        );
    }

    #[test]
    fn parses_infix_comparison_operator_variants() {
        let query = parse_query("A <= B, B >= C, C = D").expect("query");

        assert_eq!(
            query,
            Query::multi(vec![
                Clause::builtin(
                    "lte",
                    vec![
                        Term::variable("A").expect("variable"),
                        Term::variable("B").expect("variable"),
                    ],
                ),
                Clause::builtin(
                    "gte",
                    vec![
                        Term::variable("B").expect("variable"),
                        Term::variable("C").expect("variable"),
                    ],
                ),
                Clause::builtin(
                    "eq",
                    vec![
                        Term::variable("C").expect("variable"),
                        Term::variable("D").expect("variable"),
                    ],
                ),
            ])
            .expect("multi"),
        );
    }

    #[test]
    fn parses_named_builtin_clauses() {
        let query = parse_query(
            "edge(X, Y), startsWith(Name, \"Leo\"), notContains(Name, \"x\"), before(Start, End)",
        )
        .expect("query");

        assert_eq!(
            query,
            Query::multi(vec![
                Clause::atom(
                    Atom::new(
                        "edge",
                        vec![
                            Term::variable("X").expect("variable"),
                            Term::variable("Y").expect("variable"),
                        ],
                    )
                    .expect("atom"),
                ),
                Clause::builtin(
                    "startsWith",
                    vec![
                        Term::variable("Name").expect("variable"),
                        Term::constant(Value::string("Leo")),
                    ],
                ),
                Clause::builtin(
                    "notContains",
                    vec![
                        Term::variable("Name").expect("variable"),
                        Term::constant(Value::string("x")),
                    ],
                ),
                Clause::builtin(
                    "before",
                    vec![
                        Term::variable("Start").expect("variable"),
                        Term::variable("End").expect("variable"),
                    ],
                ),
            ])
            .expect("multi"),
        );
    }

    #[test]
    fn reports_contextual_diagnostics_for_missing_right_paren() {
        let error = parse_query("edge(X, Y").expect_err("parse error");

        assert_eq!(
            error,
            Error::Parse {
                diagnostics: vec![Diagnostic::new("expected `,` or `)` after term")],
            }
        );
    }

    #[test]
    fn reports_contextual_diagnostics_for_unexpected_tokens() {
        let error = parse_query("edge(X) ???").expect_err("parse error");

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(diagnostics.len(), 1);
                assert_eq!(diagnostics[0].message, "unexpected character");
                assert_eq!(diagnostics[0].found.as_deref(), Some("?"));
                assert_eq!(diagnostics[0].span, Some(crate::Span::new(8, 9)));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn reports_contextual_diagnostics_for_missing_left_paren_after_predicate() {
        let error = parse_query("edge X)").expect_err("parse error");

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(
                    diagnostics,
                    vec![
                        Diagnostic::new("expected `(` after predicate")
                            .with_span(Span::new(5, 6))
                            .with_found("X"),
                    ]
                );
                assert_eq!(
                    diagnostics[0].to_string(),
                    "expected `(` after predicate at 5..6 (found `X`)"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn reports_contextual_diagnostics_for_unterminated_strings() {
        let error = parse_query("edge(\"hello)").expect_err("parse error");

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(
                    diagnostics,
                    vec![
                        Diagnostic::new("unterminated string literal").with_span(Span::new(5, 12))
                    ]
                );
                assert_eq!(
                    diagnostics[0].to_string(),
                    "unterminated string literal at 5..12"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn reports_contextual_diagnostics_for_unexpected_trailing_input() {
        let error = parse_query("edge(X) trailing").expect_err("parse error");

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(
                    diagnostics,
                    vec![
                        Diagnostic::new("unexpected trailing input")
                            .with_span(Span::new(8, 16))
                            .with_found("trailing"),
                    ]
                );
                assert_eq!(
                    diagnostics[0].to_string(),
                    "unexpected trailing input at 8..16 (found `trailing`)"
                );
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn parses_single_quoted_predicate() {
        let query = parse_query("'local://schema/name'(Entity, Value)").expect("query");

        assert_eq!(
            query,
            Query::single(
                Atom::new(
                    "local://schema/name",
                    vec![
                        Term::variable("Entity").expect("variable"),
                        Term::variable("Value").expect("variable"),
                    ],
                )
                .expect("atom"),
            )
        );
    }

    #[test]
    fn parses_quoted_predicate_with_special_chars() {
        let query = parse_query("'http://example.org/pred#frag?x=1'(E, V)").expect("query");

        assert_eq!(
            query,
            Query::single(
                Atom::new(
                    "http://example.org/pred#frag?x=1",
                    vec![
                        Term::variable("E").expect("variable"),
                        Term::variable("V").expect("variable"),
                    ],
                )
                .expect("atom"),
            )
        );
    }

    #[test]
    fn unquoted_predicate_still_works() {
        let query = parse_query("displayName(Album, Name)").expect("query");

        assert_eq!(
            query,
            Query::single(
                Atom::new(
                    "displayName",
                    vec![
                        Term::variable("Album").expect("variable"),
                        Term::variable("Name").expect("variable"),
                    ],
                )
                .expect("atom"),
            )
        );
    }

    #[test]
    fn reports_diagnostics_for_unterminated_quoted_predicate() {
        let error = parse_query("'local:schema:name(Entity, Value)").expect_err("parse error");

        match error {
            Error::Parse { diagnostics } => {
                assert_eq!(
                    diagnostics.len(),
                    1,
                    "expected 1 diagnostic, got {}",
                    diagnostics.len()
                );
                assert_eq!(diagnostics[0].message, "unterminated quoted identifier");
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }

    proptest! {
        #[test]
        fn parse_query_never_panics_on_arbitrary_utf8(source in "\\PC{0,512}") {
            let _ = parse_query(&source);
        }

        #[test]
        fn parse_query_sets_never_panic_on_arbitrary_utf8(source in "\\PC{0,512}") {
            let _ = parse_queries(&source);
        }
    }
}
