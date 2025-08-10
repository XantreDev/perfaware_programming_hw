mod ast;
mod lexer;

fn main() {
    println!("Hello, world!");
    lexer::lexicize("{ \"about\": 10 }".to_string());
}
