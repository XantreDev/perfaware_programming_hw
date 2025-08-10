use crate::{
    ast::{Ast, build_ast},
    lexer::lexicize,
};

mod ast;
mod lexer;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}
impl ParseError {
    fn from_string(message: String) -> ParseError {
        ParseError { message: message }
    }
}

pub fn parse_json(json: String) -> Result<Ast, ParseError> {
    let token_stream = lexicize(json).map_err(|err| {
        ParseError::from_string(format!("failed to tokenize; cause: {:?}", err.message))
    })?;

    let ast = build_ast(token_stream).map_err(|err| {
        ParseError::from_string(format!("failed to build ast; cause {:?}", err.message))
    })?;

    Ok(ast)
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn feature() {
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
