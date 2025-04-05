use crate::codegen::{VMError, Visitor, Value};
use crate::parser::{AstNode, NodePosition, Function, ExprValue};
use crate::lexer::tokens::TokenType;

#[cfg(not(feature = "gxhash"))]
use std::collections::HashMap;

#[cfg(feature = "gxhash")]
use gxhash::{HashMap, HashMapExt};

use std::convert::TryFrom;

pub type Result<T> = std::result::Result<T, VMError>;
type NativeFn = fn(Vec<Value>) -> Result<Value>;

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
            constants: Vec::with_capacity(1024),
            variables: HashMap::new(),
            chunk: Box::new(Chunk{
                positions: vec![],
                code: vec![
                ]
            }),
            ip: 0,
            stack: Vec::with_capacity(128),
            callstack: Vec::with_capacity(64),
            functions: HashMap::new(), // maybe store functions within variables by compiling the bytecode of a function?
            // arena: Bump::new()
        };

        vm.init();

        for (node, pos) in astnodes {
            self.position = pos.clone();
            match node {
                AstNode::Expression(e) => {
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
        // vm.disassemble_chunk(&vm.chunk, "main");

        // // // Try to disassemble the 'count' function if found
        // if let Some((start_ip, _)) = vm.functions.get("count") {
        //     // Need to determine the end of the function's bytecode
        //     // This requires knowing the length stored in the FunctionDef opcode
        //     let current_ip = *start_ip;
        //     if let OpCode::FunctionDef(length) = vm.chunk.code[current_ip] {
        //         let end_ip = current_ip + 1 + (length as usize);
        //         // println!("chunk posns {:?}", vm.chunk.positions);
        //         // println!("chunk code {:?}", vm.chunk.code);
        //         let function_chunk = Chunk {
        //             positions: vm.chunk.positions.to_vec(),
        //             code: vm.chunk.code[*start_ip..end_ip].to_vec(),
        //         };
        //         vm.disassemble_chunk(&function_chunk, "count");
        //     }
        // }
    }
}

impl<'a> VM<'a> {
    
    fn pop(&mut self) -> Value {

        match self.stack.pop() {
            Some(x) => x,
            // Some(Value::LoxNativeError(s)) =>
            None => {
                panic!("VM panic! Attempted to pop a value when the value stack was empty")
            },
        }
    }

    // Note to future self: peek_mut SHOULD NEVER BE IMPLEMENTED!
    // Values on the stack are always implicit copy/cloned, any persistent values must be allocated with the future GC.

    fn peek(&self) -> &Value {
        self.peek_at(0)
    }

    fn peek_at(&self, dist: usize) -> &Value {
        self.stack.get(self.stack.len() - dist - 1).unwrap()
    }

    fn quick_insert_native_fn(&mut self, name: &str, function: NativeFn){
        self.variables.insert(
            name.to_string(),
            Value::NativeFunction(name.to_string(), function)
        );
    }

    pub fn init(&mut self) {
        self.quick_insert_native_fn("print", crate::codegen::stdlib::print);
        self.quick_insert_native_fn("input", crate::codegen::stdlib::input);
        self.quick_insert_native_fn("getattr", crate::codegen::stdlib::__getattr);
        self.quick_insert_native_fn("setattr", crate::codegen::stdlib::__setattr);
        self.quick_insert_native_fn("dict", crate::codegen::stdlib::__dict);
        self.quick_insert_native_fn("__dict_keys", crate::codegen::stdlib::__dict_keys);
        self.quick_insert_native_fn("startswith", crate::codegen::stdlib::__startswith);
        self.quick_insert_native_fn("len", crate::codegen::stdlib::__len);        
        self.quick_insert_native_fn("array", crate::codegen::stdlib::__array);        
        self.quick_insert_native_fn("len", crate::codegen::stdlib::__len);        
        self.quick_insert_native_fn("len", crate::codegen::stdlib::__len);        
        self.quick_insert_native_fn("json_parse", crate::codegen::json::json_parse);        
        self.quick_insert_native_fn("json_dumps", crate::codegen::json::json_dumps);        
        self.quick_insert_native_fn("start_tcp_server", crate::codegen::stdlib::start_tcp_server);        
        self.quick_insert_native_fn("read_file", crate::codegen::stdlib::read_file);        
        self.quick_insert_native_fn("write_file", crate::codegen::stdlib::write_file);        
    }

    fn disassemble_chunk(&self, chunk: &Chunk, name: &str) {
            println!("== {} ==", name);

            let mut offset = 0;
            while offset < chunk.code.len() {
                offset = self.disassemble_instruction(chunk, offset);
            }
        }

        fn disassemble_instruction(&self, chunk: &Chunk, offset: usize) -> usize {
            print!("{:04} ", offset);
            
            // if offset > 0 && chunk.positions[offset].0 == chunk.positions[offset - 1].0 {
            //     print!("   | ");
            // } else {
            //     print!("{:4} ", chunk.positions[offset].0);
            // }

            let instruction = &chunk.code[offset];
            match instruction {
                OpCode::Push => println!("OP_PUSH"),
                OpCode::Pop => println!("OP_POP"),
                OpCode::Return => println!("OP_RETURN"),
                OpCode::Call(arg) => println!("OP_CALL {}", self.constants[*arg as usize]),
                OpCode::Args(arg) => println!("OP_ARGS {}", arg),
                OpCode::Jump(arg) => println!("OP_JUMP {}", arg),
                OpCode::JumpIfFalse(arg) => println!("OP_JUMP_IF_FALSE {}", arg),
                OpCode::Constant(arg) => println!("OP_CONSTANT {}", self.constants[*arg as usize]),
                OpCode::Add => println!("OP_ADD"),
                OpCode::Sub => println!("OP_SUB"),
                OpCode::Mul => println!("OP_MUL"),
                OpCode::Div => println!("OP_DIV"),
                OpCode::Less => println!("OP_LESS"),
                OpCode::Greater => println!("OP_GREATER"),
                OpCode::LessEq => println!("OP_LESS_EQ"),
                OpCode::GreaterEq => println!("OP_GREATER_EQ"),
                OpCode::Equal => println!("OP_EQUAL"),
                OpCode::Concat => println!("OP_CONCAT"),
                OpCode::Load(arg) => println!("OP_LOAD {}", self.constants[*arg as usize]),
                OpCode::Assign(arg) => println!("OP_ASSIGN {}", self.constants[*arg as usize]),
                OpCode::VarDecl => println!("OP_VAR_DECL"),
                OpCode::FunctionDef(arg) => println!("OP_FUNCTION_DEF ({} instructions)", arg),
                OpCode::DefineArray(arg) => println!("OP_DEFINE_ARRAY {}", arg),
                OpCode::Do => println!("OP_DO"),
                OpCode::End => println!("OP_END"),
                // _ => println!("Unknown opcode {:?}", instruction),
            }
            offset + 1
        }

    pub fn _gen(&mut self, astnodes: Vec<(AstNode, NodePosition)>) {
        let _chunk: Vec<OpCode> = vec![];
        let _stack: Vec<Value> = vec![];

        for (node, _pos) in astnodes {
            match node {
                AstNode::Expression(_e) => {

                    // let _ = uoe(self.visit_expr(&mut e), &self.position);
                }
                AstNode::FunctionDef(_f) => {
                }
                // AstNode::Class(c) => self.visit_class(c),
                _ => todo!(),
            }
        }
    }

    pub fn _gen_function(&mut self, func: &Function){
            // Create a new Chunk to store the function's bytecode
            let mut function_chunk = Chunk {
                positions: vec![],
                code: vec![],
            };

            let len_before = function_chunk.code.len();
            self._gen_expr_into_chunk(&func.expression.0, &mut function_chunk);
            function_chunk.code.push(OpCode::Return);
            let len_after = function_chunk.code.len();

            let ip = self.chunk.code.len();
            self.chunk.code.push(OpCode::FunctionDef((len_after - len_before) as u8));
            self.chunk.code.append(&mut function_chunk.code);
            self.chunk.positions.append(&mut function_chunk.positions); // Don't forget to append positions
            self.functions.insert(func.name.to_string(), (ip, func.args.name.clone()));
        }

        fn _gen_expr_into_chunk(&mut self, expr: &ExprValue, chunk: &mut Chunk) {
            match expr {
                ExprValue::Integer(i) => {
                    self.constants.push(Value::Float64(*i as f64));
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                },
                ExprValue::Double(d) => {
                    self.constants.push(Value::Float64(*d as f64));
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                },
                ExprValue::Str(s) => {
                    self.constants.push(Value::Str(s.to_string()));
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                },
                ExprValue::Boolean(b) => {
                    self.constants.push(Value::Boolean(*b));
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                },
                ExprValue::None => {
                    self.constants.push(Value::None);
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                },
                ExprValue::BinOp(lhs, op, rhs) => match **op {
                    TokenType::Plus => match ((**lhs).clone(), (**rhs).clone()) {
                        (ExprValue::Str(_), _) | (_, ExprValue::Str(_)) => {
                            self._gen_expr_into_chunk(rhs, chunk);
                            self._gen_expr_into_chunk(lhs, chunk);
                            chunk.code.push(OpCode::Concat);
                        }
                        _ => {
                            self._gen_expr_into_chunk(rhs, chunk);
                            self._gen_expr_into_chunk(lhs, chunk);
                            chunk.code.push(OpCode::Add);
                        }
                    },
                    TokenType::Minus => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Sub);
                    }
                    TokenType::Div => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Div);
                    }
                    TokenType::Mul => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Mul);
                    }
                    TokenType::Less => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Less);
                    }
                    TokenType::Greater => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Greater);
                    }
                    TokenType::LessEq => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::LessEq);
                    }
                    TokenType::GreaterEq => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::GreaterEq);
                    }
                    TokenType::Equal => {
                        self._gen_expr_into_chunk(rhs, chunk);
                        self._gen_expr_into_chunk(lhs, chunk);
                        chunk.code.push(OpCode::Equal);
                    }
                    _ => todo!(),
                },
                ExprValue::Assign { name, value } => {
                    self._gen_expr_into_chunk(value, chunk);
                    self.constants.push(Value::Str(name.to_string()));
                    chunk.code.push(OpCode::Assign((self.constants.len()-1) as u8));
                }
                ExprValue::VarDecl { name, type_: _ } => {
                    self.constants.push(Value::Str(name.to_string()));
                    chunk.code.push(OpCode::Constant((self.constants.len()-1) as u8));
                    chunk.code.push(OpCode::VarDecl);
                }
                ExprValue::Do (expressions) => {
                    for ex in expressions {
                        self._gen_expr_into_chunk(ex, chunk);
                    }
                }
                ExprValue::Array (expressions) => {
                    let len_before = chunk.code.len();
                    for ex in expressions {
                        self._gen_expr_into_chunk(ex, chunk);
                    }
                    let len_after = chunk.code.len();
                    chunk.code.push(OpCode::DefineArray((len_after - len_before)as u8));
                }
                ExprValue::Identifier(i) => {
                    self.constants.push(Value::Str(i.to_string()));
                    chunk.code.push(OpCode::Load((self.constants.len()-1) as u8));
                }
                ExprValue::FnCall(name, args) => {
                    for ex in args {
                        self._gen_expr_into_chunk(ex, chunk);
                    }
                    self.constants.push(Value::Str(name.to_string()));
                    chunk.code.push(OpCode::Call((self.constants.len()-1) as u8));
                    chunk.code.push(OpCode::Args(args.len() as u8));
                }
                ExprValue::IfElse{cond, if_, else_} => {
                    self._gen_expr_into_chunk(&cond, chunk);

                    let mut if_chunk = Chunk { positions: vec![], code: vec![] };
                    self._gen_expr_into_chunk(&if_, &mut if_chunk);
                    let if_len = if_chunk.code.len();

                    let mut else_chunk = Chunk { positions: vec![], code: vec![] };
                    self._gen_expr_into_chunk(&else_, &mut else_chunk);
                    let else_len = else_chunk.code.len();

                    chunk.code.push(OpCode::JumpIfFalse((if_len + 1) as u8)); // +1 for the jump over else
                    chunk.code.append(&mut if_chunk.code);
                    chunk.code.push(OpCode::Jump((else_len) as u8));
                    chunk.code.append(&mut else_chunk.code);
                }
                x=>unimplemented!("{:#?}", x)
            }
        }


    pub fn _gen_expr(&mut self, expr: &ExprValue) {
    
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
                self.chunk.code.push(OpCode::DefineArray((len_after - len_before)as u8));
            }
            ExprValue::Identifier(i) => {
                self.constants.push(Value::Str(i.to_string()));
                self.chunk.code.push(OpCode::Load((self.constants.len()-1) as u8));
            }
            ExprValue::FnCall(name, args) => {
                // evaluate all args
                
                let _len_before = self.chunk.code.len();
                for ex in args {
                    self._gen_expr(ex);
                }
                let _len_after = self.chunk.code.len();

                self.constants.push(Value::Str(name.to_string())); // feels wrong
                self.chunk.code.push(OpCode::Call((self.constants.len()-1) as u8));
                self.chunk.code.push(OpCode::Args(args.len() as u8));            
            }
            ExprValue::IfElse{cond, if_, else_} => {
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

                self.chunk = chunk_original;
                self.chunk.code.push(OpCode::JumpIfFalse(if_len as u8));
                self.chunk.code.append(&mut if_chunk.code);
                self.chunk.code.push(OpCode::Jump((if_len+1) as u8));
                self.chunk.code.append(&mut else_chunk.code);
                self.chunk.code.push(OpCode::Jump((else_len+1) as u8));
            }
            x=>unimplemented!("{:#?}", x)
        }
    }

    pub fn _run(&mut self){
        #[cfg(feature="debug-build")]
        let mut values = [
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
            0,
        ];

        #[cfg(feature="debug-build")]
        let mut value_names = [
            "const", "sub", "add",
            "mul", "div", "less",
            "greater", "lesseq", "greatereq",
            "equal", "def_array", "load",
            "assign", "call", "fndef",
            "ret", "jif", "jump",
            "args",
        ];
        loop {
            match (*self.chunk).code[self.ip] {
                OpCode::Constant(index)=>{
                    #[cfg(feature="debug-build")]
                    {
                        values[0] += 1;
                    }
                    self.stack.push(self.constants[index as usize].clone());
                }
                OpCode::Sub => {
                    #[cfg(feature="debug-build")]
                    {
                        values[1] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a - b
                    self.stack.push(Value::Float64(
                        f64::try_from(a).unwrap()
                            - f64::try_from(b)
                            .unwrap()
                    ));
                }
                OpCode::Add => {
                    #[cfg(feature="debug-build")]
                    {
                        values[2] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a + b
                    match (a, b) {
                        // (Value::Integer(i1), Value::Integer(i2)) => self.stack.push(Value::Integer(i1 + i2)),
                        (Value::Float64(f1), Value::Float64(f2)) => self.stack.push(Value::Float64(f1 + f2)),
                        // (Value::Float64(f), Value::Integer(i)) => self.stack.push(Value::Float64(f + i)),
                        (Value::Str(s), o) => self.stack.push(Value::Str(s + &o.to_string())),
                        // _ => 
                        
                        // ... handle other potential type combinations
                        _ => panic!("Invalid operands for addition"),
                    }
                    // self.stack.push(Value::Float64(
                    //     f64::try_from(a).unwrap()
                    //         + f64::try_from(b)
                    //         .unwrap()
                    // ));
                }
                OpCode::Mul => {
                    #[cfg(feature="debug-build")]
                    {
                        values[3] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a * b
                    match (a, b) {
                        // (Value::Integer(i1), Value::Integer(i2)) => self.stack.push(Value::Integer(i1 + i2)),
                        (Value::Float64(f1), Value::Float64(f2)) => self.stack.push(Value::Float64(f1 * f2)),
                        // (Value::Float64(f), Value::Integer(i)) => self.stack.push(Value::Float64(f + i)),
                        // (Value::Str(s), o) => self.stack.push(Value::Str(s + &o.to_string())),
                        // _ => 
                        
                        // ... handle other potential type combinations
                        _ => panic!("Invalid operands for addition"),
                    }
                }
                OpCode::Div => {
                    #[cfg(feature="debug-build")]
                    {
                        values[4] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a / b
                    match (a, b) {
                        // (Value::Integer(i1), Value::Integer(i2)) => self.stack.push(Value::Integer(i1 + i2)),
                        (Value::Float64(f1), Value::Float64(f2)) => self.stack.push(Value::Float64(f1 / f2)),
                        
                        // ... handle other potential type combinations
                        _ => panic!("Invalid operands for addition"),
                    }
                }
                OpCode::Less => {
                    #[cfg(feature="debug-build")]
                    {
                        values[5] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a < b
                    self.stack.push(Value::Boolean(a < b));
                }
                OpCode::Greater => {
                    #[cfg(feature="debug-build")]
                    {
                        values[6] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a > b
                    self.stack.push(Value::Boolean(a > b));
                }
                OpCode::LessEq => {
                    #[cfg(feature="debug-build")]
                    {
                        values[7] += 1;
                    }


                    let a = self.pop();
                    let b = self.pop();
                    // a <= b
                    match (a, b) {
                        (Value::Float64(f1), Value::Float64(f2)) => self.stack.push(Value::Boolean(f1 <= f2)),
                        // ... handle other potential type combinations
                        (x, y) => self.stack.push(Value::Boolean(x <= y)),
                    }
                    // self.stack.push(Value::Boolean(a <= b));
                
                }
                OpCode::GreaterEq => {
                    #[cfg(feature="debug-build")]
                    {
                        values[8] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a >= b
                    println!("> {}", a >= b);
                    self.stack.push(Value::Boolean(a >= b));
                }
                OpCode::Equal => {
                    #[cfg(feature="debug-build")]
                    {
                        values[9] += 1;
                    }

                    let a = self.pop();
                    let b = self.pop();
                    // a == b
                    println!("> {}", a == b);
                    self.stack.push(Value::Boolean(a == b));
                }
                OpCode::DefineArray(len) => {
                    #[cfg(feature="debug-build")]
                    {
                        values[10] += 1;
                    }


                    let mut exprs = vec![];
                    let _values_len = self.constants.len()-1; 
                    
                    for _i in 0..len as usize { 
                        exprs.push(self.pop());
                    }
                    self.stack.push(Value::Array(exprs));
                    // println!("> {:?}", self.pop()); // this should ALWAYS point to the newly created array
                }
                OpCode::Load(index) => {
                     #[cfg(feature="debug-build")]
                    {
                        values[11] += 1;
                    }

                    let name = self.constants[index as usize].to_string(); // hacky or bad?
                    let val = match self.variables.get(&name) {
                        Some(expr) => expr,
                        None => panic!("no such variable! {}", name.to_string()), // todo: runtime error
                    };
                    self.stack.push(val.clone());
                }
                OpCode::Assign(index) => {
                    #[cfg(feature="debug-build")]
                    {
                        values[12] += 1;
                    }
                    // im tired rn
                    let name = self.constants[index as usize].to_string(); // hacky or bad?
                    let value = self.pop();
                    self.variables.insert(name, value.clone());
                    self.stack.push(value);
                }
                OpCode::Call(idx) => {
                    #[cfg(feature="debug-build")]
                    {
                        values[13] += 1;
                    }

                    let fname = self.constants[idx as usize].to_string();
                    let _original_ip = self.ip;
                    match self.functions.get(&fname){
                        Some(s)=>{
                            let ip = s.0;
                            // loads all the args
                            let mut argnames = s.1.clone();
                            argnames.reverse();
                            for argname in argnames {
                                {
                                    let value = self.pop();
                                    self.variables.insert(argname.to_string(), value);
                                }
                            }
                            self.callstack.push(self.ip);
                            self.ip = ip; // because we want to skip the args instruction
                            // how do i return to where the stack was before calling?
                        }
                        None=>{
                            let fun = match self.variables.get(&fname) {
                                Some(Value::NativeFunction(_name, f)) => {
                                    f.clone()
                                },
                                _ => panic!("NO SUCH FUNCTION {}", fname),
                            };
                            self.ip+=1;
                            // see the no of args
                            if let OpCode::Args(n) = (*self.chunk).code[self.ip] {
                                let mut args = vec![];
                                for _x in 0..n {
                                    args.push(self.pop());
                                }
                                let _ = fun(args);
                            }
                        },
                    }
                }
                OpCode::FunctionDef(len) => {
                    #[cfg(feature="debug-build")]
                    {
                        values[14] += 1;
                    }

                    self.ip += len as usize;
                    // len is the addr of the last instruction in the function
                    // just after this match statement, ip is incremented by 1

                    // TL; DR: this just skips the function body
                }
                OpCode::Return => {
                    #[cfg(feature="debug-build")]
                    {
                        values[15] += 1;
                    }

                    self.ip = self.callstack.pop().expect("unreachable");
                }
                OpCode::JumpIfFalse(idx) => {
                    #[cfg(feature="debug-build")]
                    {
                        values[16] += 1;
                    }

                    let cond = bool::from(self.pop());
                    if !cond {
                        self.ip += (idx-1) as usize;
                    }
                }
                OpCode::Jump(idx)=>{
                    #[cfg(feature="debug-build")]
                    {
                        values[17] += 1;
                    }

                    self.ip += (idx-1) as usize;
                }
                OpCode::Args(_)=>{
                    #[cfg(feature="debug-build")]
                    {
                        values[18] += 1;
                    }
                    // just skip this
                }
                ref x=>panic!("{:?}", x)
            }
            self.ip+=1;

            if (*self.chunk).code.len()==self.ip {
                #[cfg(feature="debug-build")]
                for i in 0..values.len(){
                    println!("{} -> {}", value_names[i], values[i]);
                }
                return
            }
        }
    }
}
