use log::error;
use lyronc::codegen::Visitor;
use lyronc::lexer::Lexer;
use lyronc::parser::Parser;
use lyronc::{init_cli, init_logger};
use std::process::{self, Command};
// use futures_lite::future;

/// Unwrap and return result, or log and exit if Err.
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

pub fn main() {
    let cli_input = init_cli();
    init_logger(cli_input.verbose);

    match cli_input.matches.subcommand_name(){
        None=>{
            // Lexer
            let lexer = unwrap_or_exit!(Lexer::from_file(&cli_input.input_path), "IO");
            let tokens = lexer
                .map(|t| unwrap_or_exit!(t, "Lexing"))
                .collect::<Vec<_>>();

            if cli_input.print_tokens {
                println!("***TOKENS***");
                tokens.iter().for_each(|t| println!("{:?}", t));
            }

            // Parser
            let mut parser = Parser::new(tokens.into_iter().peekable(), &cli_input.input_path);
            let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
            if cli_input.print_ast {
                println!("***AST***\n{:#?}", program);
            }
            let mut visitor = Visitor::new();
            visitor.init();
            visitor.visit_program(program);
        },
        Some("build-ffi")=>{
            Command::new("g++")
                .args(["-fPIC", "-c", "-Wall", &(cli_input.input_path.clone()+".o"), "-fpermissive", ])
                .output()
                .expect("failed to execute process");
            Command::new("ld")
                .args(["-shared", "-Wall", &(cli_input.input_path+".so")])
                .output()
                .expect("failed to execute process");
            println!("ffi");
        }
        Some(_)=>todo!()
    };
}