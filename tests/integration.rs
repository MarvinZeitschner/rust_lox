use regex::Regex;
use std::{fs, path::PathBuf, process::Command};
use test_generator::test_resources;

const INTERPRETER_PATH: &str = "target/debug/rust_lox";

#[test_resources("tests/scripts/*/*.lox")]
fn test_script(path: &str) {
    let path = PathBuf::from(path);
    let script_content = fs::read_to_string(&path).expect("Failed to read test script");

    let test_expectations = parse_test_expectations(&script_content);

    let output = Command::new(INTERPRETER_PATH)
        .arg(&path)
        .output()
        .expect("Failed to execute interpreter");

    let stdout_lines: Vec<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .collect();

    let stderr_lines: Vec<String> = String::from_utf8_lossy(&output.stderr)
        .lines()
        .map(|line| line.trim().to_string())
        .collect();

    println!("--------------------------------------------------------------------------------------------");
    println!("Testing file: {}", path.display());
    println!("Stdout lines: {:?}", stdout_lines);
    println!("Stderr lines: {:?}", stderr_lines);
    println!("Test expectations: {:#?}", test_expectations);
    println!("--------------------------------------------------------------------------------------------");
    println!();

    if let Some(runtime_error_idx) = test_expectations
        .iter()
        .position(|expectation| matches!(expectation, TestExpectation::RuntimeError { .. }))
    {
        let expected_error = match &test_expectations[runtime_error_idx] {
            TestExpectation::RuntimeError {
                line_number,
                error_message,
            } => {
                println!(
                    "Checking runtime error at line {}: {}",
                    line_number, error_message
                );
                error_message
            }
            _ => unreachable!(),
        };

        let error_found = stderr_lines
            .iter()
            .any(|line| line.contains(expected_error));
        assert!(
            error_found,
            "Expected runtime error '{}' not found in stderr",
            expected_error
        );

        for (i, expectation) in test_expectations.iter().enumerate() {
            if i >= runtime_error_idx {
                break;
            }

            match expectation {
                TestExpectation::Output {
                    line_number,
                    output_line,
                } if i < stdout_lines.len() => {
                    assert_eq!(
                        &stdout_lines[i], output_line,
                        "Line {}: Expected '{}' but got '{}'",
                        line_number, output_line, stdout_lines[i]
                    );
                }
                TestExpectation::Output {
                    line_number,
                    output_line,
                } => {
                    panic!(
                        "Line {}: Expected output '{}' but execution terminated early",
                        line_number, output_line
                    );
                }
                _ => {}
            }
        }
    } else {
        let mut output_idx = 0;

        for expectation in &test_expectations {
            if let TestExpectation::Output {
                line_number,
                output_line,
            } = expectation
            {
                assert!(
                    output_idx < stdout_lines.len(),
                    "Line {}: Expected output '{}' but not enough lines in stdout",
                    line_number,
                    output_line
                );

                assert_eq!(
                    &stdout_lines[output_idx], output_line,
                    "Line {}: Expected '{}' but got '{}'",
                    line_number, output_line, stdout_lines[output_idx]
                );

                output_idx += 1;
            }
        }
    }
}

#[derive(Debug)]
enum TestExpectation {
    Output {
        line_number: usize,
        output_line: String,
    },
    RuntimeError {
        line_number: usize,
        error_message: String,
    },
}

fn parse_test_expectations(script_content: &str) -> Vec<TestExpectation> {
    let mut expectations = Vec::new();

    let expected_regex = Regex::new(r"//\s*expect:\s*(.*)").unwrap();
    let runtime_error_regex = Regex::new(r"//\s*expect runtime error:\s*(.*)").unwrap();

    for (idx, line) in script_content.lines().enumerate() {
        let line_number = idx + 1;

        if let Some(captures) = expected_regex.captures(line) {
            if let Some(value_match) = captures.get(1) {
                expectations.push(TestExpectation::Output {
                    line_number,
                    output_line: value_match.as_str().trim().to_string(),
                });
            }
        }

        if let Some(captures) = runtime_error_regex.captures(line) {
            if let Some(value_match) = captures.get(1) {
                expectations.push(TestExpectation::RuntimeError {
                    line_number,
                    error_message: value_match.as_str().trim().to_string(),
                });
            }
        }
    }

    expectations
}
