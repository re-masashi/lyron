use lyronc::init_cli;
use lyronc::lexer::Lexer;
use lyronc::parser::Parser;
use std::process::{self};

/// Unwrap and return result, or log and exit if Err.
macro_rules! unwrap_or_exit {
    ($f:expr, $origin:tt) => {
        match $f {
            Ok(a) => a,
            Err(e) => {
                println!("{}: {}", $origin, e);
                process::exit(1);
            }
        }
    };
}

pub fn main() {
    let cli_input = init_cli();

    match cli_input.matches.subcommand_name() {
        None => {
            let lexer = unwrap_or_exit!(Lexer::from_file(&cli_input.input_path), "IO");
            let tokens = lexer
                .map(|t| unwrap_or_exit!(t, "Lexing"))
                .collect::<Vec<_>>();

            if cli_input.print_tokens {
                println!("***TOKENS***");
                tokens.iter().for_each(|t| println!("{:?}", t));
            }

            let mut parser = Parser::new(tokens.into_iter().peekable(), &cli_input.input_path);
            let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
            if cli_input.print_ast {
                println!("***AST***\n{:#?}", program);
            }
        }

        Some(_) => todo!(),
    };
}
