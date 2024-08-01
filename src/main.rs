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
            // call_dynamic("puts");
            // future
                // ::block_on(async {
                    // unsafe {
                    //     let lib = libloading::Library::new("/usr/lib/liblyron.so").unwrap();
                    //     let func: libloading::Symbol<unsafe extern fn() -> *mut lyronc::ffi::LyValue> = lib.get("gen_val_ptr\0".as_bytes()).unwrap();
                    //     let lyval = func();
                    //     (*(*lyval).val).StringVal = "Hey\0"
                    //                             .as_ptr() as *mut core::ffi::c_char;
                    //     (*lyval).typeindex = 3;

                    //     println!("typeindex of call_printf: {:?}", (
                    //         *call_dynamic(
                    //             "call_printf\0", 
                    //             1,
                    //             [lyval].as_mut_ptr()
                    //         ).unwrap()).typeindex
                    //     );
                    // }
                    visitor.visit_program(program);
                // })
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

fn call_dynamic(funname: &str, arity: i32, args: *mut *mut lyronc::ffi::LyValue) -> Result<*mut lyronc::ffi::LyValue, Box<dyn std::error::Error>> {
    unsafe {
        let lib = libloading::Library::new("./examples/ffi/call_printf.so")?;
        let func: libloading::Symbol<unsafe extern fn(i32, *mut *mut lyronc::ffi::LyValue) -> *mut lyronc::ffi::LyValue> = lib.get(funname.as_bytes())?;
        Ok(func(arity, args))
    }
}