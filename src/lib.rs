pub mod ast;
pub mod interpreter;
pub mod lex;
pub mod parser;

use std::fmt;

use ast::Stmt;
use interpreter::{resolver::Resolver, Interpreter};
use lex::Scanner;
use parser::{Parser, TokenStream};

#[derive(Debug)]
pub enum RunError {
    /// Errors from the lexer, parser or resolver. Nothing was executed.
    Compile(Vec<String>),
    /// An error during execution. Output up to that point has already been written.
    Runtime(String),
}

impl RunError {
    pub fn exit_code(&self) -> i32 {
        match self {
            RunError::Compile(_) => 65,
            RunError::Runtime(_) => 70,
        }
    }
}

impl fmt::Display for RunError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::Compile(messages) => write!(f, "{}", messages.join("\n")),
            RunError::Runtime(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for RunError {}

fn compile_error<E: fmt::Display>(errors: Vec<E>) -> RunError {
    RunError::Compile(errors.iter().map(ToString::to_string).collect())
}

/// Runs `source` through the whole pipeline: lexer, parser, resolver and interpreter.
///
/// The same `interpreter` can run several sources one after another (like the lines of a REPL);
/// globals defined by one are visible to the next.
///
/// Functions keep a reference to their declaration in the AST, so the AST has to live as long as
/// the interpreter. It is therefore leaked: it is never freed, which is exactly that lifetime.
pub fn run<'a>(interpreter: &mut Interpreter<'a>, source: &'a str) -> Result<(), RunError> {
    let tokens = Scanner::new(source).scan_tokens().map_err(compile_error)?;
    let stmts = Parser::new(TokenStream::new(tokens))
        .parse()
        .map_err(compile_error)?;
    let stmts: &'a [Stmt<'a>] = Box::leak(stmts.into_boxed_slice());

    Resolver::new().resolve(stmts).map_err(compile_error)?;

    interpreter
        .interpret(stmts)
        .map_err(|err| RunError::Runtime(err.to_string()))
}
