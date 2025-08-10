use std::{fmt::Debug, iter::Peekable};

use crate::lexer::{Token, TokenStream};

#[derive(Debug)]
pub struct KeyValuePair(pub String, pub Ast);

#[derive(Debug)]
pub enum Ast {
    Object(Vec<KeyValuePair>),
    Array(Vec<Ast>),
    Number(f64),
    Bool(bool),
    Null,
    String(String),
}

pub(crate) struct InvalidTokenStreamError {
    pub(crate) message: String,
}

impl InvalidTokenStreamError {
    fn new(message: &str) -> InvalidTokenStreamError {
        InvalidTokenStreamError {
            message: message.to_string(),
        }
    }
    fn unexpected_end_of_tokens(location: &str) -> InvalidTokenStreamError {
        InvalidTokenStreamError {
            message: format!("unexpected end of tokens in {}", location),
        }
    }

    fn unexpected_token<T: Debug>(actual: T, expected: &str) -> InvalidTokenStreamError {
        InvalidTokenStreamError {
            message: format!("unexpected token: {:?} ({} is expected) ", actual, expected),
        }
    }
}

fn parse_object<T: Iterator<Item = Token>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, InvalidTokenStreamError> {
    let mut content = Vec::new();

    loop {
        let Some(next) = iter.peek() else {
            return Err(InvalidTokenStreamError::unexpected_end_of_tokens("object"));
        };

        match next {
            Token::BraceClose => {
                return Ok(Ast::Object(content));
            }
            Token::String(key) => {
                let key = key.clone();
                iter.next();
                let colon = iter.next();
                if !matches!(colon, Some(Token::Colon)) {
                    return Err(InvalidTokenStreamError {
                        message: format!("unexpected token: {:?} (colon is expected) ", colon),
                    });
                }

                let ast_node = parse_unknown(iter)?;

                content.push(KeyValuePair(key, ast_node));
                let next_token = iter.peek();
                match next_token {
                    Some(Token::Comma) => {
                        iter.next();
                    }
                    Some(Token::BraceClose) => {
                        continue;
                    }
                    _ => {
                        return Err(InvalidTokenStreamError::unexpected_token(
                            colon,
                            "comma or BraceClose",
                        ));
                    }
                }
            }
            _ => {
                return Err(InvalidTokenStreamError::unexpected_token(next, "String"));
            }
        }
    }
}

fn parse_array<T: Iterator<Item = Token>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, InvalidTokenStreamError> {
    let mut content = Vec::new();

    loop {
        let Some(next_token) = iter.peek() else {
            return Err(InvalidTokenStreamError::unexpected_end_of_tokens("array"));
        };

        match next_token {
            Token::BracketClose => {
                return Ok(Ast::Array(content));
            }
            _ => {
                let ast_node = parse_unknown(iter)?;

                content.push(ast_node);
                let Some(next_token) = iter.peek() else {
                    return Err(InvalidTokenStreamError::unexpected_end_of_tokens("array"));
                };

                match next_token {
                    Token::Comma => {
                        iter.next();
                    }
                    Token::BracketClose => {
                        continue;
                    }
                    _ => {
                        return Err(InvalidTokenStreamError::unexpected_token(
                            next_token,
                            "BracketClose or Comma",
                        ));
                    }
                }
            }
        }
    }
}

fn parse_number(number_str: &String) -> Result<f64, InvalidTokenStreamError> {
    let result = number_str.parse::<f64>();
    return result.map_err(|_| InvalidTokenStreamError::new("invalid fp number"));
}

fn parse_unknown<T: Iterator<Item = Token>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, InvalidTokenStreamError> {
    let Some(next_token) = iter.next() else {
        return Err(InvalidTokenStreamError {
            message: "unexpected token stream end".to_string(),
        });
    };
    let ast_node = match next_token {
        Token::Bool(bool) => Ast::Bool(bool),
        Token::Comma => panic!("unexpected comma"),
        Token::Null => Ast::Null,
        Token::String(str) => Ast::String(str.to_owned()),
        Token::Number(value) => Ast::Number(parse_number(&value)?),
        Token::BraceOpen => {
            let obj = parse_object(iter)?;
            let next_token = iter.next();
            if !matches!(next_token, Some(Token::BraceClose)) {
                return Err(InvalidTokenStreamError {
                    message: format!("brace close expected, but got {:?}", next_token),
                });
            }

            obj
        }
        Token::BracketOpen => {
            let array = parse_array(iter)?;

            let next_token = iter.next();

            if !matches!(next_token, Some(Token::BracketClose)) {
                return Err(InvalidTokenStreamError {
                    message: format!("bracket close expected, but got {:?}", next_token),
                });
            }

            array
        }
        Token::BraceClose | Token::BracketClose | Token::Colon => {
            return Err(InvalidTokenStreamError {
                message: format!("invariant {:?} is unexpected", next_token),
            });
        }
    };

    Ok(ast_node)
}

pub fn build_ast(token_stream: TokenStream) -> Result<Ast, InvalidTokenStreamError> {
    let mut iter = token_stream.tokens.into_iter().peekable();

    parse_unknown(&mut iter)
}
