#[macro_use]
extern crate bencher;

use bencher::Bencher;

use lyronc::codegen::Visitor;
use lyronc::lexer::Lexer;
use lyronc::parser::Parser;

use std::process;
use log::error;

macro_rules! unwrap_or_exit {
    ($f:expr, $origin:tt) => {
        match $f {
            Ok(a) => a,
            Err(e) => {
                error!("{}: {}", $origin, e);
                process::exit(1);
            }
        }
    };
}


fn bench_clone_visitor(bench: &mut Bencher) {
    let lexer = unwrap_or_exit!(Lexer::from_file("bench.txt"), "IO");
    let tokens = lexer
        .map(|t| unwrap_or_exit!(t, "Lexing"))
        .collect::<Vec<_>>();

    // Parser
    let mut parser = Parser::new(tokens.into_iter().peekable(), "bench.txt");
    let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
    
    let mut visitor = Visitor::new();
    visitor.init();
    visitor.visit_program(program);

    bench.iter(|| {
        for _ in 1..10000{
            visitor.clone();
        }
    });
}

benchmark_group!(benches, bench_clone_visitor);
benchmark_main!(benches);