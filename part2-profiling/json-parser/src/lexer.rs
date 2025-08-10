use std::{
    char,
    iter::{Enumerate, Peekable},
};

#[derive(Debug, Clone)]
pub enum Token {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Null,
    Bool(bool),
    String(String),
    Number(String),
    Comma,
    Colon,
}
// TODO: probably arena allocations will be beneficial
fn is_whitespace(value: char) -> bool {
    value == ' ' || value == '\n' || value == '\r' || value == '\t'
}

#[derive(Debug)]
pub struct TokenStream {
    pub tokens: Vec<Token>,
}

fn skip_n<T: Iterator>(value: &mut T, elements: u32) {
    for _ in 0..elements {
        value.next();
    }
}

#[derive(Debug, Clone)]
pub(crate) struct UnknownTokenError {
    pub(crate) message: String,
}

impl UnknownTokenError {
    fn new(message: &str) -> UnknownTokenError {
        UnknownTokenError {
            message: message.to_string(),
        }
    }
    fn from_string(message: String) -> UnknownTokenError {
        UnknownTokenError { message: message }
    }
}

pub fn lexicize(data: String) -> Result<TokenStream, UnknownTokenError> {
    let mut chars = data.chars().enumerate().peekable();
    let mut tokens = Vec::new();

    loop {
        loop {
            let Some((_, char)) = chars.peek() else {
                drop(chars);
                return Ok(TokenStream { tokens });
            };
            if is_whitespace(char.to_owned()) {
                chars.next();
            } else {
                break;
            }
        }

        let Some((_, char)) = chars.next() else {
            drop(chars);
            return Ok(TokenStream { tokens });
        };

        let token = match char {
            '{' => Token::BraceOpen,
            ':' => Token::Colon,
            '}' => Token::BraceClose,
            '[' => Token::BracketOpen,
            ']' => Token::BracketClose,
            'n' => {
                skip_n(&mut chars, 3);
                Token::Null
            }
            ',' => Token::Comma,
            't' => {
                skip_n(&mut chars, 3);
                Token::Bool(true)
            }
            'f' => {
                skip_n(&mut chars, 4);
                Token::Bool(false)
            }
            '"' => Token::String(parse_string(&mut chars)?),
            _ if is_digit_char(&char) => Token::Number(parse_number_string(&char, &mut chars)),
            _ => {
                return Err(UnknownTokenError::from_string(format!(
                    "invariant char {}",
                    char
                )));
            }
        };

        tokens.push(token);
    }
}

fn is_digit_char(char: &char) -> bool {
    matches!(
        char,
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9' | '.'
    )
}

fn parse_number_string<T: Iterator<Item = char>>(
    starting_char: &char,
    iter: &mut Peekable<Enumerate<T>>,
) -> String {
    let mut str = starting_char.to_string();
    loop {
        let Some((_, char)) = iter.peek() else {
            return str;
        };

        if !is_digit_char(char) {
            return str;
        }
        str.push(char.to_owned());

        iter.next();
    }
}

fn parse_string<T: Iterator<Item = char>>(
    iter: &mut Peekable<Enumerate<T>>,
) -> Result<String, UnknownTokenError> {
    let mut str = String::new();
    loop {
        let Some((_, char)) = iter.next() else {
            return Err(UnknownTokenError::new("unexpected end of json"));
        };

        if char == '"' {
            return Ok(str);
        }

        // escaping
        if char == '\\' {
            let Some((_, next_char)) = iter.next() else {
                return Err(UnknownTokenError::new("unexpected end by escape character"));
            };

            str.push(next_char);
            continue;
        }

        str.push(char);
    }
}

#[test]
fn check_basic_lexing() {
    insta::assert_debug_snapshot!(lexicize("{ \"about\": 10 }".to_string()).unwrap());
}
#[test]
fn check_basic_lexing2() {
    insta::assert_debug_snapshot!(lexicize("123.4".to_string()).unwrap());
}

#[test]
fn check_basic_lexing3() {
    insta::assert_debug_snapshot!(
        lexicize(
            "{
            \"ability\": [1, null, false, 2.0, \"55\", { \"obj\": 213 }],
            \"key\": { \"value\": 220 }
        }"
            .to_string()
        )
        .unwrap()
    );
}
