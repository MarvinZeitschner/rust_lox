use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(required = true)]
    path: String,
}

fn main() {
    let args = Args::parse();
    println!("path: {}", args.path);
}
