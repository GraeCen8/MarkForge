use crate::{
    ast::{Node, Parser},
    token::{LexTok, Tokenizer},
};

pub mod ast;
pub mod codegen;
pub mod token;

fn main() {
    println!("Hello, world!");
}

fn lex_text(text: &String) -> Vec<LexTok> {
    let mut tokenizer = Tokenizer::new(text);
    tokenizer.tokenize()
}

fn parse_tokens(tokens: Vec<LexTok>) -> Vec<Node> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}
