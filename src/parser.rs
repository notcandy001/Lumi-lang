// ============================================================
//  Lumi Language — Parser
//  Converts a flat token list into an AST.
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
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    // ── Low-level helpers ────────────────────────────────────

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&Token::Eof)
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
        while self.peek() == &Token::Newline {
            self.advance();
        }
    }

    fn consume_newline(&mut self) {
        if self.peek() == &Token::Newline {
            self.advance();
        }
    }

    // ── Top-level parse ──────────────────────────────────────

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

    // ── Statements ───────────────────────────────────────────

    fn parse_statement(&mut self) -> Result<Statement, ParseError> {
        match self.peek().clone() {
            Token::Create => self.parse_component(),
            Token::Let    => self.parse_var_decl(),
            Token::Set    => self.parse_var_set(),
            Token::Print  => self.parse_print(),
            Token::If     => self.parse_if_else(),
            t => Err(ParseError {
                message: format!("Unexpected token at statement level: {:?}", t),
            }),
        }
    }

    /// create <kind> <name>:
    ///     <component body>
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
            return Ok(items); // empty body
        }
        self.advance(); // consume Indent

        loop {
            self.skip_newlines();
            match self.peek().clone() {
                Token::Dedent | Token::Eof => break,

                Token::On => {
                    let handler = self.parse_event_handler()?;
                    items.push(handler);
                }

                Token::Create => {
                    let child = self.parse_component()?;
                    items.push(ComponentItem::Child(child));
                }

                Token::Identifier(name) => {
                    self.advance(); // consume property name
                    self.expect(&Token::Is)?;
                    let value = self.parse_expr()?;
                    self.consume_newline();
                    items.push(ComponentItem::Property { name, value });
                }

                t => return Err(ParseError {
                    message: format!("Unexpected token in component body: {:?}", t),
                }),
            }
        }

        if self.peek() == &Token::Dedent {
            self.advance();
        }

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

    /// A generic indented block of statements.
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
                _ => {
                    let stmt = self.parse_statement()?;
                    stmts.push(stmt);
                }
            }
        }

        if self.peek() == &Token::Dedent {
            self.advance();
        }

        Ok(stmts)
    }

    /// let <name> is <expr>
    fn parse_var_decl(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // consume `let`
        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => return Err(ParseError {
                message: format!("Expected variable name, got {:?}", t),
            }),
        };
        self.expect(&Token::Is)?;
        let value = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::VarDecl { name, value })
    }

    /// set <name> is <expr>
    fn parse_var_set(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // consume `set`
        let name = match self.advance() {
            Token::Identifier(n) => n,
            t => return Err(ParseError {
                message: format!("Expected variable name, got {:?}", t),
            }),
        };
        self.expect(&Token::Is)?;
        let value = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::VarSet { name, value })
    }

    /// print <expr>
    fn parse_print(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // consume `print`
        let expr = self.parse_expr()?;
        self.consume_newline();
        Ok(Statement::Print(expr))
    }

    /// if <condition>:
    ///     <body>
    /// else:
    ///     <body>
    fn parse_if_else(&mut self) -> Result<Statement, ParseError> {
        self.advance(); // consume `if`
        let condition = self.parse_expr()?;
        self.expect(&Token::Colon)?;
        self.consume_newline();
        let then_body = self.parse_block()?;

        let else_body = if self.peek() == &Token::Else {
            self.advance(); // consume `else`
            self.expect(&Token::Colon)?;
            self.consume_newline();
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Statement::IfElse { condition, then_body, else_body })
    }

    // ── Expression parsing (precedence climb) ────────────────

    fn parse_expr(&mut self) -> Result<Expr, ParseError> {
        self.parse_or_expr()
    }

    fn parse_or_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_and_expr()?;
        while self.peek() == &Token::Or {
            self.advance();
            let right = self.parse_and_expr()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOpKind::Or,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_and_expr(&mut self) -> Result<Expr, ParseError> {
        let mut left = self.parse_unary_expr()?;
        while self.peek() == &Token::And {
            self.advance();
            let right = self.parse_unary_expr()?;
            left = Expr::BinOp {
                left: Box::new(left),
                op: BinOpKind::And,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary_expr(&mut self) -> Result<Expr, ParseError> {
        if self.peek() == &Token::Not {
            self.advance();
            let expr = self.parse_primary()?;
            return Ok(Expr::Not(Box::new(expr)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.advance() {
            Token::StringLit(s) => Ok(Expr::StringLit(s)),
            Token::NumberLit(n) => Ok(Expr::NumberLit(n)),
            Token::True          => Ok(Expr::BoolLit(true)),
            Token::False         => Ok(Expr::BoolLit(false)),
            Token::Identifier(v) => Ok(Expr::Var(v)),
            t => Err(ParseError {
                message: format!("Expected expression, got {:?}", t),
            }),
        }
    }
}
