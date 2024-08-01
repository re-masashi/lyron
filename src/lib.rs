pub mod codegen;
pub mod lexer;
pub mod parser;
pub mod ffi;

use clap::{command, Command, arg};
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

    fn _lookup(&mut self, name: String) -> Option<Symbol> {
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
        .arg(
            arg!([input] "Path to the lyron file to run")
            .required(true)
        )
        // .arg(
        //     arg!(--print-tokens <PRINTTOKENS> "print tokens")
        // )
        // .arg(arg!(
        //     -t --printtokens ... "Turn debugging information on"
        // ))
        // .arg(arg!(
        //     -a --print-ast ... "Turn debugging information on"
        // ))
        // .arg(
        //     Arg::new("verbose")
        //         .help("Level of logging (0-2)")
        //         .short("v")
        //         .multiple(true),
        // )
        .subcommand(
            Command::new("build-ffi")
                .about("Builds an FFI from a given .cpp file")
        )
        .get_matches();

    let input_path = match matches.get_one::<String>("input"){
        Some(v)=>v,
        _=>{
            println!("[input] needed", );
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
        print_ast: false, // matches.is_present("print AST"),
        verbose: 0, // matches.occurrences_of("verbose") as u32,
        matches: matches,
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
