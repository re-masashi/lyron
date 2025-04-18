use crate::codegen::{uoe, VMError, Visitor};
use crate::parser::{AstNode, NodePosition};

pub type Result<T> = std::result::Result<T, VMError>;

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
}
