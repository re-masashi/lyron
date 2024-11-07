use crate::codegen::{uoe, VMError, Visitor, Value};
use crate::parser::{AstNode, NodePosition, Function, ExprValue};
use crate::lexer::tokens::TokenType;
use gxhash::{HashMap, HashMapExt};
use std::convert::TryFrom;

pub type Result<T> = std::result::Result<T, VMError>;

// clone only for compile-time usage in Chunk
#[derive(Debug, Clone)]
enum OpCode {
    Push,
    Pop,
    Return,
    Call(u8), // index of name of function
    Args(u8),
    // CallNative(u8), // no of args
    Jump(u8),
    JumpIfFalse(u8),

    Constant(u8), // index of the constant
    
    Add,
    Sub,
    Mul,
    Div,
    Less,
    Greater,
    LessEq,
    GreaterEq,
    Equal,
    Concat,

    Load(u8), // index of variable name (in the constants)
    Assign(u8), // index of variable name (in the constants)
    VarDecl,
    FunctionDef(u8), // no of instructions in the function

    DefineArray(u8), // length of the initial elements

    // scope start and scope end to free values once they are out of scope.
    // is this a bad idea?
    // moreover, the return value of a do expression is needed.

    Do, // scope start
    End, // scope end
}

pub struct VM<'a> {
    constants: Vec<Value>,
    variables: HashMap<String, Value>,
    chunk: Box<Chunk<'a>>,
    ip: usize, // instruction pointer
    stack: Vec<Value>, // 64 is enough for now I guess
    callstack: Vec<usize>, // pop it to get the address of where you're supposed to return to
    functions: HashMap<String, (usize, Vec<String>)>, // (function_name -> (index of jump, list of arg names)) functions are just... jumps
}

// clone only for compile-time usage
// is there any way i avoid that as well
#[derive(Debug, Clone)]
struct Chunk<'a> {
    positions: Vec<(i32, i32, &'a str)>, // line, pos, file
    code: Vec<OpCode>
}

impl Visitor {
    pub fn visit_program(&mut self, astnodes: Vec<(AstNode, NodePosition)>) {
        let mut vm = VM {
            constants: vec![],
            variables: HashMap::new(),
            chunk: Box::new(Chunk{
                positions: vec![],
                code: vec![
                ]
            }),
            ip: 0,
            stack: vec![],
            callstack: vec![],
            functions: HashMap::new() // maybe store functions within variables by compiling the bytecode of a function?
        };

        // vm._gen_expr(
        //     &ExprValue::BinOp(
        //         Box::new(ExprValue::Double(20.0)),
        //         Box::new(TokenType::Greater),
        //         Box::new(ExprValue::Double(2.0)),
        //     )
        // );
        // vm._run();

        vm.init();

        for (node, pos) in astnodes {
            self.position = pos.clone();
            match node {
                AstNode::Expression(mut e) => {
                    // let _ = uoe(self.visit_expr(&mut e), &self.position);
                    vm._gen_expr(&e.clone());
                }
                AstNode::FunctionDef(f) => {
                    // let _ = self.visit_fn(f);
                    vm._gen_function(&f.clone());
                }
                AstNode::Class(c) => self.visit_class(c),
                _ => todo!(),
            }
        }
        // println!("");
        vm._run();
    }
}

impl<'a> VM<'a> {
    
    fn pop(&mut self) -> Value {
        // println!("chunk: {:?}", self.chunk);
        // println!("stack: {:?}", self.stack);
        // println!("");

        match self.stack.pop() {
            Some(x) => x,
            // Some(Value::LoxNativeError(s)) =>
            None => {
                panic!("VM panic! Attempted to pop a value when the value stack was empty")
            },
        }
    }

    // Note to future self: peek_mut SHOULD NEVER BE IMPLEMENTED!
    // Values on the stack are always implicit copy/cloned, any persistent values must be allocated with the Gc and represented with LoxPointers instead

    fn peek(&self) -> &Value {
        self.peek_at(0)
    }

    fn peek_at(&self, dist: usize) -> &Value {
        self.stack.get(self.stack.len() - dist - 1).unwrap()
    }

    pub fn init(&mut self) {
        self.variables.insert(
            "print".to_string(),
            Value::NativeFunction("print".to_string(), crate::codegen::stdlib::print),
        );
        self.variables.insert(
            "input".to_string(),
            Value::NativeFunction("input".to_string(), crate::codegen::stdlib::input),
        );
        self.variables.insert(
            "getattr".to_string(),
            Value::NativeFunction("getattr".to_string(), crate::codegen::stdlib::__getattr),
        );
        self.variables.insert(
            "setattr".to_string(),
            Value::NativeFunction("setattr".to_string(), crate::codegen::stdlib::__setattr),
        );
        self.variables.insert(
            "dict".to_string(),
            Value::NativeFunction("dict".to_string(), crate::codegen::stdlib::__dict),
        );
        self.variables.insert(
            "__dict_keys".to_string(),
            Value::NativeFunction("__dict_keys".to_string(), crate::codegen::stdlib::__dict_keys),
        );
        self.variables.insert(
            "startswith".to_string(),
            Value::NativeFunction("startswith".to_string(), crate::codegen::stdlib::__startswith),
        );
        self.variables.insert(
            "len".to_string(),
            Value::NativeFunction("len".to_string(), crate::codegen::stdlib::__len),
        );
        self.variables.insert(
            "array".to_string(),
            Value::NativeFunction("array".to_string(), crate::codegen::stdlib::__array),
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
            "start_tcp_server".to_string(),
            Value::NativeFunction("start_tcp_server".to_string(), crate::codegen::stdlib::start_tcp_server),
        );
        self.variables.insert(
            "read_file".to_string(),
            Value::NativeFunction("read_file".to_string(), crate::codegen::stdlib::read_file),
        );
        self.variables.insert(
            "write_file".to_string(),
            Value::NativeFunction("write_file".to_string(), crate::codegen::stdlib::write_file),
        );
        // self.variables.borrow_mut().insert(
        //     "exec".to_string(),
        //     Value::NativeFunction("exec".to_string(), crate::codegen::osutils::__exec),
        // );
        // self.variables.borrow_mut().insert(
        //     "socklisten".to_string(),
        //     Value::NativeFunction("socklisten".to_string(), crate::codegen::osutils::__socklisten),
        // );
    }

    pub fn _gen(&mut self, astnodes: Vec<(AstNode, NodePosition)>) {
        let mut chunk: Vec<OpCode> = vec![];
        let mut stack: Vec<Value> = vec![];

        for (node, pos) in astnodes {
            match node {
                AstNode::Expression(e) => {

                    // let _ = uoe(self.visit_expr(&mut e), &self.position);
                }
                AstNode::FunctionDef(f) => {
                }
                // AstNode::Class(c) => self.visit_class(c),
                _ => todo!(),
            }
        }
    }

    pub fn _gen_function(&mut self, func: &Function){
        // seems stupid. longer compile times but okay ig...
        let chunk_original = self.chunk.clone();

        self.chunk.code = vec![];

        let len_before = self.chunk.code.len();
        self._gen_expr(&func.expression.0);
        self.chunk.code.push(OpCode::Return);
        let len_after = self.chunk.code.len();

        let mut chunk_modified = self.chunk.clone(); // cloning again

        /*
        [
            ...,
            return 0;
            print(1+1);
            functionDef,
            ...
        ]
        */

        self.chunk = chunk_original;
        let ip = self.chunk.code.len();
        self.chunk.code.push(OpCode::FunctionDef((len_after - len_before)as u8)); // MAGIC! we know how long the function is before it is defined
        self.chunk.code.append(&mut chunk_modified.code);
        self.functions.insert(func.name.to_string(), (ip, func.args.name.clone()));
    }

    pub fn _gen_expr(&mut self, expr: &ExprValue) {
        // println!("chunk: {:?}", self.chunk);
        // println!("stack: {:?}", self.stack);
        // println!("expr: {:?}\n", expr);

        match expr {
            ExprValue::Integer(i) => {
                self.constants.push(Value::Float64(*i as f64));
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
            },
            ExprValue::Double(d) => {
                self.constants.push(Value::Float64(*d as f64));
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
            },
            ExprValue::Str(s) => {
                self.constants.push(Value::Str(s.to_string()));
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
            },
            ExprValue::Boolean(b) => {
                self.constants.push(Value::Boolean(*b));
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
            },
            ExprValue::None => {
                self.constants.push(Value::None);
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
            },
            ExprValue::BinOp(lhs, op, rhs) => match **op {
                // todo: add constant folding here
                TokenType::Plus => match ((**lhs).clone(), (**rhs).clone()) {
                    (ExprValue::Str(_), _) | (_, ExprValue::Str(_)) => {
                        self._gen_expr(rhs);
                        self._gen_expr(lhs);
                        self.chunk.code.push(OpCode::Concat);
                    }
                    _ => {
                        self._gen_expr(rhs);
                        self._gen_expr(lhs);
                        self.chunk.code.push(OpCode::Add);
                    }
                },
                TokenType::Minus => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Sub);
                }
                TokenType::Div => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Div);
                }
                TokenType::Mul => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Mul);
                }
                TokenType::Less => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Less);
                }
                TokenType::Greater => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Greater);
                }
                TokenType::LessEq => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::LessEq);
                }
                TokenType::GreaterEq => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::GreaterEq);
                }
                TokenType::Equal => {
                    self._gen_expr(rhs);
                    self._gen_expr(lhs);
                    self.chunk.code.push(OpCode::Equal);
                }
                _ => todo!(),
            },
            ExprValue::Assign { name, value } => {
                self._gen_expr(value);
                self.constants.push(Value::Str(name.to_string())); // feels wrong
                // self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::Assign((self.constants.len()-1) as u8));
                // length of self.constants > 2 here at all times
            }
            ExprValue::VarDecl { name, type_: _ } => {
                self.constants.push(Value::Str(name.to_string())); // feels wrong
                self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::VarDecl);
            }
            ExprValue::Do (expressions) => {
                for ex in expressions {
                    self._gen_expr(ex);
                }
            }
            ExprValue::Array (expressions) => {
                let len_before = self.chunk.code.len();
                for ex in expressions {
                    self._gen_expr(ex);
                }
                let len_after = self.chunk.code.len();

                // pop the next `expressions.len()` items from the stack
                // self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::DefineArray((len_after - len_before)as u8));
                // println!("consts {:?}", self.constants);
            }
            ExprValue::Identifier(i) => {
                self.constants.push(Value::Str(i.to_string()));
                self.chunk.code.push(OpCode::Load((self.constants.len()-1) as u8));
            }
            ExprValue::FnCall(name, args) => {
                // evaluate all args
                
                let len_before = self.chunk.code.len();
                for ex in args {
                    self._gen_expr(ex);
                }
                let len_after = self.chunk.code.len();

                self.constants.push(Value::Str(name.to_string())); // feels wrong
                // self.chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::Call((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::Args(args.len() as u8));
                
                // self.constants.push(name);
            }
            ExprValue::IfElse{cond, if_, else_} => {
                // println!("{:?}", if_);
                self._gen_expr(&cond);
                
                let chunk_original = self.chunk.clone();

                self.chunk.code = vec![];
                /*
                [   
                    ..., // len = a
                    cond,
                    jumpiffalse(idx),    |
                -- if starts here        | len' starts here
                    op1,                 |
                    op2,                 |
                    ...                  |
                    opn,                 | len' = n
                -- else starts here      | len '' starts here
                    op1,               <-|
                    op2,
                    op3
                    ...
                    opm                   // len'' = m
                ]
                */

                self.chunk.code = vec![];
                self._gen_expr(&if_);
                let mut if_chunk = self.chunk.clone();
                let if_len = self.chunk.code.len();
                
                self.chunk.code = vec![];
                self._gen_expr(&else_);
                let mut else_chunk = self.chunk.clone();
                let else_len = self.chunk.code.len();

                // self._gen_expr(&else_);
                // println!("{:?} {}", self.chunk, if_len);

                self.chunk = chunk_original;
                self.chunk.code.push(OpCode::JumpIfFalse(if_len as u8));
                self.chunk.code.append(&mut if_chunk.code);
                self.chunk.code.push(OpCode::Jump((if_len+1) as u8));
                self.chunk.code.append(&mut else_chunk.code);
                self.chunk.code.push(OpCode::Jump((else_len+1) as u8));
                // let ip = self.chunk.code.len();
                // self.chunk.code.append(&mut chunk_modified.code);
            }
            _=>todo!()
        }
    }

    pub fn _run(&mut self){
        // println!("chunk: {:?}", self.chunk);
        loop {
            // println!("ip: {:?}", self.ip);
            // println!("op: {:?}", (*self.chunk).code[self.ip]);
            // println!("stack: {:?}", self.stack);

            match (*self.chunk).code[self.ip] {
                OpCode::Constant(index)=>{
                    self.stack.push(self.constants[index as usize].clone());
                    // println!("const > {:?}", self.constants[index as usize]);
                }
                OpCode::Sub => {
                    let a = self.pop();
                    let b = self.pop();
                    // a - b
                    // println!("> {}", f64::try_from(a.clone()).unwrap()
                        // - f64::try_from(b.clone())
                            // .unwrap());
                    self.stack.push(Value::Float64(
                        f64::try_from(a).unwrap()
                            - f64::try_from(b)
                            .unwrap()
                    ));
                }
                OpCode::Add => {
                    let a = self.pop();
                    let b = self.pop();
                    // a + b
                    // println!("> {}", f64::try_from(a.clone()).unwrap()
                        // + f64::try_from(b.clone())
                            // .unwrap());
                    self.stack.push(Value::Float64(
                        f64::try_from(a).unwrap()
                            + f64::try_from(b)
                            .unwrap()
                    ));
                }
                OpCode::Mul => {
                    let a = self.pop();
                    let b = self.pop();
                    // a * b
                    // println!("> {}", f64::try_from(a).unwrap()
                        // * f64::try_from(b)
                            // .unwrap());
                    self.stack.push(Value::Float64(
                        f64::try_from(a).unwrap()
                            * f64::try_from(b)
                            .unwrap()
                    ));
                }
                OpCode::Div => {
                    let a = self.pop();
                    let b = self.pop();
                    // a / b
                    // println!("> {}", f64::try_from(a).unwrap()
                        // / f64::try_from(b)
                            // .unwrap());
                    self.stack.push(Value::Float64(
                        f64::try_from(a).unwrap()
                            / f64::try_from(b)
                            .unwrap()
                    ));
                }
                OpCode::Less => {
                    let a = self.pop();
                    let b = self.pop();
                    // a < b
                    // println!("> {}", a < b);
                    self.stack.push(Value::Boolean(a < b));
                }
                OpCode::Greater => {
                    let a = self.pop();
                    let b = self.pop();
                    // a > b
                    // println!("> {}", a > b);
                    self.stack.push(Value::Boolean(a > b));
                }
                OpCode::LessEq => {
                    let a = self.pop();
                    let b = self.pop();
                    // a <= b
                    // println!("{:?} <= {:?}", a, b);
                    // println!("> {}", a <= b);
                    self.stack.push(Value::Boolean(a <= b));
                }
                OpCode::GreaterEq => {
                    let a = self.pop();
                    let b = self.pop();
                    // a >= b
                    println!("> {}", a >= b);
                    self.stack.push(Value::Boolean(a >= b));
                }
                OpCode::Equal => {
                    // println!("{:?}", self.stack);

                    let a = self.pop();
                    let b = self.pop();
                    // a == b
                    println!("> {}", a == b);
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::DefineArray(len) => {
                    let mut exprs = vec![];
                    let values_len = self.constants.len()-1; 
                    
                    for i in 0..len as usize { 
                        exprs.push(self.pop());
                    }
                    // self.constants.push(Value::Array(exprs));
                    self.stack.push(Value::Array(exprs));
                    // println!("stack: {:?}", self.stack);
                    println!("> {:?}", self.pop()); // this should ALWAYS point to the newly created array
                }
                OpCode::Load(index) => {
                    let name = self.constants[index as usize].clone(); // hacky or bad?
                    // println!("{} {:?}", name, self.constants);
                    // println!("{:?}", self.variables);
                    let val = match self.variables.get(&name.to_string()) {
                        Some(expr) => expr,
                        None => panic!("no such variable! {}", name.to_string()), // todo: runtime error
                    };
                    self.stack.push(val.clone());
                    // println!("loaded {:?}", val);
                    // println!("stack: {:?}", self.stack);
                    // println!("> {:?}", self.pop()); // this should ALWAYS point to the newly created array
                }
                OpCode::Assign(index) => {
                    // im tired rn
                    let name = self.constants[index as usize].clone(); // hacky or bad?
                    let value = self.pop();
                    // println!("name: {:?}", name);
                    // println!("inserting {:?}", value);
                    self.variables.insert(name.to_string(), value.clone());
                    // println!("variables {:?}", self.variables);
                    self.stack.push(value);
                }
                OpCode::Call(idx) => {
                    // println!("calling");
                    // let mut exprs = vec![];
                    // [..., arg1, arg2, ..., argn, ...]
                    let fname = self.constants[idx as usize].to_string();
                    // let fn_name = self.constants[self.constants.len()-1].to_string(); // hacky or bad?

                    let mut ip = 0;
                    let original_ip = self.ip;
                    match self.functions.get(&fname){
                        Some(s)=>{
                            // println!("{:?}", s.1.clone());
                            // if len as usize!=s.1.len() {
                            //     panic!("incorrect number of args. expect {}, found {}", s.1.len(), len);
                            // }
                            ip = s.0;
                            // loads all the args
                            let mut argnames = s.1.clone();
                            argnames.reverse();
                            for argname in argnames {
                                // println!("idx {:?}", i);
                                // exprs.push(self.pop());

                                // let name = self.constants[(len-1-i) as usize].clone(); // hacky or bad?
                                {
                                    let value = self.pop();
                                    self.variables.insert(argname.to_string(), value);
                                }
                            }
                            self.callstack.push(self.ip);
                            self.ip = ip; // because we want to skip the args instruction
                            // println!("ip': {:?}", self.ip);
                            // println!("========={:?}", (*self.chunk).code[self.ip] );
                            // how do i return to where the stack was before calling?
                        }
                        None=>{
                            let fun = match self.variables.get(&fname) {
                                Some(Value::NativeFunction(name, f)) => {
                                    f.clone()
                                },
                                _ => panic!("NO SUCH FUNCTION {}", fname),
                            };
                            self.ip+=1;
                            // see the no of args
                            if let OpCode::Args(n) = (*self.chunk).code[self.ip] {
                                let mut args = vec![];
                                for x in 0..n {
                                    args.push(self.pop());
                                }
                                fun(args);
                                // println!("{:?}", fun(args));
                            }
                        },
                    }
                }
                OpCode::FunctionDef(len) => {
                    // println!("{:?}", len);
                    self.ip += len as usize;
                    // len is the addr of the last instruction in the function
                    // just after this match statement, ip is incremented by 1
                    // TL; DR: this just skips the function body
                }
                OpCode::Return => {
                    // println!("{:?}", self.constants);
                    self.ip = self.callstack.pop().expect("unreachable");
                }
                OpCode::JumpIfFalse(idx) => {
                    let cond = bool::from(self.pop());
                    // println!("{:?} {:?}", cond, (*self.chunk).code[self.ip]);
                    if !cond {
                        self.ip += (idx-1) as usize;
                        // println!("{:?} {:?} {}", cond, (*self.chunk).code[self.ip], self.ip);
                    }
                }
                OpCode::Jump(idx)=>{
                    self.ip += (idx-1) as usize;
                }
                OpCode::Args(_)=>{
                    // just skip this
                }
                ref x=>panic!("{:?}", x)
            }
            self.ip+=1;
            // println!("");

            if (*self.chunk).code.len()==self.ip {
                return
            }
            // let mut line = "".to_string();
            // std::io::stdin().read_line(&mut line).unwrap();
        }
    }

    fn function_transform(&self, fun: Function){
        self.expression_transform((*fun.expression).0)
    }

    fn expression_transform(&self, expression: ExprValue){
        match expression {
            x => panic!("{:?}", x),
        }
        return self.expression_transform(expression)
    }
}
