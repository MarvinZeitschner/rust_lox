use printer::AstPrinter;
use rust_lox::{ast::*, lex::lexer::*, parser::*};

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    path: String,
}

fn main() {
    let input = "1 + 2";
    let mut parser = setup(input);

    match parser.parse() {
        Ok(expr) => {
            let res = expr.accept(&mut AstPrinter);
            println!("{:#?}", res);
        }
        Err(e) => println!("{}", e),
    }
}

fn setup(input: &str) -> Parser {
    let mut lexer = Scanner::new(input);
    Parser::new(TokenStream::new(lexer.scan_tokens().unwrap()))
}
