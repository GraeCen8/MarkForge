use crate::{
    ast::{Node, Parser},
    codegen::generate,
    token::{LexTok, Tokenizer},
};

pub mod ast;
pub mod codegen;
pub mod token;

fn main() {
    println!("Hello, world!");
}

fn md_to_html(markdown: String) -> String {
    let tokens = lex_text(&markdown);
    let ast = parse_tokens(tokens);
    let html = generate_html(ast);
    html
}

fn lex_text(text: &String) -> Vec<LexTok> {
    let mut tokenizer = Tokenizer::new(text);
    tokenizer.tokenize()
}

fn parse_tokens(tokens: Vec<LexTok>) -> Vec<Node> {
    let mut parser = Parser::new(tokens);
    parser.parse()
}

fn generate_html(ast: Vec<Node>) -> String {
    generate(ast.as_slice())
}
