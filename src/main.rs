use clap::Parser;
use rust_lox::{
    interpreter::{resolver::Resolver, Interpreter},
    lex::Scanner,
    parser::{self, TokenStream},
};
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let binding = fs::read_to_string(&args.path)?;
    let contents = binding.trim();

    let mut lexer = Scanner::new(contents);
    let tokens = lexer.scan_tokens();
    let tokens = match tokens {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(65);
        }
    };

    let mut parser = parser::Parser::new(TokenStream::new(tokens));
    let stmts = parser.parse();
    let stmts = match stmts {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(65);
        }
    };

    let mut resolver = Resolver::new();
    if let Err(e) = resolver.resolve(&stmts) {
        eprintln!("{e:#}");
        std::process::exit(65);
    }

    let mut interpreter = Interpreter::new(resolver.get_locals());
    let res = interpreter.interpret(&stmts);

    match res {
        Ok(_) => Ok(()),
        Err(e) => {
            eprintln!("{e:#}");
            std::process::exit(65);
        }
    }
}
