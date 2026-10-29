use std::{
    io::Write,
    process::{Command, Stdio},
};

fn run_repl(input: &str) -> (String, String, Option<i32>) {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rust_lox"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("failed to start the interpreter");

    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();

    let output = child.wait_with_output().unwrap();
    (
        String::from_utf8_lossy(&output.stdout).to_string(),
        String::from_utf8_lossy(&output.stderr).to_string(),
        output.status.code(),
    )
}

#[test]
fn state_is_kept_between_lines() {
    let (stdout, stderr, code) = run_repl(
        "var a = 1;\n\
         fun times(n) { return a * n; }\n\
         a = 2;\n\
         print times(21);\n",
    );

    assert_eq!(stdout, "42\n");
    assert_eq!(stderr, "");
    assert_eq!(code, Some(0));
}

#[test]
fn errors_do_not_end_the_session() {
    let (stdout, stderr, code) = run_repl(
        "print 1;\n\
         print missing;\n\
         var = 3;\n\
         print 2;\n",
    );

    assert_eq!(stdout, "1\n2\n");
    assert!(stderr.contains("Undefined variable 'missing'."), "{stderr}");
    assert!(stderr.contains("Expected variable name."), "{stderr}");
    assert_eq!(code, Some(0));
}
