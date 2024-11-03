use crate::lexer::tokens::TokenType;
use crate::parser::{ExprValue, NodePosition, Parser};
use crate::{unwrap_some, Result, Symbol};
use log::trace;

impl Parser {
    pub fn parse_expression(&mut self) -> Result<(ExprValue, NodePosition)> {
        trace!("Parsing expression");
        let l_value: Result<(ExprValue, NodePosition)> =
            match unwrap_some!(self.tokens.peek()).type_ {
                TokenType::LParen => {
                    self.tokens.next();
                    self.advance();
                    self.parse_paren_expression()
                }
                // Unary
                TokenType::Plus | TokenType::Minus | TokenType::Not => self.parse_unop(),

                TokenType::If => self.parse_if_else(),

                TokenType::While => self.parse_while(),

                TokenType::Let => self.parse_declaration(),

                TokenType::True => self.parse_true(),

                TokenType::False => self.parse_false(),

                TokenType::Identifier(_) => self.parse_identifier(), // Parses identifiers, assignments and function calls as well

                TokenType::Return => self.parse_return(),

                TokenType::Use => self.parse_use(),

                TokenType::Do => self.parse_do(),

                TokenType::Extern => self.parse_extern(),

                TokenType::None => self.parse_none(),

                TokenType::Integer(i) => {
                    let nx = unwrap_some!(self.tokens.next());
                    self.advance();
                    Ok((
                        ExprValue::Integer(i),
                        NodePosition {
                            pos: nx.pos,
                            line_no: nx.line_no,
                            file: nx.file,
                        },
                    ))
                }

                TokenType::Double(f) => {
                    let nx = unwrap_some!(self.tokens.next());
                    self.advance();
                    Ok((
                        ExprValue::Double(f),
                        NodePosition {
                            pos: nx.pos,
                            line_no: nx.line_no,
                            file: nx.file,
                        },
                    ))
                }

                TokenType::Str(_) => self.parse_string(),
                TokenType::Async|TokenType::Await=> panic!("yet to be implemented"),

                _ =>{
                    // println!("{:?}", x);
                    return Err(self.parser_error("Invalid expression"))
                }
            };

        // The functions above will eat the value, then we can proceed to check for a bin op.
        loop {
            let op: TokenType = match unwrap_some!(self.tokens.peek()).type_ {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Div
                | TokenType::Mul
                | TokenType::Less
                | TokenType::Dot
                | TokenType::LessEq
                | TokenType::Greater
                | TokenType::GreaterEq
                | TokenType::Equal
                | TokenType::NotEq => {
                    self.advance();
                    unwrap_some!(self.tokens.next()).type_
                }
                _ => return l_value,
            };
            let r_value = self.parse_expression();
            match r_value {
                Ok(_) => {},
                Err(ref e) => println!("{}",self.parser_error(&e)),
            }
            match unwrap_some!(self.tokens.peek()).type_ {
                TokenType::Plus
                | TokenType::Minus
                | TokenType::Div
                | TokenType::Mul
                | TokenType::Less
                | TokenType::LessEq
                | TokenType::Greater
                | TokenType::GreaterEq
                | TokenType::Equal
                | TokenType::NotEq => continue, // Leave it at this stage, let the loop start with binop search again.
                _ => {
                    // println!("{:#?} {:?} {:#?}",l_value,op, r_value );
                    return Ok((
                        // todo: match to avoid unwrap
                        ExprValue::BinOp(
                            Box::new(l_value.unwrap().0),
                            Box::new(op),
                            Box::new(r_value.unwrap().0),
                        ),
                        NodePosition {
                            pos: self.pos,
                            line_no: self.line_no,
                            file: self.file.clone(),
                        },
                    ));
                }
            };
        }
    }

    pub fn parse_unop(&mut self) -> Result<(ExprValue, NodePosition)> {
        trace!("Parsing unop");
        // Eat the operator while working.
        let nx = unwrap_some!(self.tokens.next());
        let start = NodePosition {
            pos: nx.pos,
            line_no: nx.line_no,
            file: nx.file,
        };
        self.advance();
        let t = nx.type_;
        let op = Box::new(t);
        let expr = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
        Ok((ExprValue::UnOp(op, expr), start))
    }

    pub fn parse_paren_expression(&mut self) -> Result<(ExprValue, NodePosition)> {
        trace!("Parsing paren expr");
        let expr = self.parse_expression();
        let expr = expr.unwrap().0; // todo: remove unwrap
        if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
            self.advance();
            let nx = unwrap_some!(self.tokens.next()); // Eat ')'
            Ok((
                expr,
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            ))
        } else {
            Err(self.parser_error("Missing closing ')'"))
        }
    }

    pub fn parse_do(&mut self) -> Result<(ExprValue, NodePosition)>{
        let mut exprs = vec![];
        
        // println!("some {:?}", self.tokens.peek());

        self.advance();
        self.tokens.next(); // eat 'do'

        let pos = NodePosition {
            pos: self.pos,
            line_no: self.line_no,
            file: self.file.clone(),
        };

        loop {
            match self.parse_expression() {
                Ok((expr, _)) => exprs.push(expr),
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
                    // return Err(self.parser_error("Expected ';' or 'end'"))
                },
            }
        }

        if unwrap_some!(self.tokens.peek()).type_ == TokenType::End {
            self.advance();
            self.tokens.next(); // Eat 'end'
        } // No other case

        return Ok((
            ExprValue::Do(exprs),
            pos
        ))
    }

    pub fn parse_if_else(&mut self) -> Result<(ExprValue, NodePosition)> {
        // trace!("Parsing if else");
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat 'if'
        let type_ = String::from("unavailable");
        let hastype = !true;

        let cond = Box::new(self.parse_expression().unwrap().0);

        if unwrap_some!(self.tokens.peek()).type_ == TokenType::Then {
            self.advance();
            self.tokens.next(); // eat 'then'
        }else {
            // println!("{:?}", self.tokens.peek());
            panic!("then expected after condition");
        }

        // println!("{:?} {:?}", self.pos, self.line_no);
        let (expression_if, pos) = self.parse_expression().unwrap();

        if unwrap_some!(self.tokens.peek()).type_ == TokenType::Else {
            self.advance();
            self.tokens.next(); // Eat 'else'

            let (expression_else, pos) = self.parse_expression().unwrap();

            Ok((
                ExprValue::IfElse {
                    cond,
                    if_: Box::new(expression_if),
                    else_: Box::new(expression_else),
                },
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            ))

        } else {
            return Ok((
                ExprValue::IfElse {
                    cond,
                    if_: Box::new(expression_if),
                    else_: Box::new(ExprValue::None),
                },
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            ));
        }
    }

    pub fn parse_while(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat 'while'
        let condition = self.parse_expression().unwrap().0; // todo: remove unwrap
        let expression = self.parse_expression().unwrap().0;

        Ok((
            ExprValue::While(Box::new(condition), Box::new(expression)),
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_declaration(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `let`
        let name: String = match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Identifier(n) => n,
            _ => return Err(self.parser_error("Expected an identifier after let")),
        };
        if unwrap_some!(self.tokens.peek()).type_ == TokenType::Colon {
            self.advance();
            self.tokens.next(); // Eat ':'
        } else {
            return Err(self.parser_error("Missing ':'."));
        }

        let type_ = match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Identifier(t) => t,
            _ => return Err(self.parser_error("Expected an identifier")),
        };
        self.symtab.insert(
            name.clone(),
            Symbol::new(type_.clone(), self.current_scope.clone()),
        );
        Ok((
            ExprValue::VarDecl { name, type_ },
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_true(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `true`
        Ok((
            ExprValue::Boolean(true),
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_false(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `false`
        Ok((
            ExprValue::Boolean(false),
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_none(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `true`
        Ok((
            ExprValue::None,
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_identifier(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        // Eat the identifier and work.
        let nx = unwrap_some!(self.tokens.next());
        let start = NodePosition {
            pos: nx.pos,
            line_no: nx.line_no,
            file: nx.file,
        };
        let name = match nx.type_ {
            TokenType::Identifier(n) => n,
            _ => unreachable!(),
        };
        // Check for assignment
        match unwrap_some!(self.tokens.peek()).type_ {
            TokenType::Assign => {
                self.advance();
                self.tokens.next(); // Eat '='
                let value = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
                return Ok((ExprValue::Assign { name, value }, start));
            }
            TokenType::PlusEq => {
                self.advance();
                let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '+='
                let value = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
                return Ok((ExprValue::AugAssign { name, op, value }, start));
            }
            TokenType::MinusEq => {
                self.advance();
                let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '-='
                let value = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
                return Ok((ExprValue::AugAssign { name, op, value }, start));
            }
            TokenType::DivEq => {
                self.advance();
                let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '/='
                let value = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
                return Ok((ExprValue::AugAssign { name, op, value }, start));
            }
            TokenType::MulEq => {
                self.advance();
                let op = Box::new(unwrap_some!(self.tokens.next()).type_); // Eat '*='
                let value = Box::new(self.parse_expression().unwrap().0); // todo: remove unwrap
                return Ok((ExprValue::AugAssign { name, op, value }, start));
            }
            _ => {}
        }
        // Check for function call
        if unwrap_some!(self.tokens.peek()).type_ == TokenType::LParen {
            self.advance();
            self.tokens.next(); // Eat '('
            let mut values = Vec::new();
            loop {
                match self.parse_expression() {
                    Ok((expr, _)) => values.insert(values.len(), expr),
                    Err(e) => {
                        if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
                            break;
                        } else if unwrap_some!(self.tokens.peek()).type_ == TokenType::RParen {
                            self.advance();
                            self.tokens.next(); // Eat ')'
                            return Ok((ExprValue::FnCall(name, values), start));
                        } else {
                            return Err(e);
                        }
                    }
                }
                if unwrap_some!(self.tokens.peek()).type_ == TokenType::Comma {
                    self.advance();
                    self.tokens.next(); // Eat ','
                }
            }
        }
        Ok((ExprValue::Identifier(name), start))
    }

    pub fn parse_return(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `return`
        let expr = self.parse_expression().unwrap().0; // todo: remove unwrap
        Ok((
            ExprValue::Return(Box::new(expr)),
            NodePosition {
                pos: nx.pos,
                line_no: nx.line_no,
                file: nx.file,
            },
        ))
    }

    pub fn parse_string(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next());
        match nx.type_ {
            TokenType::Str(s) => Ok((
                ExprValue::Str(s),
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            )),
            _ => unreachable!(),
        }
    }

    pub fn parse_use(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `use`
        self.advance();
        match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Str(s) => Ok((
                ExprValue::Use(s.to_string()),
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            )),
            _ => Err(self.parser_error("Invalid 'use' expression")),
        }
    }

    pub fn parse_extern(&mut self) -> Result<(ExprValue, NodePosition)> {
        self.advance();
        let nx = unwrap_some!(self.tokens.next()); // Eat `extern`
        self.advance();
        match unwrap_some!(self.tokens.next()).type_ {
            TokenType::Str(s) => Ok((
                ExprValue::Extern(s.to_string()),
                NodePosition {
                    pos: nx.pos,
                    line_no: nx.line_no,
                    file: nx.file,
                },
            )),
            _ => Err(self.parser_error("Invalid 'extern' expression")),
        }
    }
}
