use crate::codegen::Value;
use std::convert::TryFrom;

pub fn print(_arity: i32, args: Vec<Value>) -> Value {
    for arg in args {
        print!("{} ", String::try_from(arg).unwrap());
    }
    print!("\n");
    return Value::None;
}

pub fn input(_arity: i32, args: Vec<Value>) -> Value {
    let mut line = String::new();
    let mut args = args.clone();
    if args.len() == 0 {
        args[0] = Value::Str("".to_string())
    }
    println!(
        "{}",
        match args[0].clone() {
            Value::Str(s) => s,
            _ => "".to_string(),
        }
    );
    std::io::stdin().read_line(&mut line).unwrap();
    return Value::Str(line);
}
