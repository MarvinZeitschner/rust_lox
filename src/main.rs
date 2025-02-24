use std::fs;

use clap::Parser;
use rust_lox::{
    ast::printer::AstPrinter,
    lex::lexer::Scanner,
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
    let file_content =
        &fs::read_to_string(args.path).expect("Should have been able to read the file");
    let contents = file_content.trim();

    let mut parser = setup(contents);

    match parser.parse() {
        Ok(expr) => {
            let res = expr.accept(&mut AstPrinter);
            println!("{:#?}", res);
        }
        Err(e) => println!("{}", e),
    }
}

fn setup(input: &str) -> parser::Parser {
    let mut lexer = Scanner::new(input);
    parser::Parser::new(TokenStream::new(lexer.scan_tokens().unwrap()))
}
