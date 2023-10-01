use crate::codegen::{Value, Visitor};
use std::collections::HashMap;
use std::convert::TryFrom;

pub fn print(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    for arg in args {
        print!("{} ", String::try_from(arg.clone()).unwrap());
    }
    println!();
    Value::None
}

pub fn input(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    let mut line = String::new();
    let mut args = args.clone();
    if args.is_empty() {
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
    Value::Str(line)
}

pub fn __getattr(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    if args.len() != 2 {
        return Value::None;
    }
    // println!("getattr {:?}", _visitor.objects);
    match &args[0] {
        Value::Dict(attrs) => match attrs.get(&args[1].to_string()) {
            Some(s) => s.clone(),
            None => Value::None,
        },
        Value::Array(a) => {
            if f64::try_from(args[1].clone()).unwrap() as usize >= a.len() {
                return Value::None;
            }
            a[f64::try_from(args[1].clone()).unwrap() as usize].clone()
        }
        Value::Object(name, fns, attrs, pos) => match &_visitor.objects[*pos] {
            Some(Value::Object(n, f, a, _)) => match a.get(&args[1].to_string()) {
                Some(s) => s.clone(),
                None => Value::None,
            },
            _ => todo!(),
        },
        _ => Value::None,
    }
}
pub fn __setattr(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
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
    let mut myarr: Vec<Value>;
    if let Value::Array(a) = &args[0] {
        if f64::try_from(att.clone()).unwrap() as usize > a.len() {
            return Value::None;
        }
        myarr = a.clone();
        if f64::try_from(att.clone()).unwrap() as usize == a.len() {
            myarr.push(v);
            return Value::Array(myarr);
        }
        myarr[f64::try_from(att).unwrap() as usize] = v;
        return Value::Array(myarr);
    }
    if let Value::Object(name, fns, attrs, pos) = &args[0] {
        let mut attrs = attrs.clone();
        attrs.insert(att.to_string(), v);
        _visitor.objects[*pos] = Some(Value::Object(
            name.clone(),
            fns.clone(),
            attrs.clone(),
            *pos,
        ));
        return Value::Object(name.clone(), fns.clone(), attrs.clone(), *pos);
    }
    Value::None
}

pub fn __dict(_args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    Value::Dict(HashMap::new())
}

pub fn __array(_args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    Value::Array(Vec::new())
}

pub fn __startswith(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    if args.len() != 2 {
        return Value::None;
    }
    if let Value::Str(s) = &args[0] {
        if args[1].to_string()[..] == s[..args[1].clone().to_string().len()] {
            return Value::Boolean(true);
        }
    }
    Value::Boolean(false)
}

pub fn __len(args: Vec<Value>, _visitor: &mut Visitor) -> Value {
    if args.len() != 1 {
        return Value::None;
    }
    if let Value::Dict(d) = &args[0] {
        return Value::Float64(d.keys().len() as f64);
    }
    if let Value::Str(s) = &args[0] {
        return Value::Float64(s.len() as f64);
    }
    Value::Float64(0.0)
}
