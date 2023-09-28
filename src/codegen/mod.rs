use crate::parser::{Function, NodePosition};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::convert::{From, TryFrom};
use std::fmt;
use std::fmt::Debug;
use std::rc::Rc;

pub mod expression;
pub mod function;
pub mod program;
pub mod stdlib;
pub mod class;

type NativeFn = fn(i32, Vec<Value>) -> Value;
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
    Class(String, HashMap<String, VMFunction>),
    Object(String, HashMap<String, VMFunction>),
    None,
}

impl PartialEq for Value {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (&Value::Str(ref s), &Value::Str(ref o)) => s == o,
            (&Value::Int32(ref s), &Value::Int32(ref o)) => s == o,
            (&Value::Int64(ref s), &Value::Int64(ref o)) => s == o,
            (&Value::Float32(ref s), &Value::Float32(ref o)) => s == o,
            (&Value::Float64(ref s), &Value::Float64(ref o)) => s == o,
            (&Value::Boolean(ref s), &Value::Boolean(ref o)) => s == o,
            _ => false,
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Value) -> Option<Ordering> {
        match (self, other) {
            (&Value::Str(ref s), &Value::Str(ref o)) => s.partial_cmp(o),
            (&Value::Int32(ref s), &Value::Int32(ref o)) => s.partial_cmp(o),
            (&Value::Int64(ref s), &Value::Int64(ref o)) => s.partial_cmp(o),
            (&Value::Float32(ref s), &Value::Float32(ref o)) => s.partial_cmp(o),
            (&Value::Float64(ref s), &Value::Float64(ref o)) => s.partial_cmp(o),
            (&Value::Boolean(ref s), &Value::Boolean(ref o)) => s.partial_cmp(o),
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Value::Str(ref s) => write!(f, "{}", s),
            &Value::Int32(n) => write!(f, "{}", n),
            &Value::Int64(n) => write!(f, "{}", n),
            &Value::Float32(n) => write!(f, "{}", n),
            &Value::Float64(n) => write!(f, "{}", n),
            &Value::Boolean(ref b) => write!(f, "{}", b),
            Value::Class(name, ..)=>write!(f,"<class {}>", name.clone()),
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
            Ok(n as f64)
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
            Value::Str(s) => {
                return Ok(s);
            }
            Value::Int32(f) => {
                return Ok(f.to_string());
            }
            Value::Int64(f) => {
                return Ok(f.to_string());
            }
            Value::Float32(f) => {
                return Ok(f.to_string());
            }
            Value::Float64(f) => {
                return Ok(f.to_string());
            }
            Value::Boolean(b) => return Ok(b.to_string()),
            Value::Function(n, _f) => {
                return Ok(format!("Function <{:#}>", n).to_string());
            }
            Value::NativeFunction(n, _f) => {
                return Ok(format!("Native function <{:#}>", n).to_string())
            }
            Value::None => return Ok("None".to_string()),
            Value::Class(name, _) => return Ok(format!("Class <{:#}>", name).to_string()),
            Value::Object(classname, _) => return Ok(format!("Object of class <{:#}>", classname).to_string())
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
}

#[derive(Debug)]
pub struct VMError {
    type_: String,
    cause: String,
}

#[derive(Debug, Clone)]
struct VMFunction {
    decl: Rc<Function>,
}

impl Callable for VMFunction {
    fn arity(&self) -> usize {
        match self.decl.borrow() {
            Function {
                name: _,
                ref args,
                expressions: _,
                return_type: _,
            } => args.name.len(),
            _ => 0,
        }
    }
    fn call_(&self, visitor: &mut Visitor, arguments: Vec<Value>) -> Result<Value, VMError> {
        // println!("Called");
        let (_name, args, expressions, _return_type) = match self.decl.borrow() {
            Function {
                name,
                args,
                expressions,
                return_type,
            } => (name, args, expressions, return_type),
            _ => panic!("Tried to call an invalid function"),
        };
        if args.name.len() == 0 {
            2;
        }else if arguments.len()!=self.arity(){
            panic!("Tried to call an invalid function");
        }else {
            for (i, arg) in args.name.clone().into_iter().enumerate() {
                visitor.variables.insert(arg, arguments[i].clone());
            }
        }

        let mut last: Result<Value, VMError> = Ok(Value::None);
        for ex in expressions {
            // println!("{:#?}", ex);
            last = visitor.visit_expr(ex.clone());
        }
        match last {
            Ok(v) => return Ok(v),
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

macro_rules! define_native {
    ($f:expr) => {
        self.variables.insert(
            $f.to_string(),
            Value::NativeFunction("$f".to_string(), stdlib::$f),
        );
    };
}

impl Visitor {
    pub fn new() -> Self {
        Visitor {
            position: NodePosition {
                pos: 0,
                line_no: 0,
                file: "main".to_string(),
            },
            variables: HashMap::new(),
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
    }
    fn _unwrap(&self, expr: Result<Value, VMError>) -> Value {
        match expr {
            Ok(v) => v,
            _ => std::process::exit(1),
        }
    }
}
