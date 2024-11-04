use crate::codegen::{uoe, VMError, Visitor, Value};

use crate::parser::{AstNode, NodePosition, Function, ExprValue};
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use gxhash::HashMap;

pub type Result<T> = std::result::Result<T, VMError>;

#[derive(Debug)]
enum OpCode {
    Return,
    Push,
    Pop,
    Call(u8), // no of args
    Jump,
    Constant(u8),
}

struct VM {
    constants: Arc<RefCell<Vec<Value>>>,
    variables: Mutex<RefCell<HashMap<String, Value>>>,
}

#[derive(Debug)]
struct Chunk {
    code: Vec<OpCode>
}

impl Visitor {
    pub fn visit_program(&mut self, astnodes: Vec<(AstNode, NodePosition)>) {
        for (node, pos) in astnodes {
            self.position = pos.clone();
            match node {
                AstNode::Expression(mut e) => {
                    let _ = uoe(self.visit_expr(&mut e), &self.position);
                }
                AstNode::FunctionDef(f) => {
                    let _ = self.visit_fn(f);
                }
                AstNode::Class(c) => self.visit_class(c),
                _ => todo!(),
            }
        }
    }

    pub fn _run(&mut self, astnodes: Vec<(AstNode, NodePosition)>) {
        let mut chunk: Vec<OpCode> = vec![];
        let mut stack: Vec<Value> = vec![];

        for (node, pos) in astnodes {
            self.position = pos.clone();
            match node {
                AstNode::Expression(mut e) => {
                    // let _ = uoe(self.visit_expr(&mut e), &self.position);
                }
                AstNode::FunctionDef(f) => {
                }
                // AstNode::Class(c) => self.visit_class(c),
                _ => todo!(),
            }
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