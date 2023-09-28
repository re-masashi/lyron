use crate::codegen::{Callable, VMError, Value, Visitor};
use crate::lexer::tokens::TokenType;
use crate::lexer::Lexer;
use crate::parser::{ExprValue, Parser};
use log::error;
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

impl Visitor {
    pub fn visit_expr(&mut self, expr: ExprValue) -> Result<Value> {
        match expr {
            ExprValue::FnCall(name, args) => {
                let n = self.variables.get(&name);
                match n {
                    Some(Value::Function(_n, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(self.clone().visit_expr(a).unwrap());
                        }
                        let c = f.clone().call_(&mut self.clone(), myargs).unwrap();
                        // std::mem::replace(self, i);
                        return Ok(c);
                    }
                    Some(Value::NativeFunction(_n, f)) => {
                        let mut myargs: Vec<Value> = Vec::new();
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(self.clone().visit_expr(a).unwrap());
                        }
                        let c = f.clone()(myargs.len() as i32, myargs);
                        return Ok(c);
                    }
                    Some(Value::Class(n, cl))=>{
                        let mut myargs: Vec<Value> = vec![Value::Object(n.clone(), cl.clone())];
                        for (_i, a) in args.into_iter().enumerate() {
                            myargs.push(self.clone().visit_expr(a).unwrap());
                        }
                        match cl.get(n) {
                            Some(f) => f.clone().call_(&mut self.clone(), myargs.clone()),
                            None => return Ok(Value::Object(n.to_string(), cl.clone())),
                        };
                        return Ok(myargs[0].clone());
                    }
                    Some(_) => {
                        panic!("Wahhahahhaha");
                    }
                    None => {
                        return Err(VMError {
                            type_: "UnderclaredVariable".to_string(),
                            cause: "No fn".to_string(),
                        })
                    }
                }
            }
            ExprValue::UnOp(op, expr) => match *op {
                TokenType::Plus => return self.visit_expr(*expr),
                TokenType::Minus => {
                    return Ok(Value::Float64(
                        (-1 as f64) * f64::try_from(self.visit_expr(*expr).unwrap()).unwrap(),
                    ));
                }
                TokenType::Not => {
                    return Ok(Value::Boolean(!bool::from(self.visit_expr(*expr).unwrap())))
                }
                _ => {
                    return Err(VMError {
                        type_: "OperatorError".to_string(),
                        cause: "Invalid op".to_string(),
                    })
                }
            },
            ExprValue::BinOp(lhs, op, rhs) => Ok(match *op {
                TokenType::Plus => match ((*lhs).clone(), (*rhs).clone()) {
                    (ExprValue::Str(_),_) | (_, ExprValue::Str(_)) => {
                        Value::Str(self.visit_expr(*lhs).unwrap().clone().to_string().to_owned()+&self.visit_expr(*rhs).unwrap().clone().to_string().to_owned())
                    }
                    _ => Value::Float64(
                        f64::try_from(self.visit_expr((*lhs).clone()).unwrap()).unwrap()
                            + f64::try_from(self.visit_expr((*rhs).clone()).unwrap()).unwrap(),
                    ),
                },
                TokenType::Minus => Value::Float64(
                    f64::try_from(self.visit_expr(*lhs).unwrap()).unwrap()
                        - f64::try_from(self.visit_expr(*rhs).unwrap()).unwrap(),
                ),
                TokenType::Div => Value::Float64(
                    f64::try_from(self.visit_expr(*lhs).unwrap()).unwrap()
                        / f64::try_from(self.visit_expr(*rhs).unwrap()).unwrap(),
                ),
                TokenType::Mul => Value::Float64(
                    f64::try_from(self.visit_expr(*lhs).unwrap()).unwrap()
                        * f64::try_from(self.visit_expr(*rhs).unwrap()).unwrap(),
                ),
                TokenType::Less => {
                    Value::Boolean(self.visit_expr(*lhs).unwrap() < self.visit_expr(*rhs).unwrap())
                }
                TokenType::Greater => {
                    Value::Boolean(self.visit_expr(*lhs).unwrap() > self.visit_expr(*rhs).unwrap())
                }
                TokenType::LessEq => {
                    Value::Boolean(self.visit_expr(*lhs).unwrap() <= self.visit_expr(*rhs).unwrap())
                }
                TokenType::GreaterEq => {
                    Value::Boolean(self.visit_expr(*lhs).unwrap() >= self.visit_expr(*rhs).unwrap())
                }
                TokenType::Equal => {
                    Value::Boolean(self.visit_expr(*lhs).unwrap() == self.visit_expr(*rhs).unwrap())
                }
                TokenType::Dot => {
                    let classname: String;
                    if let ExprValue::Identifier(i) = *lhs{
                        classname = i;
                    }else{
                        return Err(VMError{type_:"InvalidInvocation".to_string(), cause: "IDK".to_string()});
                    }
                    let class = self.variables.get(&classname);
                    if let Some(Value::Class(_n, c)) = class  {
                        match *rhs {
                            ExprValue::Identifier(n)=>{
                                match c.get(&n){
                                    Some(s)=>{
                                        return Ok(Value::Function("".to_string(),s.clone()));
                                    },
                                    None=> {return Err(VMError{type_:"InvalidInvocation".to_string(), cause: "IDK".to_string()});}
                                }
                            }
                            ExprValue::FnCall(n, args)=>{
                                match c.get(&n){
                                    Some(s)=>{
                                        let mut myargs = Vec::new();
                                        for (_i, a) in args.into_iter().enumerate() {
                                            myargs.push(self.clone().visit_expr(a).unwrap());
                                        }
                                        let c = s.clone().call_(&mut self.clone(), myargs).unwrap();
                                        return Ok(c);
                                    },
                                    None=> {return Err(VMError{type_:"InvalidInvocation".to_string(), cause: "IDK".to_string()});}
                                }
                            }
                            _=>return Err(VMError{type_:"InvalidInvocation".to_string(), cause: "IDK".to_string()})
                        }
                    }else{
                        return Err(VMError{type_:"InvalidInvocation".to_string(), cause: "IDK".to_string()})
                    }
                }
                _ => todo!(),
            }),
            ExprValue::Boolean(b) => return Ok(Value::Boolean(b)),
            ExprValue::Integer(b) => return Ok(Value::Float64(b as f64)),
            ExprValue::Str(s) => return Ok(Value::Str(s.clone())),
            ExprValue::Identifier(i) => match self.variables.get(&i) {
                Some(expr) => {
                    Ok(expr.clone())
                },
                None => Err(VMError {
                    type_: "UnderclaredVariable".to_string(),
                    cause: i,
                }),
            },
            ExprValue::VarDecl { name, type_: _ } => {
                if self.variables.get(&name) != None {
                    return Err(VMError {
                        type_: "Redeclaration".to_string(),
                        cause: name,
                    });
                }
                self.variables.insert(name, Value::Int32(0));
                Ok(Value::Int32(0))
            }
            ExprValue::IfElse { cond, if_, else_ } => {
                if bool::from(self.visit_expr((*cond).clone()).unwrap()) {
                    let mut retval: Result<Value> = Ok(self.visit_expr((*cond).clone()).unwrap());
                    for ex in &(*if_) {
                        retval = self.visit_expr(ex.clone());
                    }
                    return retval;
                } else {
                    let mut retval: Result<Value> = Ok(self.visit_expr((*cond).clone()).unwrap());
                    for ex in &(*else_) {
                        retval = self.visit_expr(ex.clone());
                    }
                    return retval;
                }
            }
            ExprValue::Assign { name, value } => {
                let val = self.visit_expr(*value.clone()).unwrap();
                self.variables.insert(name, val.clone());
                return Ok(val);
            }
            ExprValue::Use(path) => {
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
                return Ok(Value::None);
            }
            x => {
                panic!("{:?}", x);
            }
        }
    }
}
