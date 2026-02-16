use std::{fmt::Debug, iter::Peekable};

use crate::{labels::Labels, with_label_expr};

use super::lexer::Token;

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

#[derive(Debug, Clone)]
pub struct ParseError {
    pub message: String,
}

impl ParseError {
    pub(crate) fn from_string(message: String) -> ParseError {
        ParseError { message: message }
    }

    pub(crate) fn new(message: &str) -> Self {
        ParseError {
            message: message.to_string(),
        }
    }

    pub(crate) fn unexpected_end_of_tokens(location: &str) -> ParseError {
        ParseError {
            message: format!("unexpected end of tokens in {}", location),
        }
    }

    pub(crate) fn unexpected_token<T: Debug>(actual: T, expected: &str) -> ParseError {
        ParseError {
            message: format!("unexpected token: {:?} ({} is expected) ", actual, expected),
        }
    }
}

fn parse_object<T: Iterator<Item = Result<Token, ParseError>>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, ParseError> {
    let mut content = Vec::new();

    loop {
        let Some(next) = iter.peek() else {
            return Err(ParseError::unexpected_end_of_tokens("object"));
        };
        let next = match next {
            Ok(it) => it,
            Err(err) => return Err(err.clone()),
        };

        match next {
            Token::BraceClose => {
                return Ok(Ast::Object(content));
            }
            Token::String(key) => {
                let key = key.clone();
                iter.next();
                match iter.next() {
                    Some(Ok(Token::Colon)) => {}
                    Some(Ok(token)) => {
                        return Err(ParseError {
                            message: format!("unexpected token: {:?} (colon is expected) ", token),
                        });
                    }
                    Some(Err(err)) => return Err(err.clone()),
                    None => {
                        return Err(ParseError::unexpected_end_of_tokens("object"));
                    }
                };

                let ast_node = parse_unknown(iter)?;

                content.push(KeyValuePair(key, ast_node));
                match iter.peek() {
                    Some(Ok(Token::Comma)) => {
                        iter.next();
                    }
                    Some(Ok(Token::BraceClose)) => {
                        continue;
                    }
                    Some(Ok(token)) => {
                        return Err(ParseError::unexpected_token(token, "comma or BraceClose"));
                    }
                    Some(Err(_)) => return Err(iter.next().unwrap().unwrap_err()),
                    None => return Err(ParseError::unexpected_end_of_tokens("object")),
                }
            }
            _ => {
                return Err(ParseError::unexpected_token(next, "String"));
            }
        }
    }
}

fn parse_array<T: Iterator<Item = Result<Token, ParseError>>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, ParseError> {
    let mut content = Vec::new();

    loop {
        let Some(next_token) = iter.peek() else {
            return Err(ParseError::unexpected_end_of_tokens("array"));
        };

        match next_token {
            Err(err) => return Err(err.clone()),
            Ok(Token::BracketClose) => {
                return Ok(Ast::Array(content));
            }
            _ => {
                let ast_node = parse_unknown(iter)?;

                content.push(ast_node);
                let Some(next_token) = iter.peek() else {
                    return Err(ParseError::unexpected_end_of_tokens("array"));
                };

                match next_token {
                    Err(err) => return Err(err.clone()),
                    Ok(Token::Comma) => {
                        iter.next();
                    }
                    Ok(Token::BracketClose) => {
                        continue;
                    }
                    Ok(next_token) => {
                        return Err(ParseError::unexpected_token(
                            next_token,
                            "BracketClose or Comma",
                        ));
                    }
                }
            }
        }
    }
}

pub(crate) fn parse_unknown<T: Iterator<Item = Result<Token, ParseError>>>(
    iter: &mut Peekable<T>,
) -> Result<Ast, ParseError> {
    let Some(next_token) = iter.next() else {
        return Err(ParseError::new("unexpected token stream end"));
    };
    let next_token = next_token?;
    let ast_node = match next_token {
        Token::Bool(bool) => Ast::Bool(bool),
        Token::Comma => return Err(ParseError::new("unexpected comma")),
        Token::Null => Ast::Null,
        Token::String(str) => Ast::String(str.to_owned()),
        Token::Number(value) => Ast::Number(value),
        Token::BraceOpen => {
            let obj = parse_object(iter)?;
            let next_token = iter.next();
            if !matches!(next_token, Some(Ok(Token::BraceClose))) {
                return Err(ParseError::from_string(format!(
                    "brace close expected, but got {:?}",
                    next_token
                )));
            }

            obj
        }
        Token::BracketOpen => {
            let array = parse_array(iter)?;

            let next_token = iter.next();

            if !matches!(next_token, Some(Ok(Token::BracketClose))) {
                return Err(ParseError::from_string(format!(
                    "bracket close expected, but got {:?}",
                    next_token
                )));
            }

            array
        }
        Token::BraceClose | Token::BracketClose | Token::Colon => {
            return Err(ParseError::from_string(format!(
                "invariant {:?} is unexpected",
                next_token
            )));
        }
    };

    Ok(ast_node)
}
