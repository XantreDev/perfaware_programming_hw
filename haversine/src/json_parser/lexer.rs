use std::char;

use super::ast::ParseError;

#[derive(Debug, Clone)]
pub enum Token {
    BraceOpen,
    BraceClose,
    BracketOpen,
    BracketClose,
    Null,
    Bool(bool),
    String(String),
    Number(f64),
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

pub(crate) struct Lexer<'a> {
    data: &'a String,
    position: usize,
}

impl Lexer<'_> {
    pub(crate) fn new<'a>(value: &'a String) -> Lexer<'a> {
        Lexer {
            data: value,
            position: 0,
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Result<Token, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        lexicize(self)
    }
}

#[inline(always)]
fn get_current_char<'a>(lexer: &mut Lexer<'a>) -> Option<char> {
    lexer
        .data
        .get(lexer.position..=lexer.position)
        .and_then(|it| it.chars().next())
}

#[inline(always)]
fn skip_char<'a>(lexer: &mut Lexer<'a>) {
    lexer.position += 1;
}

#[inline(always)]
fn get_current_char_and_skip<'a>(lexer: &mut Lexer<'a>) -> Option<char> {
    let char = get_current_char(lexer);

    lexer.position += 1;
    char
}

trait AsciiReadable {
    fn read_ascii_char_at(&self, idx: usize) -> Option<char>;
}
impl AsciiReadable for str {
    fn read_ascii_char_at(&self, idx: usize) -> Option<char> {
        self.get(idx..=idx).unwrap().chars().next()
    }
}
impl AsciiReadable for String {
    fn read_ascii_char_at(&self, idx: usize) -> Option<char> {
        self.get(idx..=idx).unwrap().chars().next()
    }
}

pub fn lexicize(lexer: &mut Lexer<'_>) -> Option<Result<Token, ParseError>> {
    loop {
        if lexer.data.len() <= lexer.position {
            return None;
        };
        let Some(char) = get_current_char(lexer) else {
            return None;
        };
        if is_whitespace(char) {
            lexer.position += 1;
        } else {
            break;
        }
    }

    let Some(char) = get_current_char_and_skip(lexer) else {
        return None;
    };

    let token = match char {
        '{' => Token::BraceOpen,
        ':' => Token::Colon,
        '}' => Token::BraceClose,
        '[' => Token::BracketOpen,
        ']' => Token::BracketClose,
        'n' => {
            match consume_str(lexer, "ull") {
                Err(err) => return Some(Err(err)),
                _ => {}
            };
            Token::Null
        }
        ',' => Token::Comma,
        't' => {
            match consume_str(lexer, "rue") {
                Err(err) => return Some(Err(err)),
                _ => {}
            };
            Token::Bool(true)
        }
        'f' => {
            match consume_str(lexer, "alse") {
                Err(err) => return Some(Err(err)),
                _ => {}
            };

            Token::Bool(false)
        }
        '"' => {
            let string_token = parse_string(lexer);
            match string_token {
                Err(res) => return Err(res).into(),
                Ok(res) => Token::String(res),
            }
        }
        _ if is_zero_nine_digit(&char) || char == '-' => {
            let res = parse_number(&char, lexer);
            match res {
                Err(res) => return Err(res).into(),
                Ok(res) => Token::Number(res),
            }
        }
        _ => {
            return Err(ParseError::from_string(format!("invariant char {}", char))).into();
        }
    };

    Some(Ok(token))
}

fn lexicize_complete(data: String) -> Result<TokenStream, ParseError> {
    let mut lexer = Lexer {
        data: &data,
        position: 0,
    };

    let mut tokens = Vec::new();

    loop {
        let Some(result) = lexicize(&mut lexer) else {
            return Ok(TokenStream { tokens: tokens });
        };
        tokens.push(result?);
    }
}

fn consume_str<'a>(lexer: &mut Lexer<'a>, to_consume: &str) -> Result<(), ParseError> {
    let mut idx = 0;

    while idx < to_consume.len() {
        let Some(char) = get_current_char_and_skip(lexer) else {
            return Err(ParseError::new("unexpected tokens end"));
        };
        let expected_char = to_consume.read_ascii_char_at(idx).unwrap();
        if expected_char != char {
            return Err(ParseError::from_string(format!(
                "exected {}, but got {}",
                expected_char, char
            )));
        }

        idx += 1;
    }

    return Ok(());
}

fn is_one_nine_digit(char: &char) -> bool {
    matches!(char, '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9')
}
fn is_zero_nine_digit(char: &char) -> bool {
    matches!(
        char,
        '0' | '1' | '2' | '3' | '4' | '5' | '6' | '7' | '8' | '9'
    )
}

fn parse_number(starting_char: &char, lexer: &mut Lexer<'_>) -> Result<f64, ParseError> {
    let sign: f64 = if starting_char.to_owned() == '-' {
        -1.0
    } else {
        1.0
    };

    let mut integer: u64 = 0;
    if is_one_nine_digit(starting_char) {
        integer = (*starting_char as u64) - ('0' as u64);
    }

    loop {
        let Some(char) = get_current_char(lexer) else {
            break;
        };
        if is_zero_nine_digit(&char) {
            integer = integer * 10 + ((char as u64) - ('0' as u64));
            skip_char(lexer);
        } else {
            break;
        }
    }

    let mut fraction: f64 = 0.0;
    if matches!(get_current_char(lexer), Some('.')) {
        skip_char(lexer);

        let mut mult = 0.1;
        let mut is_first_iter = true;

        loop {
            let Some(char) = get_current_char(lexer) else {
                break;
            };

            if is_zero_nine_digit(&char) {
                is_first_iter = false;
                skip_char(lexer);
                fraction += ((char as u64) - ('0' as u64)) as f64 * mult;
                mult *= 1.0 / 10.0;
            } else {
                break;
            }
        }

        if is_first_iter {
            return Err(ParseError::from_string(format!(
                "expected to have a least one digit in fraction part, but got {:?}",
                get_current_char(lexer)
            )));
        };
    }
    let mut exponent: i64 = 0;
    if matches!(get_current_char(lexer), Some('e') | Some('E')) {
        skip_char(lexer);
        let Some(char) = get_current_char(lexer) else {
            return Err(ParseError::new("unexpected end of input in exponent form"));
        };
        let sign = if char == '-' {
            skip_char(lexer);
            -1
        } else if char == '+' {
            skip_char(lexer);
            1
        } else {
            1
        };

        let mut is_first_iter = true;
        loop {
            let Some(char) = get_current_char(lexer) else {
                break;
            };

            if (is_first_iter && !is_one_nine_digit(&char)) || !is_zero_nine_digit(&char) {
                break;
            }

            is_first_iter = false;
            skip_char(lexer);
            exponent = (exponent * 10) + ((char as i64) - ('0' as i64));
        }
        exponent = exponent * sign;

        if is_first_iter {
            return Err(ParseError::new(
                "expected to have a least one digit in fraction part",
            ));
        };
    }

    Ok((sign as f64) * ((integer as f64) + fraction) * ((10f64).powf(exponent as f64)))
}

fn parse_string(lexer: &mut Lexer<'_>) -> Result<String, ParseError> {
    let mut str = String::new();
    loop {
        let Some(char) = get_current_char_and_skip(lexer) else {
            return Err(ParseError::new("unexpected end of json"));
        };

        if char == '"' {
            return Ok(str);
        }

        // escaping
        if char == '\\' {
            let Some(next_char) = get_current_char_and_skip(lexer) else {
                return Err(ParseError::new("unexpected end by escape character"));
            };

            str.push(next_char);
            continue;
        }

        str.push(char);
    }
}

#[test]
fn check_basic_lexing() {
    insta::assert_debug_snapshot!(lexicize_complete("{ \"about\": 10 }".to_string()).unwrap());
}
#[test]
fn check_basic_lexing2() {
    insta::assert_debug_snapshot!(lexicize_complete("123.4".to_string()).unwrap());
}

#[test]
fn check_basic_lexing_exponential() {
    insta::assert_debug_snapshot!(
        lexicize_complete(
            "[123.4e1, 123e1, 123.04e1, 123.04E1, 123.04E-1, 123.04E-12, 0.5e-12]]".to_string()
        )
        .unwrap()
    );
}

#[test]
fn check_basic_lexing3() {
    insta::assert_debug_snapshot!(
        lexicize_complete(
            "{
            \"ability\": [1, null, false, 2.0, \"55\", { \"obj\": 213 }],
            \"key\": { \"value\": 220 }
        }"
            .to_string()
        )
        .unwrap()
    );
}
