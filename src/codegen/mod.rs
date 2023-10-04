use crate::parser::{Function, NodePosition};

use log::error;
use owo_colors::OwoColorize;
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::fmt;
use std::fmt::Debug;
use std::fs::read_to_string;
use std::process;
use std::rc::Rc;

pub mod class;
pub mod expression;
pub mod function;
pub mod json;
pub mod program;
pub mod stdlib;
pub mod osutils;

type NativeFn = fn(Vec<Value>, &mut Visitor) -> Result<Value, VMError>;
// (arity, args)->return value

pub trait Callable: Debug {
    fn arity(&self) -> usize;

    fn call_(&self, visitor: &mut Visitor, arguments: Vec<Value>) -> Result<Value, VMError>;

    fn box_clone(&self) -> Box<dyn Callable>;
}

impl Clone for Box<dyn Callable> {
    fn clone(&self) -> Box<dyn Callable> {
        self.box_clone()
    }
}

impl PartialEq for Box<dyn Callable> {
    fn eq(&self, _other: &Box<dyn Callable>) -> bool {
        false
    }
}

impl PartialOrd for Box<dyn Callable> {
    fn partial_cmp(&self, _other: &Box<dyn Callable>) -> Option<Ordering> {
        None
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Int32(i32),
    Int64(i64),
    Float32(f32),
    Float64(f64),
    Str(String),
    Boolean(bool),
    Function(String, VMFunction),
    NativeFunction(String, NativeFn),
    Class(String, HashMap<String, VMFunction>, HashMap<String, Value>),
    Object(
        String,
        HashMap<String, VMFunction>,
        HashMap<String, Value>,
        usize,
    ),
    Dict(HashMap<String, Value>),
    Array(Vec<Value>),
    None,
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::Str(ref s), Value::Str(ref o)) => s == o,
            (Value::Int32(s), Value::Int32(o)) => s == o,
            (Value::Int64(s), Value::Int64(o)) => s == o,
            (Value::Float32(s), Value::Float32(o)) => s == o,
            (Value::Float64(s), Value::Float64(o)) => s == o,
            (Value::Boolean(s), Value::Boolean(o)) => s == o,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (Value::Str(s), Value::Str(o)) => s.partial_cmp(o),
            (Value::Int32(s), Value::Int32(o)) => s.partial_cmp(o),
            (Value::Int64(s), Value::Int64(o)) => s.partial_cmp(o),
            (Value::Float32(s), Value::Float32(o)) => s.partial_cmp(o),
            (Value::Float64(s), Value::Float64(o)) => s.partial_cmp(o),
            (Value::Boolean(s), Value::Boolean(o)) => s.partial_cmp(o),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Str(s) => write!(f, "{}", s),
            Value::Int32(n) => write!(f, "{}", n),
            Value::Int64(n) => write!(f, "{}", n),
            Value::Float32(n) => write!(f, "{}", n),
            Value::Float64(n) => write!(f, "{}", n),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Class(name, ..) => write!(f, "<class {}>", name.clone()),
            _ => Ok(()),
        }
    }
}

impl TryFrom<Value> for f64 {
    type Error = VMError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Float32(n) = value {
            Ok(n as f64)
        } else if let Value::Float64(n) = value {
            Ok(n)
        } else if let Value::Int32(n) = value {
            Ok(n as f64)
        } else if let Value::Int64(n) = value {
            Ok(n as f64)
        } else {
            Err(VMError {
                type_: "CastingError".to_string(),
                cause: "Failed to cast Value into f64".to_string(),
            })
        }
    }
}

impl TryFrom<Value> for String {
    type Error = VMError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Str(s) => Ok(s),
            Value::Int32(f) => Ok(f.to_string()),
            Value::Int64(f) => Ok(f.to_string()),
            Value::Float32(f) => Ok(f.to_string()),
            Value::Float64(f) => Ok(f.to_string()),
            Value::Boolean(b) => Ok(b.to_string()),
            Value::Function(n, _f) => Ok(format!("Function <{:#}>", n).to_string()),
            Value::NativeFunction(n, _f) => Ok(format!("Native function <{:#}>", n).to_string()),
            Value::None => Ok("None".to_string()),
            Value::Class(name, _, _) => Ok(format!("Class <{:#}>", name).to_string()),
            Value::Object(classname, _, _, _) => {
                Ok(format!("Object of class <{:#}>", classname).to_string())
            }
            Value::Dict(_d) => Ok("Dict ".to_string()),
            Value::Array(_a) => Ok("array".to_string()),
        }
    }
}

// impl TryFrom<Value> for bool {
//     type Error = VMError;
//     fn try_from(value: Value) -> Result<Self,Self::Error> {
//         if let Value::Boolean(b) = value {
//             Ok(b)
//         } else {
//             Err(VMError{type_:"CastingError".to_string(), cause:"Failed to cast Value into bool".to_string()})
//         }
//     }
// }

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => b,
            _ => true,
        }
    }
}

impl TryFrom<Value> for () {
    type Error = VMError;
    fn try_from(_value: Value) -> Result<Self, Self::Error> {
        Err(VMError {
            type_: "CastingError".to_string(),
            cause: "Failed to cast Value into ()".to_string(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Visitor {
    pub position: NodePosition,
    pub variables: HashMap<String, Value>,
    pub objects: Vec<Option<Value>>,
}

#[derive(Debug)]
pub struct VMError {
    type_: String,
    cause: String,
}

#[derive(Debug, Clone)]
pub struct VMFunction {
    decl: Rc<Function>,
}

impl Callable for VMFunction {
    fn arity(&self) -> usize {
        let Function {
            name: _,
            ref args,
            expressions: _,
            return_type: _,
        } = self.decl.borrow();
        args.name.len()
    }
    fn call_(&self, visitor: &mut Visitor, arguments: Vec<Value>) -> Result<Value, VMError> {
        // println!("Called");
        let Function {
            name: _,
            args,
            expressions,
            return_type: _,
        } = self.decl.borrow();
        if args.name.is_empty() {
        } else if arguments.len() != self.arity() {
            panic!("Tried to call an invalid function");
        } else {
            for (i, arg) in args.name.clone().into_iter().enumerate() {
                visitor.variables.insert(arg, arguments[i].clone());
            }
        }

        let mut last: Result<Value, VMError> = Ok(Value::None);
        for ex in expressions {
            // println!("{:#?}", ex);
            last = visitor.visit_expr(ex.clone());
            match last {
                Ok(ref _v) => {}
                Err(e) => {
                    println!("err {:?}", e);
                    return Err(e);
                }
            }
        }
        match last {
            Ok(v) => Ok(v),
            Err(e) => {
                println!("err {:?}", e);
                Err(e)
            }
        }
    }
    fn box_clone(&self) -> Box<dyn Callable> {
        Box::new((*self).clone())
    }
}

// macro_rules! define_native {
//     ($f:expr) => {
//         self.variables.insert(
//             $f.to_string(),
//             Value::NativeFunction("$f".to_string(), stdlib::$f),
//         );
//     };
// }

impl Visitor {
    pub fn new() -> Self {
        Visitor {
            position: NodePosition {
                pos: 0,
                line_no: 0,
                file: "main".to_string(),
            },
            variables: HashMap::new(),
            objects: Vec::new(),
        }
    }
    pub fn init(&mut self) {
        self.variables.insert(
            "print".to_string(),
            Value::NativeFunction("print".to_string(), stdlib::print),
        );
        self.variables.insert(
            "input".to_string(),
            Value::NativeFunction("input".to_string(), stdlib::input),
        );
        self.variables.insert(
            "getattr".to_string(),
            Value::NativeFunction("getattr".to_string(), stdlib::__getattr),
        );
        self.variables.insert(
            "setattr".to_string(),
            Value::NativeFunction("setattr".to_string(), stdlib::__setattr),
        );
        self.variables.insert(
            "dict".to_string(),
            Value::NativeFunction("dict".to_string(), stdlib::__dict),
        );
        self.variables.insert(
            "startswith".to_string(),
            Value::NativeFunction("startswith".to_string(), stdlib::__startswith),
        );
        self.variables.insert(
            "len".to_string(),
            Value::NativeFunction("len".to_string(), stdlib::__len),
        );
        self.variables.insert(
            "array".to_string(),
            Value::NativeFunction("array".to_string(), stdlib::__array),
        );
        self.variables.insert(
            "json_parse".to_string(),
            Value::NativeFunction("json_parse".to_string(), crate::codegen::json::json_parse),
        );
        self.variables.insert(
            "json_dumps".to_string(),
            Value::NativeFunction("json_dumps".to_string(), crate::codegen::json::json_dumps),
        );
        self.variables.insert(
            "openfile".to_string(),
            Value::NativeFunction("openfile".to_string(), crate::codegen::osutils::__openfile),
        );
        self.variables.insert(
            "exec".to_string(),
            Value::NativeFunction("exec".to_string(), crate::codegen::osutils::__exec),
        );
        self.variables.insert(
            "socklisten".to_string(),
            Value::NativeFunction("socklisten".to_string(), crate::codegen::osutils::__socklisten),
        );
    }
}

pub fn uoe(v: Result<Value, VMError>, position: &NodePosition) -> Value {
    // unwrap or exit
    match v {
        Ok(a) => a,
        Err(e) => {
            error!("{}", vmerrorfmt(e, position));
            process::exit(1);
        }
    }
}

pub fn vmerrorfmt(err: VMError, position: &NodePosition) -> String {
    format!(
        "
    {text}
    {pointy}
    {type_}: {cause}

        at {line}:{pos} in file `{file}`.",
        text = read_to_string(position.file.clone())
            .unwrap()
            .lines()
            .collect::<Vec<_>>()[(position.line_no - 1) as usize],
        pointy = ("~".repeat(position.pos as usize) + "^").red(),
        type_ = err.type_.yellow(),
        cause = err.cause.blue(),
        line = position.line_no.green(),
        pos = position.pos.green(),
        file = position.file.green()
    )
    .to_string()
}

impl Default for Visitor {
    fn default() -> Self {
        Self::new()
    }
}
