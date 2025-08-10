use std::{
    char,
    iter::{Enumerate, Peekable},
};

#[derive(Debug)]
pub struct Atom {
    start: u32,
    end: u32,
}

#[derive(Debug)]
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
    data: String,
    tokens: Vec<Token>,
}

fn skip_n<T: Iterator>(value: &mut T, elements: u32) {
    for _ in 0..elements {
        value.next();
    }
}

pub fn lexicize(data: String) -> TokenStream {
    let mut chars = data.chars().enumerate().peekable();
    let mut tokens = Vec::new();

    loop {
        loop {
            let Some((_, char)) = chars.peek() else {
                drop(chars);
                return TokenStream { data, tokens };
            };
            if is_whitespace(char.to_owned()) {
                chars.next();
            } else {
                break;
            }
        }

        let Some((idx, char)) = chars.next() else {
            drop(chars);
            return TokenStream { data, tokens };
        };

        let token = match char {
            '{' => Token::BraceOpen,
            ':' => Token::Colon,
            ('}') => Token::BraceClose,
            ('[') => Token::BracketOpen,
            (']') => Token::BracketClose,
            ('n') => {
                skip_n(&mut chars, 3);
                Token::Null
            }
            (',') => Token::Comma,
            ('t') => {
                skip_n(&mut chars, 3);
                Token::Bool(true)
            }
            ('f') => {
                skip_n(&mut chars, 4);
                Token::Bool(false)
            }
            '"' => Token::String(parse_string(&mut chars)),
            _ if is_digit_char(&char) => Token::Number(parse_number_string(&char, &mut chars)),
            _ => panic!("invariant char {}", char),
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

fn parse_string<T: Iterator<Item = char>>(iter: &mut Peekable<Enumerate<T>>) -> String {
    let mut str = String::new();
    loop {
        let Some((_, char)) = iter.next() else {
            panic!("unexpected end of json");
        };

        if char == '"' {
            return str;
        }

        // escaping
        if char == '\\' {
            let Some((_, next_char)) = iter.next() else {
                panic!("unexpected end by escape character");
            };

            str.push(next_char);
            continue;
        }

        str.push(char);
    }
}

#[test]
fn check_basic_lexing() {
    insta::assert_debug_snapshot!(lexicize("{ \"about\": 10 }".to_string()));
    insta::assert_debug_snapshot!(lexicize("123.4".to_string()));
    insta::assert_debug_snapshot!(lexicize(
        "{
            \"ability\": [1, null, false, 2.0, \"55\", { \"obj\": 213 }],
            \"key\": { \"value\": 220 }
        }"
        .to_string()
    ));
}
