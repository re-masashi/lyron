use crate::lexer::tokens::{Token, TokenType};
use crate::parser::{Args, ExprValue, Function, NodePosition, Parser};
use crate::{unwrap_some, Result, Symbol};

impl Parser {
    fn parse_type_annot(&mut self) -> Result<(String, String)> {
        // Check if Identifier exists, else return Err
        match unwrap_some!(self.tokens.peek()) {
            Token {
                type_: TokenType::Identifier(_),
                pos: _,
                line_no: _,
                file: _,
            } => {}
            _ => {
                println!("{:?}", self.tokens.peek());
                return Err(self.parser_error("Expected Identifier or ')'"))
            }
        }
        // Store identifier.
        self.advance();
        let name = match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Identifier(s) => s,
            _ => unreachable!(),
        };
        // Check if colon exists.
        match unwrap_some!(self.tokens.peek()) {
            Token {
                type_: TokenType::Colon,
                pos: _,
                line_no: _,
                file: _,
            } => {}
            _ => return Err("expected ':' .".to_string()),
        }
        self.advance();
        self.tokens.next(); // Eat ':'
                            // Check if type exists
        match unwrap_some!(self.tokens.peek()) {
            Token {
                type_: TokenType::Identifier(_),
                pos: _,
                line_no: _,
                file: _,
            } => {}
            _ => return Err("expected Identifier.".to_string()),
        }
        self.advance();
        // Store type
        let type_ = match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Identifier(s) => s,
            _ => unreachable!(),
        };
        Ok((name, type_))
    }

    pub fn parse_function(&mut self) -> Result<(Function, NodePosition)> {
        let name: String;
        let return_type: String;
        let mut args = Args {
            name: vec![],
            type_: vec![],
        };
        let expressions: Vec<(ExprValue, NodePosition)> = Vec::new();
        match self.tokens.peek() {
            Some(Token {
                type_: TokenType::Def,
                pos,
                line_no,
                file,
            }) => {
                let start = NodePosition {
                    pos: *pos,
                    line_no: *line_no,
                    file: file.to_string(),
                };
                self.advance();
                self.tokens.next(); // Eat Def

                match unwrap_some!(self.tokens.peek()) {
                    Token {
                        type_: TokenType::Identifier(_),
                        pos: _,
                        line_no: _,
                        file: _,
                    } => {}
                    _ => return Err(self.parser_error("Expected Identifier after keyword 'def'")),
                }
                self.advance();
                // Eat and store
                match unwrap_some!(self.tokens.next()).type_ {
                    TokenType::Identifier(n) => name = n, // Always matches
                    _ => unreachable!(),                  // never happens
                }
                self.current_scope = format!("{}.{}", self.current_scope, name.clone());

                if unwrap_some!(self.tokens.peek()).type_ != TokenType::LParen {
                    return Err(self.parser_error("Expected '(' after Identifier"));
                }

                self.tokens.next(); // Eat '('

                if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
                    self.tokens.next(); // Eat ')'
                } else {
                    loop {
                        if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
                            self.tokens.next(); // Eat ','
                            continue;
                        }
                        if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
                            self.tokens.next(); // Eat ')'
                            break;
                        }
                        let type_annot = self.parse_type_annot();
                        match type_annot {
                            Ok((n, t)) => {
                                args.name.push(n);
                                args.type_.push(t);
                            }
                            Err(e) => {
                                return Err(e);
                            }
                        };
                    }
                }

                if unwrap_some!(self.tokens.peek()).type_ != TokenType::Arrow {
                    return Err(self.parser_error("expected '->'"));
                }
                self.advance();
                self.tokens.next(); // Eat '->'

                match &unwrap_some!(self.tokens.peek()).type_ {
                    TokenType::Identifier(n) => return_type = n.to_string(),
                    _ => return Err(self.parser_error("expected return type")),
                }
                self.advance();
                self.tokens.next(); // Eat the return_type

                let expression = self.parse_expression().unwrap();

                match self.tokens.peek() {
                    Some(t) if t.type_ == TokenType::Semicolon => {
                        self.advance();
                        self.tokens.next(); // Eat semicolon, if present
                    }
                    _ => {}
                }
                self.current_scope = "global".to_string();
                self.symtab.insert(
                    name.clone(),
                    Symbol::new(return_type.clone(), self.current_scope.clone()),
                );
                Ok((
                    Function {
                        name,
                        args,
                        expression: Box::new(expression),
                        return_type,
                    },
                    start,
                ))
            }
            _ => Err("PASS".to_string()), // never happens
        }
    }
}
