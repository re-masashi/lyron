use crate::lexer::tokens::{Token, TokenType};
use crate::parser::{Class, Function, NodePosition, Parser, Module};
use crate::{unwrap_some, Result};
use std::collections::HashMap;

impl Parser {
    pub fn parse_class(&mut self) -> Result<(Module, NodePosition)> {
        let name: String;
        let mut fns: Vec<Function> = Vec::new();

        println!("{:#?}", self.tokens.peek());

        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `mod`
        let start = NodePosition {
            pos: nx.pos,
            line_no: nx.line_no,
            file: nx.file.to_string(),
        };
        println!("{:#?}", self.tokens.peek());

        match &unwrap_some!(self.tokens.peek()).type_ {
            TokenType::Identifier(i) => name = i.clone(),
            _ => return Err("Syntax Error: expected Identifier after keyword 'mod'".to_string()),
        }
        self.advance();
        self.tokens.next(); // eat the identifier

        self.advance();
        match unwrap_some!(self.tokens.next()).type_ {
            TokenType::LBrace => {}
            _ => return Err("Expected '{' in module".to_string()),
        }

        while unwrap_some!(self.tokens.peek()).type_ != TokenType::RBrace {
            println!("{:#?}", self.tokens.peek());
            match unwrap_some!(self.tokens.peek()).type_ {
                TokenType::Function=>{
                    match self.parse_function() {
                        Ok((f, _)) => fns.insert(fns.len(), f),
                        Err(e) => {
                            println!("oops");
                            return Err(e);
                        }
                    }
                }
            }
            
        }
        self.advance();
        self.tokens.next(); // eat '}'
        return Ok((Class { name, fns }, start));
    }
}
