use std::iter::Peekable;

use crate::{
    ast::AstNode,
    token::{Span, Token},
};

#[derive(Debug, PartialEq)]
pub enum ParseError {
    UnexpectedToken {
        token: Token,
        span: Span,
        expected: ExpectedToken,
    },
    MissingClosingParen,
    UnexpectedEof,
}

#[derive(Debug, PartialEq)]
pub enum ExpectedToken {
    Eof,
    CommaOrRParen,
    ValueOrIdentOrFuncCall,
}

pub fn parse(tokens: Vec<(Token, Span)>) -> Result<AstNode, ParseError> {
    let mut tokens = tokens.into_iter().peekable();
    let ast_node = parse_inner(&mut tokens)?;
    if let Some((token, span)) = tokens.next() {
        return Err(ParseError::UnexpectedToken {
            token,
            span,
            expected: ExpectedToken::Eof,
        });
    }
    Ok(ast_node)
}

fn parse_inner(
    tokens: &mut Peekable<impl Iterator<Item = (Token, Span)>>,
) -> Result<AstNode, ParseError> {
    let Some((token, span)) = tokens.next() else {
        return Err(ParseError::UnexpectedEof);
    };
    match token {
        Token::Int(v) => Ok(AstNode::Int(v)),
        Token::Float(v) => Ok(AstNode::Float(v)),
        Token::String(v) => Ok(AstNode::String(v)),
        Token::Ident(v) => match tokens.peek() {
            Some((Token::LParen, _)) => match {
                tokens.next();
                tokens.peek()
            } {
                Some((Token::RParen, _)) => {
                    tokens.next();
                    Ok(AstNode::Func(v, Vec::new()))
                }
                None => Err(ParseError::MissingClosingParen),
                _ => {
                    let mut args = Vec::new();
                    loop {
                        args.push(parse_inner(tokens)?);
                        match tokens.next() {
                            Some((Token::Comma, _)) => {}
                            Some((Token::RParen, _)) => break Ok(AstNode::Func(v, args)),
                            Some((token, span)) => {
                                return Err(ParseError::UnexpectedToken {
                                    token,
                                    span,
                                    expected: ExpectedToken::CommaOrRParen,
                                });
                            }
                            None => return Err(ParseError::MissingClosingParen),
                        }
                    }
                }
            },
            _ => return Ok(AstNode::Ident(v)),
        },
        _ => Err(ParseError::UnexpectedToken {
            token,
            span,
            expected: ExpectedToken::ValueOrIdentOrFuncCall,
        }),
    }
}

#[cfg(test)]
mod test {
    use std::assert_eq;

    use crate::ast::AstNode;
    use crate::parse::{ExpectedToken, ParseError, parse};
    use crate::token::{Span, Token};

    #[test]
    fn primitive_int() {
        assert_eq!(
            parse(vec![(Token::Int(42), Span { start: 0, len: 2 })]).unwrap(),
            AstNode::Int(42)
        );
    }

    #[test]
    fn primitive_float() {
        assert_eq!(
            parse(vec![(Token::Float(3.14), Span { start: 0, len: 4 })]).unwrap(),
            AstNode::Float(3.14)
        );
    }

    #[test]
    fn primitive_string() {
        assert_eq!(
            parse(vec![(
                Token::String("hello".to_string()),
                Span { start: 0, len: 7 }
            )])
            .unwrap(),
            AstNode::String("hello".to_string())
        );
    }

    #[test]
    fn primitive_ident() {
        assert_eq!(
            parse(vec![(
                Token::Ident("xyz".to_string()),
                Span { start: 0, len: 3 }
            )])
            .unwrap(),
            AstNode::Ident("xyz".to_string())
        );
    }

    #[test]
    fn func_no_args() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::RParen, Span { start: 4, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func("foo".to_string(), Vec::new())
        );
    }

    #[test]
    fn func_one_arg() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Int(42), Span { start: 4, len: 2 }),
                (Token::RParen, Span { start: 6, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func("foo".to_string(), vec![AstNode::Int(42)])
        );
    }

    #[test]
    fn func_two_args() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Int(42), Span { start: 4, len: 2 }),
                (Token::Comma, Span { start: 6, len: 1 }),
                (Token::Float(3.14), Span { start: 7, len: 4 }),
                (Token::RParen, Span { start: 11, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func(
                "foo".to_string(),
                vec![AstNode::Int(42), AstNode::Float(3.14)]
            )
        );
    }

    #[test]
    fn func_three_args() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Int(42), Span { start: 4, len: 2 }),
                (Token::Comma, Span { start: 6, len: 1 }),
                (Token::Float(3.14), Span { start: 7, len: 4 }),
                (Token::Comma, Span { start: 11, len: 1 }),
                (
                    Token::String("hello".to_string()),
                    Span { start: 12, len: 7 }
                ),
                (Token::RParen, Span { start: 19, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func(
                "foo".to_string(),
                vec![
                    AstNode::Int(42),
                    AstNode::Float(3.14),
                    AstNode::String("hello".to_string())
                ]
            )
        );
    }

    #[test]
    fn func_nested() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Ident("bar".to_string()), Span { start: 4, len: 3 }),
                (Token::LParen, Span { start: 7, len: 1 }),
                (Token::Int(42), Span { start: 8, len: 2 }),
                (Token::RParen, Span { start: 10, len: 1 }),
                (Token::RParen, Span { start: 11, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func(
                "foo".to_string(),
                vec![AstNode::Func("bar".to_string(), vec![AstNode::Int(42)])]
            )
        );
    }

    #[test]
    fn func_nested_with_args() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Ident("bar".to_string()), Span { start: 4, len: 3 }),
                (Token::LParen, Span { start: 7, len: 1 }),
                (Token::Int(42), Span { start: 8, len: 2 }),
                (Token::Comma, Span { start: 10, len: 1 }),
                (Token::Float(3.14), Span { start: 11, len: 4 }),
                (Token::RParen, Span { start: 15, len: 1 }),
                (Token::RParen, Span { start: 16, len: 1 }),
            ])
            .unwrap(),
            AstNode::Func(
                "foo".to_string(),
                vec![AstNode::Func(
                    "bar".to_string(),
                    vec![AstNode::Int(42), AstNode::Float(3.14)]
                )]
            )
        );
    }

    #[test]
    fn func_missing_closing_paren() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Int(42), Span { start: 4, len: 2 }),
            ]),
            Err(ParseError::MissingClosingParen)
        );
    }

    #[test]
    fn func_missing_closing_paren_nested() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Ident("bar".to_string()), Span { start: 4, len: 3 }),
                (Token::LParen, Span { start: 7, len: 1 }),
                (Token::Int(42), Span { start: 8, len: 2 }),
                (Token::RParen, Span { start: 10, len: 1 }),
            ]),
            Err(ParseError::MissingClosingParen)
        );
    }

    #[test]
    fn func_missing_closing_paren_nested_with_args() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Ident("bar".to_string()), Span { start: 4, len: 3 }),
                (Token::LParen, Span { start: 7, len: 1 }),
                (Token::Int(42), Span { start: 8, len: 2 }),
                (Token::RParen, Span { start: 10, len: 1 }),
                (Token::Comma, Span { start: 11, len: 1 }),
                (Token::Float(3.14), Span { start: 12, len: 4 }),
            ]),
            Err(ParseError::MissingClosingParen)
        );
    }

    #[test]
    fn func_unexpected_token() {
        assert_eq!(
            parse(vec![
                (Token::Ident("foo".to_string()), Span { start: 0, len: 3 }),
                (Token::LParen, Span { start: 3, len: 1 }),
                (Token::Int(42), Span { start: 4, len: 2 }),
                (Token::Ident("bar".to_string()), Span { start: 6, len: 3 }),
                (Token::RParen, Span { start: 9, len: 1 }),
            ]),
            Err(ParseError::UnexpectedToken {
                token: Token::Ident("bar".to_string()),
                span: Span { start: 6, len: 3 },
                expected: ExpectedToken::CommaOrRParen,
            })
        );
    }
}
