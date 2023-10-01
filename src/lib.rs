pub mod codegen;
pub mod lexer;
pub mod parser;

use clap::{App, Arg};
use log::LevelFilter;
use std::collections::HashMap;
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

#[derive(Debug)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    fn new() -> Self {
        SymbolTable {
            symbols: HashMap::new(),
        }
    }

    fn insert(&mut self, name: String, symbol: Symbol) -> i32 {
        self.symbols.insert(name, symbol);
        0
    }

    fn lookup(&mut self, name: String) -> Option<Symbol> {
        self.symbols.get(&name).cloned()
    }
}

#[derive(Clone, Debug)]
pub struct Symbol {
    type_: String,
    scope: String,
}

impl Symbol {
    fn new(type_: String, scope: String) -> Self {
        Symbol { type_, scope }
    }
}

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
    /// Whether to filter logs or not.
    pub verbose: u32,
}

/// Initialize command line application to parse arguments.
pub fn init_cli() -> CLIInput {
    let matches = App::new("yotc")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Compiler for yot lang - a toy language")
        .arg(
            Arg::with_name("input")
                .help("Path to the yot file")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("print tokens")
                .help("Print raw tokens from the lexer")
                .long("print-tokens"),
        )
        .arg(
            Arg::with_name("print AST")
                .help("Print the raw abstract syntax tree")
                .long("print-ast"),
        )
        .arg(
            Arg::with_name("verbose")
                .help("Level of logging (0-2)")
                .short("v")
                .multiple(true),
        )
        .get_matches();

    let input_path = matches.value_of("input").unwrap();
    let input_name = path::Path::new(input_path)
        .file_stem()
        .unwrap()
        .to_str()
        .unwrap();

    CLIInput {
        input_path: String::from(input_path),
        input_name: String::from(input_name),
        print_tokens: matches.is_present("print tokens"),
        print_ast: matches.is_present("print AST"),
        verbose: matches.occurrences_of("verbose") as u32,
    }
}

/// Initialize logger with verbosity filter.
pub fn init_logger(verbose: u32) {
    env_logger::builder()
        .format_timestamp(None)
        .format_module_path(false)
        .filter_level(match verbose {
            0 => LevelFilter::Warn,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        })
        .init()
}
