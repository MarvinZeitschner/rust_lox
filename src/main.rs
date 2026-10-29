use clap::Parser;
use rust_lox::interpreter::Interpreter;
use std::{
    fs,
    io::{self, BufRead, IsTerminal, Write},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    path: Option<String>,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    match args.path {
        Some(path) => run_file(&path)?,
        None => run_prompt()?,
    }

    Ok(())
}

fn run_file(path: &str) -> io::Result<()> {
    let contents = fs::read_to_string(path)?;

    let mut interpreter = Interpreter::new();
    if let Err(err) = rust_lox::run(&mut interpreter, &contents) {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }

    Ok(())
}

fn run_prompt() -> io::Result<()> {
    let mut interpreter = Interpreter::new();
    let interactive = io::stdin().is_terminal();

    let mut lines = io::stdin().lock().lines();
    loop {
        if interactive {
            print!("> ");
            io::stdout().flush()?;
        }

        let Some(line) = lines.next().transpose()? else {
            break;
        };

        let source: &'static str = Box::leak(line.into_boxed_str());
        if let Err(err) = rust_lox::run(&mut interpreter, source) {
            eprintln!("{err}");
        }
    }

    if interactive {
        println!();
    }

    Ok(())
}
