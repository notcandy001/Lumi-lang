// ============================================================
//  Lumi Language — AST  (v0.2)
// ============================================================

pub type Program = Vec<Statement>;

#[derive(Debug, Clone)]
pub enum Statement {
    /// create <kind> <name>:
    ///     <body items>
    ComponentDef {
        kind: String,
        name: String,
        body: Vec<ComponentItem>,
    },

    /// let <name> is <expr>
    VarDecl { name: String, value: Expr },

    /// set <name> is <expr>
    VarSet { name: String, value: Expr },

    /// print / say <expr>
    Print(Expr),

    /// if <cond>:
    ///     <then>
    /// else:
    ///     <else>
    IfElse {
        condition: Expr,
        then_body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },

    /// while <cond>:
    ///     <body>
    While {
        condition: Expr,
        body: Vec<Statement>,
    },

    /// return <expr>
    Return(Expr),
}

#[derive(Debug, Clone)]
pub enum ComponentItem {
    /// <prop> is <expr>
    Property { name: String, value: Expr },

    /// on <event>:
    ///     <body>
    EventHandler { event: String, body: Vec<Statement> },

    /// Nested child component
    Child(Statement),
}

#[derive(Debug, Clone)]
pub enum Expr {
    StringLit(String),
    NumberLit(f64),
    BoolLit(bool),
    Var(String),

    BinOp {
        left:  Box<Expr>,
        op:    BinOpKind,
        right: Box<Expr>,
    },

    Not(Box<Expr>),
}

#[derive(Debug, Clone)]
pub enum BinOpKind {
    // Logical
    And, Or,
    // Comparison
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    // Arithmetic
    Add, Sub, Mul, Div, Mod,
}

impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::StringLit(s) => write!(f, "\"{}\"", s),
            Expr::NumberLit(n) => {
                if n.fract() == 0.0 { write!(f, "{}", *n as i64) }
                else { write!(f, "{}", n) }
            }
            Expr::BoolLit(b)   => write!(f, "{}", b),
            Expr::Var(v)       => write!(f, "{}", v),
            Expr::BinOp { left, op, right } => {
                write!(f, "({} {:?} {})", left, op, right)
            }
            Expr::Not(e) => write!(f, "(not {})", e),
        }
    }
}
