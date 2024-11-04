use crate::codegen::{uoe, Callable, VMError, Value, Visitor};
use crate::lexer::tokens::TokenType;
use crate::lexer::Lexer;
use crate::parser::{ExprValue, Parser};
use log::error;
use gxhash::{HashMap, HashMapExt};
use std::convert::TryFrom;
use std::process;
// use rayon::prelude::*;
// use serde_json::{Value as SerdeValue};
// use std::convert::TryInto;
// use std::borrow::{BorrowMut};

type Result<T> = std::result::Result<T, VMError>;
// type DynFn = unsafe extern fn(i32, *mut *mut crate::ffi::LyValue) -> *mut crate::ffi::LyValue;

const MONITOR_THRESHOLD: usize = 300;
const _JIT_THRESHOLD: usize = 700;

// macro_rules! call_ {
//     ($f:expr, $v: expr, $arguments: expr) => {

//         if $arguments.len() != $f.arity() {
//             panic!("Tried to call an invalid function")
//         } else {
//             let Function {
//                 name,
//                 args,
//                 expression,
//                 return_type: _,
//             } = $f.decl.borrow();

//             let v = $v.clone();
            
//             println!("Called {}", name);

//             for (i, arg) in args.name.clone().into_iter().enumerate() {
//                 v.variables.borrow_mut().insert(arg, $arguments[i].clone());
//             }

//             match v.visit_expr(&expression.clone().0) {
//                 Ok(v) => Ok(v),
//                 Err(e) => {
//                     println!("err {:?}", e);
//                     Err(e)
//                 }
//             }
//         }
//     }
// }

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
    pub fn visit_expr(&self, expr: &ExprValue) -> Result<Value> {
        // let mut stack = vec![];

        match expr {
            ExprValue::FnCall(name, args) => {
                let n_ = self.variables.borrow();
                let n = n_.get(name);
                match n {
                    Some(Value::Function(_fname, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        let selfclone = self;
                        for a in args {
                            myargs.push(uoe(selfclone.visit_expr(a), &self.position));
                        }
                        // if f.call_count > MONITOR_THRESHOLD {
                        //     monitor_fn();
                        // }
                        // if f.call_count > JIT_THRESHOLD {
                        //     // return Ok(jitfn(&mut self.clone(), myargs))
                        // }
                        // let c = uoe(f.clone().call_(self, myargs), &self.position);
                        // println!("before crash 2 {:?}", self.position);

                        Ok(uoe(
                            f.call_(self, myargs), 
                            &self.position
                        ))
                        // stack.push(uoe(f.clone().call_(self, myargs), &self.position));
                        // self.variables.borrow().insert(name, Value::Function(fname.to_owned(), VMFunction{decl:f.decl.clone(), call_count:f.call_count+1}));
                        // Ok(stack.pop().expect("unreachable"))
                    }
                    Some(Value::NativeFunction(_x, f)) => {
                        // println!("before crash{:?}", self.position);
                        let mut myargs: Vec<Value> = Vec::new();
                        
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(self.visit_expr(a), &self.position));
                        }
                        let c = uoe(f(myargs, self), &self.position);
                        Ok(c)
                    }
                    Some(Value::Class(n, cl, _)) => {
                        let obj_pos = self.objects.borrow().len();
                        let mut myargs: Vec<Value> = vec![Value::Object(
                            n.clone(),
                            cl.clone(),
                            HashMap::new(),
                            obj_pos,
                        )];
                        {self.objects.borrow_mut().push(Some(myargs[0].clone()));}
                        let selfclone = self;
                        // self.objects.borrow().push(Some(myargs[0].clone()));
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(selfclone.visit_expr(a), &self.position));
                        }
                        match cl.get(n) {
                            Some(f) => {
                                if f.arity() == 0 {
                                    uoe(f.call_(&self, vec![]), &self.position);
                                    self.objects.borrow_mut().push(Some(myargs[0].clone()));
                                    Ok(match &self.objects.borrow()[self.objects.borrow().len() - 1] {
                                        Some(s) => s.clone(),
                                        None => unreachable!(),
                                    })
                                } else {
                                    let objcons = uoe(
                                        f.call_(&self, myargs),
                                        &self.position,
                                    );
                                    if let Value::Object(_, _, _, _) = objcons {
                                        self.objects.borrow_mut().push(Some(objcons));
                                        Ok(match &self.objects.borrow()[self.objects.borrow().len() - 1] {
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
                                    self.objects.borrow().len(),
                                );
                                self.objects.borrow_mut().push(Some(obj));
                                Ok(match &self.objects.borrow()[self.objects.borrow().len() - 1] {
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
                        Value::Dict(d) => {
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
            ExprValue::Double(b) => Ok(Value::Float64(*b as f64)),
            ExprValue::Str(s) => Ok(Value::Str(s.clone())),
            ExprValue::Identifier(i) => match self.variables.borrow().get(i) {
                Some(expr) => Ok(expr.clone()),
                None => Err(VMError {
                    type_: "UnderclaredVariable".to_string(),
                    cause: i.to_string(),
                }),
            },
            ExprValue::VarDecl { name, type_: _ } => {
                if self.variables.borrow().get(name).is_some() {
                    return Err(VMError {
                        type_: "Redeclaration".to_string(),
                        cause: name.to_string(),
                    });
                }
                self.variables.borrow_mut().insert(name.to_string(), Value::Int32(0));
                Ok(Value::Int32(0))
            }
            ExprValue::IfElse { cond, if_, else_ } => {
                // println!("ifelse");
                if bool::from(uoe(self.visit_expr(&*cond), &self.position)) {
                    self.visit_expr(&*if_)
                } else {
                    self.visit_expr(&*else_)
                }
            }
            ExprValue::Assign { name, value } => {
                let val = uoe(self.visit_expr(&*value), &self.position);
                self.variables.borrow_mut().insert(name.to_string(), val.clone());
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
                    self.variables.borrow_mut().extend(visitor.variables.borrow().clone());
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
                    self.variables.borrow_mut().extend(visitor.variables.borrow().clone());
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
                self.variables.borrow_mut().extend(visitor.variables.borrow().clone());
                Ok(Value::None)
            }
            ExprValue::While(cond, expr) => {
                let retval: Result<Value> = Ok(Value::None);
                let mut exec_count = 0;

                while bool::from(uoe(self.visit_expr(&*cond), &self.position)) /*&& exec_count < MONITOR_THRESHOLD */
                {
                    // println!("loopin");
                    // println!("{:?}", cond);
                    let _ = self.visit_expr(&*expr);
                    exec_count+=1;
                }
                
                // println!("{:?}", cond);
                // println!("{:?}", self.variables.borrow().get("i"));

                if exec_count < MONITOR_THRESHOLD {// loop exit
                    return retval
                }
                // control comes here if cond is true and monitoring shall start.
                // todo
                retval               
            }
            ExprValue::Do(expressions) => {
                // println!("doin");
                let mut retval = Ok(Value::None);
                for expr in expressions {
                    retval = self.visit_expr(&expr);
                    // println!("ret {:?} {:?}", retval, expr);
                }
                // println!("ret final {:?}", retval);
                retval
            }
            ExprValue::Array(arr)=>{
                let mut exprs = vec![];
                for x in arr {
                    exprs.push(self.visit_expr(x).unwrap());
                }
                Ok(Value::Array(exprs))
            }
            x => panic!("{:?}", x),
        }
    }
    
    pub fn monitor_fn(args: Vec<Value>) -> bool {
        for _arg in args {
            // if let Some(pat) = expr {
            //     expr
            // }
            return true
        }
        return true
    }
}