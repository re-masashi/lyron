use crate::codegen::{Value, Visitor};
use std::collections::HashMap;
use std::convert::TryFrom;

pub fn print(args: Vec<Value>, _visitor: Visitor) -> Value {
    for arg in args {
        print!("{} ", String::try_from(arg.clone()).unwrap());
    }
    print!("\n");
    return Value::None;
}

pub fn input(args: Vec<Value>, _visitor: Visitor) -> Value {
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

pub fn __getattr(args: Vec<Value>, _visitor: Visitor) -> Value {
    if args.len() != 2 {
        return Value::None;
    }
    match &args[0] {
        Value::Dict(attrs) => {
            return match attrs.get(&args[1].to_string()) {
                Some(s) => s.clone(),
                None => Value::None,
            }
        }
        _ => return Value::None,
    }
}
pub fn __setattr(mut args: Vec<Value>, _visitor: Visitor) -> Value {
    if args.len() != 3 {
        return Value::None;
    }
    let mut myattrs: HashMap<String, Value>;
    let att = args[1].clone();
    let v = args[2].clone();
    if let Value::Dict(attrs) = &args[0] {
        myattrs = attrs.clone();
        myattrs.insert(att.to_string(), v);
        return Value::Dict(myattrs);
    }
    return Value::None;
}

pub fn __dict(mut args: Vec<Value>, _visitor: Visitor) -> Value {
    return Value::Dict(HashMap::new());
}
