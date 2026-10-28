//! Runs every `test/**/*.lox` file through the interpreter binary and checks the expectations
//! written as comments in the file. The files are discovered at runtime, so new tests are picked up
//! without recompiling.
//!
//! Supported annotations (taken from the Crafting Interpreters test suite):
//! - `// expect: <output>`: a line printed to stdout. All of them must match exactly and in order.
//! - `// expect runtime error: <message>`: stderr contains the message, the exit code is 70.
//! - `// Error ...` or `// [line N] Error ...`: a compile error on this line (or line N), the
//!   exit code is 65. Only the line number is checked, not the wording.
//! - `// [c line N] ...` expectations only apply to clox and are ignored.
//!
//! Without error annotations the program must exit with 0 and print nothing to stderr.

use std::{process::Command, sync::LazyLock};

use datatest_stable::Utf8Path;
use regex::Regex;

const EXIT_COMPILE_ERROR: i32 = 65;
const EXIT_RUNTIME_ERROR: i32 = 70;

static EXPECT_OUTPUT: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"// expect: ?(.*)").unwrap());
static EXPECT_RUNTIME_ERROR: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"// expect runtime error: (.+)").unwrap());
static EXPECT_ERROR_AT_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"// \[((java|c) )?line (\d+)\] Error").unwrap());
static EXPECT_ERROR: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"// Error").unwrap());

#[derive(Default)]
struct Expectations {
    output: Vec<String>,
    runtime_error: Option<String>,
    compile_error_lines: Vec<usize>,
}

fn parse_expectations(source: &str) -> Expectations {
    let mut expectations = Expectations::default();

    for (i, line) in source.lines().enumerate() {
        let line_number = i + 1;

        if let Some(captures) = EXPECT_OUTPUT.captures(line) {
            expectations.output.push(captures[1].to_string());
        } else if let Some(captures) = EXPECT_RUNTIME_ERROR.captures(line) {
            expectations.runtime_error = Some(captures[1].trim().to_string());
        } else if let Some(captures) = EXPECT_ERROR_AT_LINE.captures(line) {
            if captures.get(2).is_some_and(|lang| lang.as_str() == "c") {
                continue;
            }
            expectations
                .compile_error_lines
                .push(captures[3].parse().unwrap());
        } else if EXPECT_ERROR.is_match(line) {
            expectations.compile_error_lines.push(line_number);
        }
    }

    expectations
}

fn run_lox_test(path: &Utf8Path, source: String) -> datatest_stable::Result<()> {
    let expectations = parse_expectations(&source);

    let output = Command::new(env!("CARGO_BIN_EXE_rust_lox"))
        .arg(path)
        .output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code();

    let mut problems = vec![];

    let actual_output: Vec<&str> = stdout.lines().collect();
    if actual_output != expectations.output {
        problems.push(format!(
            "stdout differs\n  expected: {:?}\n  actual:   {:?}",
            expectations.output, actual_output
        ));
    }

    let expected_exit_code = if !expectations.compile_error_lines.is_empty() {
        for line in &expectations.compile_error_lines {
            if !stderr.contains(&format!("[line {line}]")) {
                problems.push(format!("expected a compile error on line {line}"));
            }
        }
        EXIT_COMPILE_ERROR
    } else if let Some(message) = &expectations.runtime_error {
        if !stderr.contains(message.as_str()) {
            problems.push(format!("expected runtime error '{message}'"));
        }
        EXIT_RUNTIME_ERROR
    } else {
        if !stderr.is_empty() {
            problems.push("expected no output on stderr".to_string());
        }
        0
    };

    if exit_code != Some(expected_exit_code) {
        problems.push(format!(
            "expected exit code {expected_exit_code}, got {exit_code:?}"
        ));
    }

    if problems.is_empty() {
        return Ok(());
    }

    Err(Box::new(Failure(format!(
        "{}\nstderr:\n{}",
        problems.join("\n"),
        stderr.trim_end()
    ))))
}

/// The harness prints errors with `Debug`, which would escape the line breaks of a plain `String`.
struct Failure(String);

impl std::fmt::Debug for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Failure {}

datatest_stable::harness! {
    { test = run_lox_test, root = "test", pattern = r"\.lox$" },
}
