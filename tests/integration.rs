use std::{fs, path::Path};

use rust_lox::{
    ast::Stmt,
    interpreter::Interpreter,
    lex::Scanner,
    parser::{Parser, TokenStream},
};

enum Expectation {
    Output(String),
    Error(String),
}

struct LineResult {
    line: usize,
    expected: Expectation,
    actual: Result<(), Box<dyn std::error::Error>>,
}

struct AnnotatedTestRunner<'a> {
    interpreter: Interpreter<'a>,
}

impl<'a> AnnotatedTestRunner<'a> {
    fn new() -> Self {
        Self {
            interpreter: Interpreter::new(),
        }
    }

    fn setup(&self, input: &'a str) -> Vec<Stmt<'a>> {
        let mut lexer = Scanner::new(input);
        // TODO: Integration tests for parsing
        Parser::new(TokenStream::new(lexer.scan_tokens().unwrap()))
            .parse()
            .unwrap()
    }

    fn parse_expectation(&self, script: &str) -> Vec<(usize, Expectation)> {
        script
            .lines()
            .enumerate()
            .filter_map(|(line_num, line)| {
                if let Some(expectation_str) = line.trim().strip_prefix("// expected:") {
                    let expectation = match expectation_str.trim() {
                        // TODO: add more types to check against
                        output if !output.is_empty() => {
                            Some(Expectation::Output(output.to_string()))
                        }
                        _ => None,
                    };
                    expectation.map(|exp| (line_num, exp))
                } else {
                    None
                }
            })
            .collect()
    }

    fn run(&mut self, script: &'a str) -> Vec<LineResult> {
        let expecations = self.parse_expectation(script);

        // expecations
        //     .into_iter()
        //     .map(|(line_num, expectation)| {
        //         let stmts = self.setup(script);
        //         let result = match self.interpreter.interpret(stmts) {
        //             Ok(output) => ,
        //             Err(_) => todo!(),
        //         };
        //     })
        //     .collect()

        todo!()
    }
}

fn run_lox_test_files(directory: &str) {
    let path = Path::new(directory);

    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to get directory entry");
        let path = entry.path();

        if path.extension().unwrap_or_default() == "lox" {
            let script_content = fs::read_to_string(&path).expect("Failed to read Lox script");

            // let result = run(&script_content);
            // println!("Running test file: {:?}", path);
            // assert!(result.is_ok(), "Test file {} failed", path.display());
        }
    }
}

#[test]
fn test_lox_scripts_in_test_directory() {
    run_lox_test_files("tests/scripts");
}
