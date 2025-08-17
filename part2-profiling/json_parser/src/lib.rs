use crate::{
    ast::{Ast, ParseError, parse_unknown},
    lexer::Lexer,
};

pub mod ast;
mod lexer;

pub fn parse_json(json: String) -> Result<Ast, ParseError> {
    let mut iter = Lexer::new(&json).peekable();

    let ast = parse_unknown(&mut iter)?;

    Ok(ast)
}

#[test]
fn parses() {
    insta::assert_debug_snapshot!(
        parse_json(
            "{
            \"ability\": [1, null, false, 2.0, \"55\", { \"obj\": 213 }],
            \"key\": { \"value\": 220 }
        }"
            .to_string()
        )
        .unwrap()
    );
}

#[test]
fn parses_numbers() {
    let json = r#"{"x0": 175.60221894314063, "y0": 60.66904027153596, "x1": -63.561257303169754, "y1": -44.86367589191168}"#.to_string();

    parse_json(json).expect("json is parsed");
}

#[test]
fn parses_big_json() {
    use std::{fs::File, io::Read};
    let mut file = File::open("test.json").unwrap();
    let mut json = String::new();
    file.read_to_string(&mut json).unwrap();

    parse_json(json).expect("json is parsed");
}
