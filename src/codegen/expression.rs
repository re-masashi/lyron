use crate::codegen::{Callable, VMError, VMFunction, Value, Visitor, uoe};
use crate::lexer::tokens::TokenType;
use crate::lexer::Lexer;
use crate::parser::{ExprValue, Parser};
use log::error;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::process;

type Result<T> = std::result::Result<T, VMError>;

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
    pub fn visit_expr(&mut self, expr: ExprValue) -> Result<Value> {
        match expr {
            ExprValue::FnCall(name, args) => {
                let n = self.variables.get(&name);
                match n {
                    Some(Value::Function(_n, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(self.clone().visit_expr(a), &self.position));
                        }
                        let c = uoe(f.clone().call_(&mut self.clone(), myargs), &self.position);
                        // std::mem::replace(self, i);
                        Ok(c)
                    }
                    Some(Value::NativeFunction(_x, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(uoe(self.clone().visit_expr(a), &self.position));
                        }
                        let c = uoe(f.clone()(myargs, self), &self.position);
                        Ok(c)
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
                        // self.objects.push(Some(myargs[0].clone()));
                        for (_i, a) in args.clone().into_iter().enumerate() {
                            myargs.push(uoe(self.clone().visit_expr(a), &self.position));
                        }
                        match cl.get(n) {
                            Some(f) => {
                                if f.clone().arity() == 0 {
                                    uoe(f.clone().call_(&mut self.clone(), vec![]), &self.position);
                                    self.objects.push(Some(myargs[0].clone()));
                                    Ok(match &self.objects[self.objects.len() - 1] {
                                        Some(s) => s.clone(),
                                        None => unreachable!(),
                                    })
                                } else {
                                    let objcons =
                                        uoe(f.clone().call_(&mut self.clone(), myargs.clone()), &self.position);
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
            ExprValue::UnOp(op, expr) => match *op {
                TokenType::Plus => self.visit_expr(*expr),
                TokenType::Minus => Ok(Value::Float64(
                    (-1_f64) * f64::try_from(uoe(self.visit_expr(*expr), &self.position)).unwrap(),
                )),
                TokenType::Not => Ok(Value::Boolean(!bool::from(uoe(self.visit_expr(*expr), &self.position)))),
                _ => Err(VMError {
                    type_: "OperatorError".to_string(),
                    cause: "Invalid op".to_string(),
                }),
            },
            ExprValue::BinOp(lhs, op, rhs) => Ok(match *op {
                TokenType::Plus => match ((*lhs).clone(), (*rhs).clone()) {
                    (ExprValue::Str(_), _) | (_, ExprValue::Str(_)) => Value::Str(
                        uoe(self.visit_expr(*lhs), &self.position)
                            .clone()
                            .to_string()
                            .to_owned()
                            + &uoe(self.visit_expr(*rhs), &self.position)
                                .clone()
                                .to_string()
                                .to_owned(),
                    ),
                    _ => Value::Float64(
                        f64::try_from(uoe(self.visit_expr((*lhs).clone()), &self.position)).unwrap()
                            + f64::try_from(uoe(self.visit_expr((*rhs).clone()), &self.position)).unwrap(),
                    ),
                },
                TokenType::Minus => Value::Float64(
                    f64::try_from(uoe(self.visit_expr((*lhs).clone()), &self.position)).unwrap()
                        - f64::try_from(uoe(self.visit_expr((*rhs).clone()), &self.position)).unwrap(),
                ),
                TokenType::Div => Value::Float64(
                    f64::try_from(uoe(self.visit_expr((*lhs).clone()), &self.position)).unwrap()
                        / f64::try_from(uoe(self.visit_expr((*rhs).clone()), &self.position)).unwrap(),
                ),
                TokenType::Mul => Value::Float64(
                    f64::try_from(uoe(self.visit_expr((*lhs).clone()), &self.position)).unwrap()
                        * f64::try_from(uoe(self.visit_expr((*rhs).clone()), &self.position)).unwrap(),
                ),
                TokenType::Less => {
                    Value::Boolean(uoe(self.visit_expr(*lhs), &self.position) < uoe(self.visit_expr(*rhs), &self.position))
                }
                TokenType::Greater => {
                    Value::Boolean(uoe(self.visit_expr(*lhs), &self.position) > uoe(self.visit_expr(*rhs), &self.position))
                }
                TokenType::LessEq => {
                    Value::Boolean(uoe(self.visit_expr(*lhs), &self.position) <= uoe(self.visit_expr(*rhs), &self.position))
                }
                TokenType::GreaterEq => {
                    Value::Boolean(uoe(self.visit_expr(*lhs), &self.position) >= uoe(self.visit_expr(*rhs), &self.position))
                }
                TokenType::Equal => {
                    Value::Boolean(uoe(self.visit_expr(*lhs), &self.position) == uoe(self.visit_expr(*rhs), &self.position))
                }
                TokenType::Dot => {
                    let obj =uoe(self.visit_expr(*lhs), &self.position) ;
                    if let Value::Object(_n, c, a, _) = obj.clone() {
                        match *rhs {
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
                                        myargs.push(obj.clone()); // self
                                    }
                                    for (_i, a) in args.into_iter().enumerate() {
                                        myargs.push(uoe(self.clone().visit_expr(a), &self.position));
                                    }
                                    let c = uoe(s.clone().call_(&mut self.clone(), myargs), &self.position);
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
                                    cause: "IDK".to_string(),
                                })
                            }
                        }
                    } else {
                        return Err(VMError {
                            type_: "InvalidInvocation".to_string(),
                            cause: "IDK".to_string(),
                        });
                    }
                }
                // TokenType::Walrus => {
                //     let obj = self.visit_expr(*lhs).unwrap();
                //     println!("rsh{:?}", rhs);
                //     match *rhs {
                //         // ExprValue::BinOp=>{}
                //         _ => {}
                //     }
                //     todo!();
                // }
                _ => todo!(),
            }),
            ExprValue::Boolean(b) => Ok(Value::Boolean(b)),
            ExprValue::Integer(b) => Ok(Value::Float64(b as f64)),
            ExprValue::Str(s) => Ok(Value::Str(s.clone())),
            ExprValue::Identifier(i) => match self.variables.get(&i) {
                Some(expr) => Ok(expr.clone()),
                None => Err(VMError {
                    type_: "UnderclaredVariable".to_string(),
                    cause: i,
                }),
            },
            ExprValue::VarDecl { name, type_: _ } => {
                if self.variables.get(&name).is_some() {
                    return Err(VMError {
                        type_: "Redeclaration".to_string(),
                        cause: name,
                    });
                }
                self.variables.insert(name, Value::Int32(0));
                Ok(Value::Int32(0))
            }
            ExprValue::IfElse { cond, if_, else_ } => {
                if bool::from(uoe(self.visit_expr((*cond).clone()), &self.position)) {
                    let mut retval: Result<Value> = Ok(uoe(self.visit_expr((*cond).clone()), &self.position));
                    for ex in &(*if_) {
                        retval = self.visit_expr(ex.clone());
                    }
                    retval
                } else {
                    let mut retval: Result<Value> = Ok(self.visit_expr((*cond).clone()).unwrap());
                    for ex in &(*else_) {
                        retval = self.visit_expr(ex.clone());
                    }
                    retval
                }
            }
            ExprValue::Assign { name, value } => {
                let val = uoe(self.visit_expr(*value.clone()), &self.position);
                self.variables.insert(name, val.clone());
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
                    let mut parser = Parser::new(tokens.into_iter().peekable(), &("iorekfiles/".to_owned() + &path[4..] + ".lyr"));
                    let program = unwrap_or_exit!(parser.parse_program(), "Parsing");
                    let mut visitor = Visitor::new();
                    visitor.init();
                    visitor.visit_program(program);
                    self.variables.extend(visitor.variables.clone());
                    return Ok(Value::None);
                }
                let lexer = unwrap_or_exit!(Lexer::from_file(&path), "IO");
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
            ExprValue::While(cond, exprs) => {
                let mut retval: Result<Value> = Ok(Value::None);
                while bool::from(uoe(self.visit_expr((*cond).clone()), &self.position)) {
                    for expr in &exprs {
                        retval = self.visit_expr(expr.clone());
                    }
                }
                retval
            }
            // ExprValue::Walrus(obj, attr, val) => {
            //     let obj =self.visit_expr(*obj).unwrap();
            //     match obj {
            //         Value::Object(n, c, mut a) => {
            //             a.insert(n.clone(), self.visit_expr(*val).unwrap());
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
}

fn base_obj_hashmap() -> HashMap<String, VMFunction> {
    let base = HashMap::new();
    // base.insert("getattr".to_string(), Value::NativeFunction("getattr".to_string(), stdlib::__getattr));
    // base.insert("setattr".to_string(), Value::NativeFunction("setattr".to_string(), stdlib::__setattr));
    base
}
