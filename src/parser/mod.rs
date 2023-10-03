use crate::lexer::tokens::{Token, TokenType};
use crate::SymbolTable;

use std::fs::{read_to_string};
use std::io::{BufRead};
use std::iter::Peekable;

use std::vec::IntoIter;

use owo_colors::OwoColorize;

pub mod class;
pub mod expression;
pub mod function;
pub mod program;

type TokenIter = Peekable<IntoIter<Token>>;

// #[derive(Debug)]
// pub struct Node{
//     pub ast_node: AstNode,
//     pub pos: i32,
//     pub line_no: i32,
//     pub file: String
// }

#[derive(Debug, Clone)]
pub struct NodePosition {
    pub pos: i32,
    pub line_no: i32,
    pub file: String,
}

//the top-level
#[derive(Debug)]
pub enum AstNode {
    Extern(External),
    FunctionDef(Function),
    Class(Class),
    Expression(ExprValue),
}

#[derive(Debug, Clone)]
pub enum ExprValue {
    FnCall(String, Vec<ExprValue>),
    UnOp(Box<TokenType>, Box<ExprValue>),
    BinOp(Box<ExprValue>, Box<TokenType>, Box<ExprValue>),
    Boolean(bool),
    Integer(i32),
    Str(String),
    Identifier(String),
    VarDecl {
        name: String,
        type_: String,
    },
    IfElse {
        cond: Box<ExprValue>,
        if_: Vec<ExprValue>,
        else_: Vec<ExprValue>,
    },
    Assign {
        name: String,
        value: Box<ExprValue>,
    },
    AugAssign {
        name: String,
        op: Box<TokenType>,
        value: Box<ExprValue>,
    },
    Return(Box<ExprValue>),
    Use(String),
    // Walrus(Box<ExprValue>, String, Box<ExprValue>),
    While(Box<ExprValue>, Vec<ExprValue>),
}

// 'extern' name (args) '->' return_type
#[derive(Debug)]
pub struct External {
    pub name: String,
    pub args: Args,
    pub return_type: String,
}

// 'def' name (args) '->' return_type { expressions}
#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub args: Args,
    pub expressions: Vec<ExprValue>,
    pub return_type: String,
}

// 'class' name {functions}
#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub fns: Vec<(Function, NodePosition)>,
}

#[derive(Debug)]
pub struct Module {
    pub name: String,
    pub classes: Vec<(Class, NodePosition)>,
    pub fns: Vec<(Function, NodePosition)>,
}

/// A parser that generates an abstract syntax tree.
pub struct Parser {
    tokens: TokenIter,
    pub symtab: SymbolTable,
    current_scope: String,
    pos: i32,
    line_no: i32,
    file: String,
}

#[derive(Debug, Clone)]
pub struct Args {
    pub name: Vec<String>,
    pub type_: Vec<String>,
} // I will  improvise this later.

impl Parser {
    pub fn new(tokens: TokenIter, file_path: &str) -> Self {
        Parser {
            tokens,
            symtab: SymbolTable::new(),
            current_scope: "global".to_string(),
            pos: -1,
            line_no: 1,
            file: file_path.to_string(),
        }
    }

    pub fn get_tok_precedence(&mut self, tok: TokenType) -> i32 {
        match tok {
            TokenType::Equal
            | TokenType::NotEq
            | TokenType::Greater
            | TokenType::GreaterEq
            | TokenType::Less
            | TokenType::LessEq => 0,
            TokenType::Minus | TokenType::Plus => 1,
            TokenType::DivEq | TokenType::Mul => 2,
            any => panic!("Bad operator! Unknown {:?}", any),
        }
    }

    fn advance(&mut self) {
        self.pos = match self.tokens.peek() {
            Some(t) => t,
            None => panic!("Dunno"),
        }
        .pos;
        self.line_no = match self.tokens.peek() {
            Some(t) => t,
            None => panic!("Dunno"),
        }
        .line_no;
        // self.file = match self.tokens.peek(){
        //     Some(t)=>t,
        //     None=> panic!("Dunno")
        // }.file.to_string();
    }

    fn parser_error(&self, cause: &str) -> String {
        format!(
            "
{text}
{pointy}
{cause}

    at {line}:{pos} in file `{file}`.",
            text=read_to_string(self.file.clone())
                .unwrap()
                .lines()
                .collect::<Vec<_>>()[(self.line_no - 1) as usize],
            pointy=("~".repeat(self.pos as usize) + "^").red(),
            cause=cause.yellow(),
            line = self.line_no.green(),
            pos=self.pos.green(),
            file=self.file.green()
        )
        .to_string()
    }
}
