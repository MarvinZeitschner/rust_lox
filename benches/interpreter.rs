//! Benchmarks for every stage of the pipeline.
//!
//! Run with `cargo bench`, or a subset with e.g. `cargo bench -- interpret` or `cargo bench -- fib`.
//! Compare against a saved state with `cargo bench -- --save-baseline before` and later
//! `cargo bench -- --baseline before`. HTML reports end up in `target/criterion/report/index.html`.
//!
//! The programs in `benches/programs` are scaled-down versions of the ones in `bench/`
//! so that a single iteration takes milliseconds instead of seconds.

use std::{hint::black_box, io, time::Duration};

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rust_lox::{
    ast::Stmt,
    interpreter::{resolver::Resolver, Interpreter},
    lex::{Scanner, Token},
    parser::{Parser, TokenStream},
};

const PROGRAMS: &[(&str, &str)] = &[
    ("binary_trees", include_str!("programs/binary_trees.lox")),
    ("equality", include_str!("programs/equality.lox")),
    ("fib", include_str!("programs/fib.lox")),
    ("instantiation", include_str!("programs/instantiation.lox")),
    ("invocation", include_str!("programs/invocation.lox")),
    ("method_call", include_str!("programs/method_call.lox")),
    ("properties", include_str!("programs/properties.lox")),
    ("string_equality", include_str!("programs/string_equality.lox")),
    ("trees", include_str!("programs/trees.lox")),
    ("zoo", include_str!("programs/zoo.lox")),
];

fn lex(source: &str) -> Vec<Token<'_>> {
    Scanner::new(source).scan_tokens().unwrap()
}

fn parse(tokens: Vec<Token<'_>>) -> Vec<Stmt<'_>> {
    Parser::new(TokenStream::new(tokens)).parse().unwrap()
}

fn resolve(stmts: &[Stmt<'_>]) {
    let mut resolver = Resolver::new();
    resolver.resolve(stmts).unwrap();
}

fn lexer(c: &mut Criterion) {
    let mut group = c.benchmark_group("lexer");
    for &(name, source) in PROGRAMS {
        group.bench_function(BenchmarkId::from_parameter(name), |b| {
            b.iter(|| lex(black_box(source)))
        });
    }
    group.finish();
}

fn parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("parser");
    for &(name, source) in PROGRAMS {
        let tokens = lex(source);
        group.bench_function(BenchmarkId::from_parameter(name), |b| {
            b.iter_batched(|| tokens.clone(), parse, BatchSize::SmallInput)
        });
    }
    group.finish();
}

fn resolver(c: &mut Criterion) {
    let mut group = c.benchmark_group("resolver");
    for &(name, source) in PROGRAMS {
        let stmts = parse(lex(source));
        group.bench_function(BenchmarkId::from_parameter(name), |b| {
            b.iter(|| resolve(black_box(&stmts)))
        });
    }
    group.finish();
}

/// Only the interpretation itself; lexing, parsing and resolving happen outside the measurement.
fn interpret(c: &mut Criterion) {
    let mut group = c.benchmark_group("interpret");
    // A single iteration takes milliseconds, so fewer samples keep the run time reasonable
    group.sample_size(50);
    for &(name, source) in PROGRAMS {
        let stmts = parse(lex(source));
        resolve(&stmts);
        group.bench_function(BenchmarkId::from_parameter(name), |b| {
            b.iter(|| {
                let mut interpreter = Interpreter::with_output(Box::new(io::sink()));
                interpreter.interpret(&stmts).unwrap();
            })
        });
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(2));
    targets = lexer, parser, resolver, interpret
}
criterion_main!(benches);
