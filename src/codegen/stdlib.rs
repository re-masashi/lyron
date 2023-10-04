use crate::codegen::{VMError, Value, Visitor};
use std::collections::HashMap;
use std::convert::TryFrom;

pub fn print(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    for arg in args {
        print!("{} ", String::try_from(arg.clone()).unwrap());
    }
    println!();
    Ok(Value::None)
}

pub fn input(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
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
    Ok(Value::Str(line))
}

pub fn __getattr(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.len() != 2 {
        return Ok(Value::None);
    }
    // println!("getattr {:?}", _visitor.objects);
    match &args[0] {
        Value::Dict(attrs) => match attrs.get(&args[1].to_string()) {
            Some(s) => Ok(s.clone()),
            None => Err(VMError {
                type_: "KeyError".to_string(),
                cause: format!("No such key {} in dict", &args[1].to_string()),
            }),
        },
        Value::Array(a) => {
            if f64::try_from(args[1].clone()).unwrap() as usize >= a.len() {
                return Ok(Value::None);
            }
            Ok(a[f64::try_from(args[1].clone()).unwrap() as usize].clone())
        }
        Value::Object(_name, _fns, _attrs, pos) => match &_visitor.objects[*pos] {
            Some(Value::Object(_n, _f, a, _)) => match a.get(&args[1].to_string()) {
                Some(s) => Ok(s.clone()),
                None => Err(VMError {
                    type_: "AttributError".to_string(),
                    cause: format!("No such attribute {} in object", &args[1].to_string()),
                }),
            },
            _ => todo!(),
        },
        _ => Ok(Value::None),
    }
}
pub fn __setattr(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.len() != 3 {
        return Ok(Value::None);
    }
    let mut myattrs: HashMap<String, Value>;
    let att = args[1].clone();
    let v = args[2].clone();
    if let Value::Dict(attrs) = &args[0] {
        myattrs = attrs.clone();
        myattrs.insert(att.to_string(), v);
        return Ok(Value::Dict(myattrs));
    }
    let mut myarr: Vec<Value>;
    if let Value::Array(a) = &args[0] {
        if f64::try_from(att.clone()).unwrap() as usize > a.len() {
            return Ok(Value::None);
        }
        myarr = a.clone();
        if f64::try_from(att.clone()).unwrap() as usize == a.len() {
            myarr.push(v);
            return Ok(Value::Array(myarr));
        }
        myarr[f64::try_from(att).unwrap() as usize] = v;
        return Ok(Value::Array(myarr));
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
        return Ok(Value::Object(
            name.clone(),
            fns.clone(),
            attrs.clone(),
            *pos,
        ));
    }
    Ok(Value::None)
}

pub fn __dict(_args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    Ok(Value::Dict(HashMap::new()))
}

pub fn __array(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    let mut arr: Vec<Value> = Vec::new();
    for a in args {
        arr.push(a.clone());
    }
    Ok(Value::Array(arr))
}

pub fn __startswith(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.len() != 2 {
        return Ok(Value::None);
    }
    if let Value::Str(s) = &args[0] {
        if args[1].to_string()[..] == s[..args[1].clone().to_string().len()] {
            return Ok(Value::Boolean(true));
        }
    }
    Ok(Value::Boolean(false))
}

pub fn __len(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.len() != 1 {
        return Ok(Value::None);
    }
    if let Value::Dict(d) = &args[0] {
        return Ok(Value::Float64(d.keys().len() as f64));
    }
    if let Value::Str(s) = &args[0] {
        return Ok(Value::Float64(s.len() as f64));
    }
    Ok(Value::Float64(0.0))
}
