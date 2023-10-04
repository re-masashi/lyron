use std::fs::read_to_string;
use std::process;
use std::net::{TcpListener, TcpStream};
use crate::codegen::{VMError, Value, Visitor};


pub fn __openfile(args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    if args.len() != 3 {
        return Err(VMError {
            type_: "InvalidArguments".to_string(),
            cause: "Expected 3".to_string(),
        });
    }
    Ok(Value::Str(read_to_string(args[0].to_string()).unwrap()))
}

pub fn __exec(mut args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    let mut cmd = process::Command::new(args[0].to_string());
    args[1..].iter_mut().for_each(|x|{
        cmd.arg(x.to_string());
    });
    cmd.output().expect("OSError");
    Ok(Value::None)
}

pub fn __socklisten(mut args: Vec<Value>, _visitor: &mut Visitor) -> Result<Value, VMError> {
    let listener = TcpListener::bind("127.0.0.1:8000").unwrap();

    // accept connections and process them serially
    for stream in listener.incoming() {
        println!("{:?}", stream);
    }
    Ok(Value::None)
}