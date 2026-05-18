//  Lumi Language — Parser  (v0.2)
//  Converts a flat token stream into an AST.
//  Expression precedence (low → high):
//    or → and → not → comparison → add/sub → mul/div/mod → unary − → primary
// ============================================================

use crate::ast::*;
use crate::lexer::Token;

#[derive(Debug)]
pub struct ParseError {
    pub message: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ParseError: {}", self.message)
    }
}

pub struct Parser {
    tokens: Vec<Token>,
    pos:    usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── Primitives ────────────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
    }

    fn peek2(&self) -> &Token {
        self.tokens.get(self.pos + 1).unwrap_or(&Token::Eof)
    }

    fn advance(&mut self) -> Token {
        let t = self.tokens[self.pos].clone();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<Token, ParseError> {
        let t = self.advance();
        if std::mem::discriminant(&t) == std::mem::discriminant(expected) {
            Ok(t)
        } else {
            Err(ParseError {
                message: format!("Expected {:?}, got {:?}", expected, t),
            })
        }
    }

    fn skip_newlines(&mut self) {
        while self.peek() == &Token::Newline { self.advance(); }
    }

    fn consume_newline(&mut self) {
        if self.peek() == &Token::Newline { self.advance(); }
    }

    // ── Program ───────────────────────────────────────────────

    pub fn parse_program(&mut self) -> Result<Program, ParseError> {
        let mut stmts = Vec::new();
        self.skip_newlines();
        while self.peek() != &Token::Eof {
            let stmt = self.parse_statement()?;
            stmts.push(stmt);
            self.skip_newlines();
        }
        Ok(stmts)
    }

    // ── Statements ────────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().clone() {
            Token::Create => self.parse_component(),
            Token::Let    => self.parse_var_decl(),
            Token::Set    => self.parse_var_set(),
            Token::Print | Token::Say => self.parse_print(),
            Token::If     => self.parse_if_else(),
            Token::While  => self.parse_while(),
            Token::Return => self.parse_return(),
            t => Err(ParseError {
                message: format!("Unexpected token at statement level: {:?}", t),
            }),
        }
    }

    /// create <kind> <name>:
    ///     <body>
    fn parse_component(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // consume `create`

        let kind = match self.advance() {
            Token::Identifier(k) => k,
            t => return Err(ParseError {
                message: format!("Expected component kind after 'create', got {:?}", t),
            }),
        };

        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => return Err(ParseError {
                message: format!("Expected component name, got {:?}", t),
            }),
        };

        self.expect(&Token::Colon)?;
        self.consume_newline();

        let body = self.parse_component_body()?;
        Ok(Statement::ComponentDef { kind, name, body })
    }

    fn parse_component_body(&mut self) -> Result<Vec<ComponentItem>, ParseError> {
        let mut items = Vec::new();
        if self.peek() != &Token::Indent {
            return Ok(items);
        }
        self.advance(); // consume Indent

        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Dedent | Token::Eof => break,
                Token::On => {
                    items.push(self.parse_event_handler()?);
                }
                Token::Create => {
                    let child = self.parse_component()?;
                    items.push(ComponentItem::Child(child));
                }
                Token::Identifier(name) => {
                    // property: <name> is <expr>
                    // But make sure next is `is` — otherwise it's unexpected
                    if self.peek2() == &Token::Is {
                        self.advance(); // consume name
                        self.advance(); // consume `is`
                        let value = self.parse_expr()?;
                        self.consume_newline();
                        items.push(ComponentItem::Property { name, value });
                    } else {
                        return Err(ParseError {
                            message: format!(
                                "Expected 'is' after property name '{}', got {:?}",
                                name, self.peek2()
                            ),
                        });
                    }
                }
                t => return Err(ParseError {
                    message: format!("Unexpected token in component body: {:?}", t),
                }),
            }
        }

        if self.peek() == &Token::Dedent { self.advance(); }
        Ok(items)
    }

    /// on <event>:
    ///     <body>
    fn parse_event_handler(&mut self) -> Result<ComponentItem, ParseError> {
        self.advance(); // consume `on`
        let event = match self.advance() {
            Token::Identifier(e) => e,
            t => return Err(ParseError {
                message: format!("Expected event name after 'on', got {:?}", t),
            }),
        };
        self.expect(&Token::Colon)?;
        self.consume_newline();
        let body = self.parse_block()?;
        Ok(ComponentItem::EventHandler { event, body })
    }

    /// An indented block of statements.
    fn parse_block(&mut self) -> Result<Vec<Statement>, ParseError> {
        let mut stmts = Vec::new();
        if self.peek() != &Token::Indent {
            return Ok(stmts);
        }
        self.advance(); // consume Indent

        loop {
            self.skip_newlines();
            match self.peek() {
                Token::Dedent | Token::Eof => break,
                _ => stmts.push(self.parse_statement()?),
            }
        }

        if self.peek() == &Token::Dedent { self.advance(); }
        Ok(stmts)
    }

    fn parse_var_decl(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `let`
        let name = self.expect_ident("variable name")?;
        self.expect(&Token::Is)?;
        let value = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::VarDecl { name, value })
    }

    fn parse_var_set(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `set`
        let name = self.expect_ident("variable name")?;
        self.expect(&Token::Is)?;
        let value = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::VarSet { name, value })
    }

    fn parse_print(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `print` or `say`
        let expr = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::Print(expr))
    }

    fn parse_if_else(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `if`
        let condition = self.parse_expr()?;
        self.expect(&Token::Colon)?;
        self.consume_newline();
        let then_body = self.parse_block()?;

        let else_body = if self.peek() == &Token::Else {
            self.advance(); // `else`
            self.expect(&Token::Colon)?;
            self.consume_newline();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::IfElse { condition, then_body, else_body })
    }

    fn parse_while(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `while`
        let condition = self.parse_expr()?;
        self.expect(&Token::Colon)?;
        self.consume_newline();
        let body = self.parse_block()?;
        Ok(Statement::While { condition, body })
    }

    fn parse_return(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // `return`
        let expr = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::Return(expr))
    }

    // ── Expression parsing (Pratt-style precedence) ───────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp { left: Box::new(left), op: BinOpKind::Or, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_not_expr()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_not_expr()?;
            left = Expr::BinOp { left: Box::new(left), op: BinOpKind::And, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_not_expr(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == &Token::Not {
            self.advance();
            return Ok(Expr::Not(Box::new(self.parse_not_expr()?)));
        }
        self.parse_comparison()
    }

    fn parse_comparison(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_add_sub()?;

        let op = match self.peek() {
            Token::EqEq    => BinOpKind::Eq,
            Token::NotEq   => BinOpKind::NotEq,
            Token::Lt      => BinOpKind::Lt,
            Token::Gt      => BinOpKind::Gt,
            Token::LtEq    => BinOpKind::LtEq,
            Token::GtEq    => BinOpKind::GtEq,
            // English-style: "equals", "greater than", "less than"
            Token::Equals  => BinOpKind::Eq,
            Token::Greater => {
                // peek ahead for `than`
                if self.peek2() == &Token::Than {
                    self.advance(); // consume `greater`
                    self.advance(); // consume `than`
                    let right = self.parse_add_sub()?;
                    return Ok(Expr::BinOp { left: Box::new(left), op: BinOpKind::Gt, right: Box::new(right) });
                }
                return Ok(left);
            }
            Token::Less => {
                if self.peek2() == &Token::Than {
                    self.advance();
                    self.advance();
                    let right = self.parse_add_sub()?;
                    return Ok(Expr::BinOp { left: Box::new(left), op: BinOpKind::Lt, right: Box::new(right) });
                }
                return Ok(left);
            }
            _ => return Ok(left),
        };

        self.advance(); // consume operator
        let right = self.parse_add_sub()?;
        Ok(Expr::BinOp { left: Box::new(left), op, right: Box::new(right) })
    }

    fn parse_add_sub(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_mul_div()?;
        loop {
            let op = match self.peek() {
                Token::Plus  => BinOpKind::Add,
                Token::Minus => BinOpKind::Sub,
                _ => break,
            };
            self.advance();
            let right = self.parse_mul_div()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary()?;
        loop {
            let op = match self.peek() {
                Token::Star    => BinOpKind::Mul,
                Token::Slash   => BinOpKind::Div,
                Token::Percent => BinOpKind::Mod,
                _ => break,
            };
            self.advance();
            let right = self.parse_unary()?;
            left = Expr::BinOp { left: Box::new(left), op, right: Box::new(right) };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == &Token::Minus {
            self.advance();
            let operand = self.parse_primary()?;
            return Ok(Expr::BinOp {
                left:  Box::new(Expr::NumberLit(0.0)),
                op:    BinOpKind::Sub,
                right: Box::new(operand),
            });
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Token::StringLit(s)  => Ok(Expr::StringLit(s)),
            Token::NumberLit(n)  => Ok(Expr::NumberLit(n)),
            Token::True          => Ok(Expr::BoolLit(true)),
            Token::False         => Ok(Expr::BoolLit(false)),
            Token::Identifier(v) => Ok(Expr::Var(v)),
            Token::LParen => {
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            t => Err(ParseError {
                message: format!("Expected expression, got {:?}", t),
            }),
        }
    }

    // ── Helpers ───────────────────────────────────────────────

    fn expect_ident(&mut self, context: &str) -> Result<String, ParseError> {
        match self.advance() {
            Token::Identifier(n) => Ok(n),
            t => Err(ParseError {
                message: format!("Expected {} (identifier), got {:?}", context, t),
            }),
        }
    }
}
