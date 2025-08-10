use crate::lexer::TokenStream;

enum NumberType {
    Fp(f64),
    Int(i64),
}

enum Ast {
    Object(Vec<Ast>),
    Array(Vec<Ast>),
    Number(NumberType),
    Null,
    String(String),
}

fn build_ast(token_stream: TokenStream) {}
