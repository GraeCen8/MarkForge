use crate::{ast::parse_tokens, codegen::generate_html, styler::wrap_html, token::lex_text};

pub mod ast;
pub mod codegen;
pub mod styler;
pub mod token;

use clap::Parser;
use std::fs;
use std::path::PathBuf;

/// Read a file and either print or write output
#[derive(Parser, Debug)]
#[command(name = "file-cli")]
#[command(about = "Process a file and output result")]
struct Args {
    /// Input file path
    #[arg(short, long)]
    input: PathBuf,

    /// Optional output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
}

fn main() {
    let args = Args::parse();

    // Read input
    let content = match fs::read_to_string(&args.input) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("Error reading input file: {}", err);
            std::process::exit(1);
        }
    };

    let processed = md_to_html(content);

    match args.output {
        Some(path) => {
            if let Err(err) = fs::write(&path, processed) {
                eprintln!("Error writing to file: {}", err);
                std::process::exit(1);
            }
            println!("Output written to {:?}", path);
        }
        None => {
            println!("{}", processed);
        }
    }
}

fn md_to_html(markdown: String) -> String {
    let tokens = lex_text(&markdown);
    let ast = parse_tokens(tokens);
    let html = generate_html(ast);
    wrap_html(&html)
}
