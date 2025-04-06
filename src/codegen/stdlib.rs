use crate::codegen::{VMError, Value, Visitor};

#[cfg(not(feature = "gxhash"))]
use std::collections::HashMap;

#[cfg(feature = "gxhash")]
use gxhash::{HashMap, HashMapExt};

use std::convert::TryFrom;
// use rayon::prelude::*;
use crate::codegen::Callable;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;
// use rayon::vec;
// use std::borrow::{Borrow};

pub fn print(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    // let mut stdout = std::io::stdout();
    // let mut lock = stdout.lock();

    // for arg in args {
    //     write!(lock, "{} ", String::try_from(arg.clone()).unwrap());
    // }
    // writeln!(lock, "");
    for arg in args {
        print!("{} ", String::try_from(arg.clone()).unwrap());
    }
    println!("");
    Ok(Value::None)
}

pub fn input(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
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

pub fn __getattr(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    if args.len() != 2 {
        return Ok(Value::None);
    }
    // println!("getattr {:?}", _visitor.objects);
    match &args[0] {
        Value::Dict(attrs) => match attrs.get(&args[1].to_string()) {
            Some(s) => Ok(s.clone()),
            // None => Err(VMError {
            //     type_: "KeyError".to_string(),
            //     cause: format!("No such key {} in dict", &args[1].to_string()),
            // }),
            None => Ok(Value::None),
        },
        Value::Array(a) => {
            if f64::try_from(args[1].clone()).unwrap() as usize >= a.len() {
                return Ok(Value::None);
            }
            Ok(a[f64::try_from(args[1].clone()).unwrap() as usize].clone())
        }
        Value::Object(_name, _fns, _attrs, pos) => match &_visitor.objects.borrow()[*pos] {
            Value::Object(_n, _f, a, _) => match a.get(&args[1].to_string()) {
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

pub fn __setattr(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
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
        _visitor.objects.borrow_mut()[*pos] = Value::Object(
            name.clone(),
            fns.clone(),
            attrs.clone(),
            *pos,
        );
        return Ok(Value::Object(
            name.clone(),
            fns.clone(),
            attrs.clone(),
            *pos,
        ));
    }
    Ok(Value::None)
}

pub fn __dict(_args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    Ok(Value::Dict(HashMap::new()))
}

pub fn __array(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    let mut arr: Vec<Value> = Vec::new();
    for a in args {
        arr.push(a.clone());
    }
    Ok(Value::Array(arr))
}

pub fn __startswith(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
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

pub fn __len(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
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

pub fn __dict_keys(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    let mut items: Vec<Value> = vec![];
    if let Value::Dict(map) = &args[0] {
        for key in map.keys() {
            // println!("key: {key}");
            items.push(Value::Str(key.to_string()));
        }
    }
    return Ok(Value::Array(items));
}

pub fn start_tcp_server(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    // let handle = &args[0];

    let handle;
    let mut port = &Value::Str("7878".to_string());
    let mut address = &Value::Str("127.0.0.1".to_string());

    match args.len() {
        1 => {
            handle = &args[0];
        }
        2 => {
            handle = &args[0];
            port = &args[1];
        }
        3 => {
            handle = &args[0];
            port = &args[1];
            address = &args[2];
        }
        _ => {
            return Err(VMError {
                type_: "RuntimeError".to_string(),
                cause: format!(
                    "Invalid number of arguments in `start_tcp_server`. found `{}`",
                    &args.len()
                ),
            })
        }
    }

    let listener = TcpListener::bind(address.to_string() + ":" + &port.to_string()).unwrap();

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();
        let mut headers = [httparse::EMPTY_HEADER; 64];
        let mut req = httparse::Request::new(&mut headers);
        let mut data: HashMap<String, Value> = HashMap::new();
        let mut headers: HashMap<String, Value> = HashMap::new();

        data.insert(
            "addr".to_string(),
            Value::Str(stream.local_addr().unwrap().to_string()),
        );
        data.insert(
            "peer".to_string(),
            Value::Str(stream.peer_addr().unwrap().to_string()),
        );

        let buf_reader = BufReader::new(&mut stream);

        // println!("{:?}", data);
        // uoe(handle.clone().call_(
        //     self,
        // ), &self.position);

        let http_request: String = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect::<Vec<String>>()
            .join("\r\n");

        if req.parse(http_request.as_bytes()).unwrap().is_partial() {
            // do nothn
        }

        for header in req.headers {
            if header.name != "" {
                // println!("{:?}", header);
                headers.insert(
                    header.name.to_lowercase().to_string(),
                    Value::Str(std::str::from_utf8(header.value).unwrap().to_string()),
                );
            }
        }

        if let Value::Function(_name, fn_) = handle {
            data.insert("headers".to_string(), Value::Dict(headers));

            data.insert(
                "path".to_string(),
                match req.path {
                    Some(path) => Value::Str(path.to_string()),
                    _ => Value::Str('/'.to_string()),
                },
            );

            let resp = crate::codegen::uoe(
                fn_.clone().call_(
                    _visitor,
                    vec![Value::Dict(data), Value::Str(http_request.clone())],
                ),
                &_visitor.position,
            )
            .to_string();

            // println!("{:?}", req.parse(http_request.as_bytes()).unwrap());

            println!("\n\nRequest: {http_request:}\n\n");
            println!("Connection established!");
            stream
                .write_all(format!("HTTP/1.1 200 OK\r\n\r\n{resp}\r\n\r\n").as_bytes())
                .unwrap();
        } else {
            stream
                .write_all(format!("HTTP/1.1 200 OK\r\n\r\nHELLO\r\n\r\n").as_bytes())
                .unwrap();
        }
    }
    return Ok(Value::Float64(0.0));
}

pub fn read_file(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    let file = OpenOptions::new().read(true).open(args[0].to_string());
    if let Err(_e) = file {
        return Err(VMError {
            type_: "FSError".to_string(),
            cause: format!("No such file `{}` found", &args[0].to_string()),
        });
    }
    let mut contents = String::new();
    if let Err(_e) = file.unwrap().read_to_string(&mut contents) {
        return Err(VMError {
            type_: "FSError".to_string(),
            cause: format!("Failed to read file `{}`", &args[0].to_string()),
        });
    };
    return Ok(Value::Str(contents));
}

pub fn write_file(args: Vec<Value>, _visitor: &Visitor) -> Result<Value, VMError> {
    let file = OpenOptions::new().write(true).open(args[0].to_string());
    if let Err(_e) = file {
        return Err(VMError {
            type_: "FSError".to_string(),
            cause: format!("No such file `{}` found", &args[0].to_string()),
        });
    }
    if let Err(_e) = file.unwrap().write_all(args[1].to_string().as_bytes()) {
        return Err(VMError {
            type_: "FSError".to_string(),
            cause: format!("Failed to read file `{}`", &args[0].to_string()),
        });
    };
    return Ok(Value::Boolean(true));
}
