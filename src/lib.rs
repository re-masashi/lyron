pub mod lexer;
pub mod parser;

use clap::{arg, command};
use std::path;

#[macro_export]
macro_rules! unwrap_some {
    ($val:expr) => {
        match $val {
            Some(s) => s,
            None => return Err("EOF".to_string()),
        }
    };
}

pub type Result<T> = std::result::Result<T, String>;

/// CLI input configuration and parameters.
pub struct CLIInput {
    /// Path to input file.
    pub input_path: String,
    /// `input_path` file name without file extension.
    pub input_name: String,
    /// Whether or not raw tokens should be printed.
    pub print_tokens: bool,
    /// Whether or not raw AST should be printed.
    pub print_ast: bool,

    pub matches: clap::ArgMatches,
}

/// Initialize command line application to parse arguments.
pub fn init_cli() -> CLIInput {
    let matches = command!("lyronc")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Compiler for lyronc - a toy language")
        // .arg(
        //     Arg::new("input")
        //         .help("Path to the lyr file")
        //         .required(true)
        //         .index(1),
        // )
        .infer_subcommands(true)
        .arg(arg!([input] "Path to the lyron file to run").required(true))
        .get_matches();

    let input_path = match matches.get_one::<String>("input") {
        Some(v) => v,
        _ => {
            println!("[input] needed",);
            unreachable!()
        }
    };

    // let input_path = matches.value_of("input").unwrap();
    let input_name = path::Path::new(input_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    CLIInput {
        input_path: String::from(input_path),
        input_name: String::from(input_name),
        print_tokens: false, // matches.is_present("print tokens"),
        print_ast: false,    // matches.is_present("print AST"),
        matches,
    }
}
