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
                return Err(self.parser_error("Expected Identifier"))
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

    // pub fn parse_extern(&mut self) -> Result<(External, NodePosition)> {
    //     let mut args = Args {
    //         name: vec![],
    //         type_: vec![],
    //     };

    //     self.advance();
    //     let nx = unwrap_some!(self.tokens.next()); // Eat extern
    //     let start = NodePosition {
    //         pos: nx.pos,
    //         line_no: nx.line_no,
    //         file: nx.file.to_string(),
    //     };

    //     match unwrap_some!(self.tokens.peek()) {
    //         Token {
    //             type_: TokenType::Identifier(_),
    //             pos: _,
    //             line_no: _,
    //             file: _,
    //         } => {}
    //         _ => return Err(self.parser_error("Expected Identifier after keyword 'extern'")),
    //     }
    //     self.advance();
    //     // Eat and store name
    //     let name = match unwrap_some!(self.tokens.next()).type_ {
    //         TokenType::Identifier(n) => n, // Always matches
    //         _ => unreachable!(),           // never happens
    //     };

    //     if unwrap_some!(self.tokens.peek()).type_ != TokenType::LParen {
    //         return Err(self.parser_error("Expected '(' after identifier"));
    //     }
    //     self.advance();
    //     self.tokens.next(); // Eat '('

    //     if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
    //         self.tokens.next(); // Eat ')'
    //     } else {
    //         loop {
    //             if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
    //                 self.advance();
    //                 self.tokens.next(); // Eat ','
    //                 continue;
    //             }
    //             if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
    //                 self.advance();
    //                 self.tokens.next(); // Eat ')'
    //                 break;
    //             }
    //             let type_annot = self.parse_type_annot();
    //             match type_annot {
    //                 Ok((n, t)) => {
    //                     args.name.insert(args.name.len(), n);
    //                     args.type_.insert(args.type_.len(), t);
    //                 }
    //                 Err(e) => {
    //                     return Err(e);
    //                 }
    //             };
    //         }
    //     }

    //     if unwrap_some!(self.tokens.peek()).type_ != TokenType::Arrow {
    //         return Err(self.parser_error("expected '->'"));
    //     }
    //     self.advance();
    //     self.tokens.next(); // Eat '->'

    //     let return_type: String = match &unwrap_some!(self.tokens.peek()).type_ {
    //         TokenType::Identifier(n) => n.to_string(),
    //         _ => return Err(self.parser_error("expected return type after extern")),
    //     };
    //     self.advance();
    //     self.tokens.next(); // Eat the identifier

    //     if unwrap_some!(self.tokens.peek()).type_ == TokenType::Semicolon {
    //         self.advance();
    //         self.tokens.next(); //Eat semicolon
    //     } else {
    //         return Err(self.parser_error("Semicolon after extern is mandatory."));
    //     }
    //     self.symtab.insert(
    //         name.clone(),
    //         Symbol::new(return_type.clone(), self.current_scope.clone()),
    //     );
    //     Ok((
    //         External {
    //             name,
    //             args,
    //             return_type,
    //         },
    //         start,
    //     ))
    // } // end of parse_extern

    pub fn parse_function(&mut self) -> Result<(Function, NodePosition)> {
        let name: String;
        let return_type: String;
        let mut args = Args {
            name: vec![],
            type_: vec![],
        };
        let mut expressions: Vec<ExprValue> = Vec::new();
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

                if unwrap_some!(self.tokens.peek()).type_ != TokenType::Do {
                    return Err(self.parser_error("expected 'do' in fn def"));
                }
                self.advance();
                self.tokens.next(); // Eat 'do'

                loop {
                    match self.parse_expression() {
                        Ok(expr) => expressions.insert(expressions.len(), expr.0),
                        Err(e) if e == self.parser_error("Invalid expression") => {
                            if unwrap_some!(self.tokens.peek()).type_ == TokenType::End
                                || unwrap_some!(self.tokens.peek()).type_ == TokenType::Semicolon
                            {
                                break;
                            } else {
                                return Err(e);
                            }
                        }
                        Err(e) => return Err(e),
                    }
                    // Eat the semicolons
                    match unwrap_some!(self.tokens.peek()).type_ {
                        TokenType::Semicolon => {
                            self.advance();
                            self.tokens.next();
                            continue;
                        }
                        TokenType::End => break,
                        _ => {
                            print!("{:?}", self.tokens.peek());
                            return Err(self.parser_error("Expected semicolon or 'do'"));
                        }
                    }
                }

                if unwrap_some!(self.tokens.peek()).type_ != TokenType::End {
                    print!("{:?}", unwrap_some!(self.tokens.peek()).type_);
                    return Err(self.parser_error("expected 'end'"));
                }
                self.advance();
                self.tokens.next(); // Eat Do

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
                        expressions,
                        return_type,
                    },
                    start,
                ))
            }
            _ => Err("PASS".to_string()), // never happens
        }
    }
}
