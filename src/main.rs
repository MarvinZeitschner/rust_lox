use std::fs;

use clap::Parser;
use rust_lox::{
    interpreter::Interpreter,
    lex::Scanner,
    parser::{self, TokenStream},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    path: String,
}

fn main() {
    let args = Args::parse();
    let file_content = &fs::read_to_string(args.path.clone())
        .unwrap_or_else(|_| panic!("Failed to read file: {}", args.path));
    let contents = file_content.trim();

    let mut parser = setup(contents);

    match parser.parse() {
        Ok(expr) => {
            let res = expr.accept(&mut Interpreter);
            println!("{:#?}", res);
        }
        Err(e) => println!("{}", e),
    }
}

fn setup(input: &str) -> parser::Parser {
    let mut lexer = Scanner::new(input);
    parser::Parser::new(TokenStream::new(lexer.scan_tokens().unwrap()))
}
