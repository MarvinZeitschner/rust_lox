# rust_lox

Welcome to rust_lox.
This was my winter-semester-break-project, and is an interpreter for the Lox language written completely in rust.

I built this as a learning project using the extremely good book [Crafting Interpreters](https://craftinginterpreters.com/).
It's not meant to be extremely fast or optimized, but it does work and uses the integration test from Crafting Interpreters.

## What is this?

- An interpreter for the Lox language (see the book above)
- Has both unit and integration tests (because why not)
- Probably not the fastest or most feature-complete, but it was really fun to make

## How do i run it?

You'll need [Rust](https://www.rust-lang.org/) installed.

Clone this repo.

```bash
git clone https://github.com/MarvinZeitschner/rust_lox
cd rust_lox
```

Run a script:

```bash
cargo run <path/to/your_file.lox>
```

## Tests

I used all test from the [Crafting Interpreters Github](https://github.com/munificent/craftinginterpreters) page. So the language should be complete.

You can run them with

```bash
cargo test
```
