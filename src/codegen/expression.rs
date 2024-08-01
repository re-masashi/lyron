use crate::codegen::{uoe, Callable, VMError, VMFunction, Value, Visitor};
use crate::lexer::tokens::TokenType;
use crate::lexer::Lexer;
use crate::parser::{ExprValue, Parser};
use log::error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::process;
use rayon::prelude::*;
use serde_json::{Value as SerdeValue};
use std::convert::TryInto;

type Result<T> = std::result::Result<T, VMError>;
type DynFn = unsafe extern fn(i32, *mut *mut crate::ffi::LyValue) -> *mut crate::ffi::LyValue;

const MONITOR_THRESHOLD: usize = 300;
const JIT_THRESHOLD: usize = 700;

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

// macro_rules! uoe {
//     ($f:expr) => {
//         self.uoe(expr)
//     };
// }

impl Visitor {
    pub fn visit_expr(&mut self, expr: &ExprValue) -> Result<Value> {
        match expr {
            ExprValue::FnCall(name, args) => {
                let n = self.variables.get(name);
                match n {
                    Some(Value::Function(fname, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        let mut selfclone = self.clone();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(selfclone.visit_expr(a), &self.position));
                        }
                        // if f.call_count > MONITOR_THRESHOLD {
                        //     monitor_fn();
                        // }
                        // if f.call_count > JIT_THRESHOLD {
                        //     // return Ok(jitfn(&mut self.clone(), myargs))
                        // }
                        let c = uoe(f.clone().call_(self, myargs), &self.position);
                        // self.variables.insert(name, Value::Function(fname.to_owned(), VMFunction{decl:f.decl.clone(), call_count:f.call_count+1}));
                        Ok(c)
                    }
                    Some(Value::NativeFunction(_x, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        let mut selfclone = self.clone();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(selfclone.visit_expr(a), &self.position));
                        }
                        let c = uoe(f(myargs, self), &self.position);
                        Ok(c)
                    }
                    Some(Value::DynFn(name, file, arity)) => {
                        let myargs: Vec<Value> = Vec::new();
                        let mut mylyargs: Vec<*mut crate::ffi::LyValue> = Vec::new();
                        let mut selfclone = self.clone();
                        unsafe{
                            let lib = libloading::Library::new(file).unwrap();
                            for (_i, a) in args.into_iter().enumerate() {
                                let arg: *mut crate::ffi::LyValue = &mut crate::ffi::rust_to_c_lyvalue(
                                    uoe(selfclone.visit_expr(a), &self.position)
                                );
                                mylyargs.push(arg);
                            }
                            let func: libloading::Symbol<DynFn>
                                = lib.get(name.as_str().as_bytes()).unwrap();

                            Ok(crate::ffi::c_to_lyvalue(
                                func(*arity, mylyargs.as_mut_ptr())
                            ))
                        }
                    }
                    Some(Value::Class(n, cl, _)) => {
                        let obj_pos = self.objects.len();
                        let mut myargs: Vec<Value> = vec![Value::Object(
                            n.clone(),
                            cl.clone(),
                            HashMap::new(),
                            obj_pos,
                        )];
                        self.objects.push(Some(myargs[0].clone()));
                        let mut selfclone = self.clone();
                        // self.objects.push(Some(myargs[0].clone()));
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(selfclone.visit_expr(a), &self.position));
                        }
                        match cl.get(n) {
                            Some(f) => {
                                if f.arity() == 0 {
                                    uoe(f.call_(&mut self.clone(), vec![]), &self.position);
                                    self.objects.push(Some(myargs[0].clone()));
                                    Ok(match &self.objects[self.objects.len() - 1] {
                                        Some(s) => s.clone(),
                                        None => unreachable!(),
                                    })
                                } else {
                                    let objcons = uoe(
                                        f.call_(&mut self.clone(), myargs),
                                        &self.position,
                                    );
                                    if let Value::Object(_, _, _, _) = objcons {
                                        self.objects.push(Some(objcons));
                                        Ok(match &self.objects[self.objects.len() - 1] {
                                            Some(s) => s.clone(),
                                            None => unreachable!(),
                                        })
                                    } else {
                                        println!("{:?}", objcons);
                                        Err(VMError {
                                            type_: "InvalidConstructor".to_string(),
                                            cause: "Constructor must return an Object".to_string(),
                                        })
                                    }
                                }
                            }
                            None => {
                                let obj = Value::Object(
                                    n.to_string(),
                                    cl.clone(),
                                    HashMap::new(),
                                    self.objects.len(),
                                );
                                self.objects.push(Some(obj));
                                Ok(match &self.objects[self.objects.len() - 1] {
                                    Some(s) => s.clone(),
                                    None => unreachable!(),
                                })
                            }
                        }
                    }
                    Some(_) => {
                        panic!("Wahhahahhaha");
                    }
                    None => Err(VMError {
                        type_: "UnderclaredVariable".to_string(),
                        cause: "No fn".to_string(),
                    }),
                }
            }
            ExprValue::UnOp(op, expr) => match **op {
                TokenType::Plus => self.visit_expr(&*expr),
                TokenType::Minus => Ok(Value::Float64(
                    (-1_f64) * f64::try_from(uoe(self.visit_expr(&*expr), &self.position)).unwrap(),
                )),
                TokenType::Not => Ok(Value::Boolean(!bool::from(uoe(
                    self.visit_expr(&*expr),
                    &self.position,
                )))),
                _ => Err(VMError {
                    type_: "OperatorError".to_string(),
                    cause: "Invalid op".to_string(),
                }),
            },
            ExprValue::BinOp(lhs, op, rhs) => Ok(match **op {
                TokenType::Plus => match ((**lhs).clone(), (**rhs).clone()) {
                    (ExprValue::Str(_), _) | (_, ExprValue::Str(_)) => Value::Str(
                        uoe(self.visit_expr(&*lhs), &self.position)
                            .to_string()
                            + &uoe(self.visit_expr(&*rhs), &self.position)
                                .to_string(),
                    ),
                    _ => Value::Float64(
                        f64::try_from(uoe(self.visit_expr(&*lhs), &self.position))
                            .unwrap()
                            + f64::try_from(uoe(self.visit_expr(&*rhs), &self.position))
                                .unwrap(),
                    ),
                },
                TokenType::Minus => Value::Float64(
                    f64::try_from(uoe(self.visit_expr(&*lhs), &self.position)).unwrap()
                        - f64::try_from(uoe(self.visit_expr(&*rhs), &self.position))
                            .unwrap(),
                ),
                TokenType::Div => Value::Float64(
                    f64::try_from(uoe(self.visit_expr(&*lhs), &self.position)).unwrap()
                        / f64::try_from(uoe(self.visit_expr(&*rhs), &self.position))
                            .unwrap(),
                ),
                TokenType::Mul => Value::Float64(
                    f64::try_from(uoe(self.visit_expr(&*lhs), &self.position)).unwrap()
                        * f64::try_from(uoe(self.visit_expr(&*rhs), &self.position))
                            .unwrap(),
                ),
                TokenType::Less => Value::Boolean(
                    uoe(self.visit_expr(&*lhs), &self.position)
                        < uoe(self.visit_expr(&*rhs), &self.position),
                ),
                TokenType::Greater => Value::Boolean(
                    uoe(self.visit_expr(&*lhs), &self.position)
                        > uoe(self.visit_expr(&*rhs), &self.position),
                ),
                TokenType::LessEq => Value::Boolean(
                    uoe(self.visit_expr(&*lhs), &self.position)
                        <= uoe(self.visit_expr(&*rhs), &self.position),
                ),
                TokenType::GreaterEq => Value::Boolean(
                    uoe(self.visit_expr(&*lhs), &self.position)
                        >= uoe(self.visit_expr(&*rhs), &self.position),
                ),
                TokenType::Equal => Value::Boolean(
                    uoe(self.visit_expr(&*lhs), &self.position)
                        == uoe(self.visit_expr(&*rhs), &self.position),
                ),
                TokenType::Dot => {
                    let obj = uoe(self.visit_expr(&*lhs), &self.position);
                    match obj.clone() {
                        Value::Object(_n, c, a, _) => {
                            match (**rhs).clone() {
                                ExprValue::Identifier(n) => match c.get(&n) {
                                    Some(s) => {
                                        return Ok(Value::Function(n, s.clone()));
                                    }
                                    None => match a.get(&n) {
                                        Some(expr) => return Ok(expr.clone()),
                                        None => {
                                            return Err(VMError {
                                                type_: "InvalidInvocation".to_string(),
                                                cause: "Not founf".to_string(),
                                            })
                                        }
                                    },
                                },
                                ExprValue::FnCall(n, args) => match c.get(&n) {
                                    Some(s) => {
                                        let mut myargs = Vec::new();
                                        if !s.decl.args.name.is_empty() {
                                            myargs.push(obj); // self
                                        }
                                        for (_i, mut a) in args.into_iter().enumerate() {
                                            myargs
                                                .push(uoe(self.visit_expr(&mut a), &self.position));
                                        }
                                        let c = uoe(
                                            s.call_(self, myargs),
                                            &self.position,
                                        );
                                        return Ok(c);
                                    }
                                    None => {
                                        println!("err");
                                        return Err(VMError {
                                            type_: "InvalidInvocation".to_string(),
                                            cause: "IDK".to_string(),
                                        });
                                    }
                                },
                                _ => {
                                    return Err(VMError {
                                        type_: "InvalidInvocation".to_string(),
                                        cause: format!("Tried to invoke {obj}, which is not an object or a dict.").to_string(),
                                    })
                                }
                            }
                        },
                        Value::Dict(mut d) => {
                            if let ExprValue::Identifier(i) = (**rhs).clone() {
                                match d.get(&i) {
                                    Some(expr) => return Ok(expr.clone()),
                                    None => return Ok(Value::None),
                                }
                            // }else if let ExprValue::Assign { name, value } = (**rhs).clone() {
                            //         let val = uoe(self.visit_expr(&*value), &self.position);
                            //         d.insert(name.to_string(), val.clone());
                            //         return Ok(val)
                            } else {
                                return Err(VMError {
                                    type_: "InvalidInvocation".to_string(),
                                    cause: format!("Tried to invoke {:?}, which is not an object or a dict.", obj).to_string(),
                                });
                            }
                        },
                        _=>{
                          return Err(VMError {
                              type_: "InvalidInvocation".to_string(),
                              cause: "IDK".to_string(),
                          });  
                        }
                    }
                }
                // TokenType::Walrus => {
                //     let obj = self.visit_expr(&mut *lhs).unwrap();
                //     println!("rsh{:?}", rhs);
                //     match *rhs {
                //         // ExprValue::BinOp=>{}
                //         _ => {}
                //     }
                //     todo!();
                // }
                _ => todo!(),
            }),
            ExprValue::Boolean(b) => Ok(Value::Boolean(*b)),
            ExprValue::None => Ok(Value::None),
            ExprValue::Integer(b) => Ok(Value::Float64(*b as f64)),
            ExprValue::Str(s) => Ok(Value::Str(s.clone())),
            ExprValue::Identifier(i) => match self.variables.get(i) {
                Some(expr) => Ok(expr.clone()),
                None => Err(VMError {
                    type_: "UnderclaredVariable".to_string(),
                    cause: i.to_string(),
                }),
            },
            ExprValue::VarDecl { name, type_: _ } => {
                if self.variables.get(name).is_some() {
                    return Err(VMError {
                        type_: "Redeclaration".to_string(),
                        cause: name.to_string(),
                    });
                }
                self.variables.insert(name.to_string(), Value::Int32(0));
                Ok(Value::Int32(0))
            }
            ExprValue::IfElse { cond, if_, else_ } => {
                if bool::from(uoe(self.visit_expr(&*cond), &self.position)) {
                    let mut retval: Result<Value> =
                        Ok(uoe(self.visit_expr(&*cond), &self.position));
                    for ex in &(*if_) {
                        retval = self.visit_expr(&ex);
                    }
                    retval
                } else {
                    let mut retval: Result<Value> = Ok(self.visit_expr(&*cond).unwrap());
                    for ex in &(*else_) {
                        retval = self.visit_expr(&ex);
                    }
                    retval
                }
            }
            ExprValue::Assign { name, value } => {
                let val = uoe(self.visit_expr(&*value), &self.position);
                self.variables.insert(name.to_string(), val.clone());
                Ok(val)
            }
            ExprValue::Use(path) => {
                if &path[..4] == "std:" {
                    let lexer = unwrap_or_exit!(
                        Lexer::from_file(&("iorekfiles/".to_owned() + &path[4..] + ".lyr")),
                        "IO"
                    );
                    let tokens = lexer
                        .map(|t| unwrap_or_exit!(t, "Lexing"))
                        .collect::<Vec<_>>();
                    let mut parser = Parser::new(
                        tokens.into_iter().peekable(),
                        &("iorekfiles/".to_owned() + &path[4..] + ".lyr"),
                    );
                    let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
                    let mut visitor = Visitor::new();
                    visitor.init();
                    visitor.visit_program(program);
                    self.variables.extend(visitor.variables.clone());
                    return Ok(Value::None);
                }
                if &path[..2] == "@:"
                 {
                    let lexer = unwrap_or_exit!(
                        Lexer::from_file(&("iorekfiles/external/".to_owned() + &path[2..] + ".lyr")),
                        "IO"
                    );
                    let tokens = lexer
                        .map(|t| unwrap_or_exit!(t, "Lexing"))
                        .collect::<Vec<_>>();
                    let mut parser = Parser::new(
                        tokens.into_iter().peekable(),
                        &("iorekfiles/external/".to_owned() + &path[2..] + ".lyr"),
                    );
                    let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
                    let mut visitor = Visitor::new();
                    visitor.init();
                    visitor.visit_program(program);
                    self.variables.extend(visitor.variables.clone());
                    return Ok(Value::None);
                }
                let lexer = unwrap_or_exit!(Lexer::from_file(path), "IO");
                let tokens = lexer
                    .map(|t| unwrap_or_exit!(t, "Lexing"))
                    .collect::<Vec<_>>();
                let mut parser = Parser::new(tokens.into_iter().peekable(), &path);
                let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
                let mut visitor = Visitor::new();
                visitor.init();
                visitor.visit_program(program);
                self.variables.extend(visitor.variables.clone());
                Ok(Value::None)
            }
            ExprValue::Extern(path) => {
                unsafe{
                    if &path[..4] == "std:" {
                        let file = std::fs::read_to_string(path[4..].to_string()+".json")
                            .unwrap();
                        let v: SerdeValue = serde_json::from_str(&file).unwrap();
                        // println!("{:?}", v)
                        let mut functions: Vec<(String, u32, DynFn)> = vec![];
                        let lib = libloading::Library::new(path[4..].to_string()+".so").unwrap();

                        match &v["functions"] {
                            SerdeValue::Array(a)=>{
                                for x in a {
                                    if let (SerdeValue::String(s), SerdeValue::Number(n)) = (&x["name"], &x["arity"]) {
                                        if let Some(arity) = n.as_i64() {
                                            let func: libloading::Symbol<DynFn>
                                                = lib.get(s.as_str().as_bytes()).unwrap();

                                            self.variables.insert(
                                                s.to_string(),
                                                Value::DynFn(s.to_string(), (path[4..].to_string()+".so").to_string(), arity.try_into().unwrap())
                                            );

                                            functions.push((s.to_string(), arity as u32, *func));
                                        }
                                    }
                                }
                            }
                            _=>todo!()
                        }
                                                
                        println!("functions {:?}", functions);
                        return Ok(Value::None);
                    }
                    if &path[..2] == "@:"
                     {
                        return Ok(Value::None);
                    }
                    Ok(Value::None)
                }
            }
            ExprValue::While(cond, exprs) => {
                let mut retval: Result<Value> = Ok(Value::None);
                let mut exec_count = 0;

                while bool::from(uoe(self.visit_expr(&*cond), &self.position)) /*&& exec_count < MONITOR_THRESHOLD */{
                    for expr in &*exprs {
                        retval = self.visit_expr(&expr);
                    }
                    exec_count+=1;
                }

                if exec_count < MONITOR_THRESHOLD {// loop exit
                    return retval
                }
                // control comes here if cond is true and monitoring shall start.
                // todo
                retval
                
            }
            // ExprValue::Walrus(obj, attr, val) => {
            //     let obj =self.visit_expr(&mut *obj).unwrap();
            //     match obj {
            //         Value::Object(n, c, mut a) => {
            //             a.insert(n.clone(), self.visit_expr(&mut *val).unwrap());
            //             Ok(Value::Object(n.to_string(), c, a))
            //         },
            //         _ => return Err(VMError {
            //             type_: "ValueError".to_string(),
            //             cause: "IDDDKKK".to_string(),
            //         }),
            //     }
            // }
            x => panic!("{:?}", x),
        }
    }
    pub fn monitor_fn(args: Vec<Value>) -> bool {
        for arg in args {
            // if let Some(pat) = expr {
            //     expr
            // }
            return true
        }
        return true
    }
}

fn base_obj_hashmap() -> HashMap<String, VMFunction> {
    
    // base.insert("getattr".to_string(), Value::NativeFunction("getattr".to_string(), stdlib::__getattr));
    // base.insert("setattr".to_string(), Value::NativeFunction("setattr".to_string(), stdlib::__setattr));
    HashMap::new()
}
