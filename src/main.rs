use clap::Parser;
use rust_lox::interpreter::Interpreter;
use std::fs;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let contents = fs::read_to_string(&args.path)?;

    let mut interpreter = Interpreter::new();
    if let Err(err) = rust_lox::run(&mut interpreter, &contents) {
        eprintln!("{err}");
        std::process::exit(err.exit_code());
    }

    Ok(())
}
