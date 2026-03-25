// ============================================================
//  Lumi Language — AST (Abstract Syntax Tree)
//  All nodes produced by the parser live here.
// ============================================================

/// A complete Lumi program is a list of top-level statements.
pub type Program = Vec<Statement>;

/// Every statement in a Lumi program.
#[derive(Debug, Clone)]
pub enum Statement {
    /// create <kind> <name>:
    ///     <properties>
    ///     <event handlers>
    ///     <child components>
    ComponentDef {
        kind: String,
        name: String,
        body: Vec<ComponentItem>,
    },

    /// let <name> is <value>
    VarDecl {
        name: String,
        value: Expr,
    },

    /// set <name> is <value>
    VarSet {
        name: String,
        value: Expr,
    },

    /// print <expr>
    Print(Expr),

    /// if <condition>:
    ///     <body>
    /// else:
    ///     <body>
    IfElse {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
}

/// Items that can appear inside a component body.
#[derive(Debug, Clone)]
pub enum ComponentItem {
    /// <property> is <value>
    Property {
        name: String,
        value: Expr,
    },

    /// on <event>:
    ///     <body>
    EventHandler {
        event: String,
        body: Vec<Statement>,
    },

    /// A nested child component
    Child(Statement),
}

/// Expressions — values and conditions.
#[derive(Debug, Clone)]
pub enum Expr {
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Var(String),
    BinOp {
        left: Box<Expr>,
        op: BinOpKind,
        right: Box<Expr>,
    },
    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOpKind {
    And,
    Or,
    Eq,   // is (inside expressions)
    Add,
    Sub,
    Mul,
    Div,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::StringLit(s) => write!(f, "\"{}\"", s),
            Expr::NumberLit(n) => write!(f, "{}", n),
            Expr::BoolLit(b)   => write!(f, "{}", b),
            Expr::Var(v)       => write!(f, "{}", v),
            Expr::BinOp { left, op, right } => {
                write!(f, "({:?} {:?} {:?})", left, op, right)
            }
            Expr::Not(e) => write!(f, "(not {:?})", e),
        }
    }
}
