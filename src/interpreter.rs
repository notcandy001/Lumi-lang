// ============================================================
//  Lumi Language — Interpreter / Runtime
//  Tree-walk interpreter that executes the AST directly.
// ============================================================

use std::collections::HashMap;
use crate::ast::*;

// ── Runtime values ──────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Nil,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::String(s) => write!(f, "{}", s),
            Value::Number(n) => {
                if n.fract() == 0.0 { write!(f, "{}", *n as i64) }
                else { write!(f, "{}", n) }
            }
            Value::Bool(b) => write!(f, "{}", b),
            Value::Nil     => write!(f, "nil"),
        }
    }
}

// ── Component model ─────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct ComponentInstance {
    pub kind: String,
    pub name: String,
    pub properties: HashMap<String, Value>,
    pub children: Vec<ComponentInstance>,
}

impl ComponentInstance {
    fn new(kind: &str, name: &str) -> Self {
        let mut props = HashMap::new();
        // Built-in defaults per component kind
        match kind {
            "window" => {
                props.insert("width".into(),  Value::Number(800.0));
                props.insert("height".into(), Value::Number(600.0));
                props.insert("title".into(),  Value::String("Lumi Window".into()));
            }
            "button" => {
                props.insert("text".into(), Value::String("Button".into()));
            }
            "text" => {
                props.insert("content".into(), Value::String("".into()));
                props.insert("size".into(),    Value::Number(14.0));
            }
            "input" => {
                props.insert("placeholder".into(), Value::String("".into()));
                props.insert("value".into(),       Value::String("".into()));
            }
            "layout" => {
                props.insert("direction".into(), Value::String("vertical".into()));
                props.insert("spacing".into(),   Value::Number(8.0));
            }
            _ => {}
        }
        Self {
            kind: kind.to_string(),
            name: name.to_string(),
            properties: props,
            children: Vec::new(),
        }
    }
}

// ── Control flow signals ─────────────────────────────────────

#[derive(Debug)]
enum Signal {
    Break,
    Continue,
    Return(Value),
}

// ── Interpreter ──────────────────────────────────────────────

pub struct Interpreter {
    /// Global variable scope
    pub vars: HashMap<String, Value>,
    /// All top-level component instances
    pub components: Vec<ComponentInstance>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            components: Vec::new(),
        }
    }

    pub fn run(&mut self, program: &Program) -> Result<(), RuntimeError> {
        for stmt in program {
            self.exec_statement(stmt, &mut HashMap::new())?;
        }
        Ok(())
    }

    // ── Statement execution ──────────────────────────────────

    fn exec_statement(
        &mut self,
        stmt: &Statement,
        local: &mut HashMap<String, Value>,
    ) -> Result<Option<Signal>, RuntimeError> {
        match stmt {
            Statement::ComponentDef { kind, name, body } => {
                let component = self.build_component(kind, name, body, local)?;
                self.components.push(component);
            }

            Statement::VarDecl { name, value } => {
                let v = self.eval_expr(value, local)?;
                local.insert(name.clone(), v.clone());
                self.vars.insert(name.clone(), v);
            }

            Statement::VarSet { name, value } => {
                let v = self.eval_expr(value, local)?;
                if local.contains_key(name) {
                    local.insert(name.clone(), v.clone());
                }
                self.vars.insert(name.clone(), v);
            }

            Statement::Print(expr) => {
                let v = self.eval_expr(expr, local)?;
                println!("{}", v);
            }

            Statement::IfElse { condition, then_body, else_body } => {
                let cond = self.eval_expr(condition, local)?;
                let branch = if is_truthy(&cond) { Some(then_body) } else { else_body.as_ref() };
                if let Some(stmts) = branch {
                    for s in stmts {
                        if let Some(sig) = self.exec_statement(s, local)? {
                            return Ok(Some(sig));
                        }
                    }
                }
            }

            Statement::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition, local)?;
                    if !is_truthy(&cond) { break; }
                    for s in body {
                        match self.exec_statement(s, local)? {
                            Some(Signal::Break)    => return Ok(None),
                            Some(Signal::Continue) => break,
                            Some(sig)              => return Ok(Some(sig)),
                            None                   => {}
                        }
                    }
                }
            }

            Statement::Loop { body } => {
                loop {
                    for s in body {
                        match self.exec_statement(s, local)? {
                            Some(Signal::Break)    => return Ok(None),
                            Some(Signal::Continue) => break,
                            Some(sig)              => return Ok(Some(sig)),
                            None                   => {}
                        }
                    }
                }
            }

            Statement::For { var, start, end, body } => {
                let start_val = self.eval_expr(start, local)?;
                let end_val   = self.eval_expr(end,   local)?;
                let (s, e) = match (&start_val, &end_val) {
                    (Value::Number(a), Value::Number(b)) => (*a as i64, *b as i64),
                    _ => return Err(RuntimeError { message: "for loop bounds must be numbers".into() }),
                };
                'outer: for i in s..=e {
                    local.insert(var.clone(), Value::Number(i as f64));
                    self.vars.insert(var.clone(), Value::Number(i as f64));
                    for s in body {
                        match self.exec_statement(s, local)? {
                            Some(Signal::Break)    => break 'outer,
                            Some(Signal::Continue) => break,
                            Some(sig)              => return Ok(Some(sig)),
                            None                   => {}
                        }
                    }
                }
            }

            Statement::RepeatTimes { count, body } => {
                let n = match self.eval_expr(count, local)? {
                    Value::Number(n) => n as i64,
                    _ => return Err(RuntimeError { message: "'repeat N times' requires a number".into() }),
                };
                'outer: for _ in 0..n {
                    for s in body {
                        match self.exec_statement(s, local)? {
                            Some(Signal::Break)    => break 'outer,
                            Some(Signal::Continue) => break,
                            Some(sig)              => return Ok(Some(sig)),
                            None                   => {}
                        }
                    }
                }
            }

            Statement::Break    => return Ok(Some(Signal::Break)),
            Statement::Continue => return Ok(Some(Signal::Continue)),

            Statement::Return(expr) => {
                let v = self.eval_expr(expr, local)?;
                return Ok(Some(Signal::Return(v)));
            }
        }
        Ok(None)
    }

    // ── Component builder ────────────────────────────────────

    fn build_component(
        &mut self,
        kind: &str,
        name: &str,
        body: &[ComponentItem],
        local: &mut HashMap<String, Value>,
    ) -> Result<ComponentInstance, RuntimeError> {
        let mut instance = ComponentInstance::new(kind, name);

        for item in body {
            match item {
                ComponentItem::Property { name: prop_name, value } => {
                    let v = self.eval_expr(value, local)?;
                    instance.properties.insert(prop_name.clone(), v);
                }

                ComponentItem::EventHandler { event, body: handler_body } => {
                    // Capture print output from the handler body and store it
                    // as a `click_output` property so the GUI can replay it on click.
                    if event == "click" {
                        let mut captured = Vec::new();
                        for stmt in handler_body {
                            if let Statement::Print(expr) = stmt {
                                let v = self.eval_expr(expr, local)?;
                                captured.push(v.to_string());
                            } else {
                                self.exec_statement(stmt, local)?;
                            }
                        }
                        instance.properties.insert(
                            "click_output".to_string(),
                            Value::String(captured.join("\n")),
                        );
                    } else {
                        for stmt in handler_body {
                            self.exec_statement(stmt, local)?;
                        }
                    }
                }

                ComponentItem::Child(child_stmt) => {
                    if let Statement::ComponentDef { kind: ck, name: cn, body: cb } = child_stmt {
                        let child = self.build_component(ck, cn, cb, local)?;
                        instance.children.push(child);
                    }
                }
            }
        }

        Ok(instance)
    }

    // ── Expression evaluator ─────────────────────────────────

    fn eval_expr(
        &self,
        expr: &Expr,
        local: &HashMap<String, Value>,
    ) -> Result<Value, RuntimeError> {
        match expr {
            Expr::StringLit(s) => Ok(Value::String(s.clone())),
            Expr::NumberLit(n) => Ok(Value::Number(*n)),
            Expr::BoolLit(b)   => Ok(Value::Bool(*b)),

            Expr::Var(name) => {
                let value = local.get(name)
                    .or_else(|| self.vars.get(name))
                    .cloned()
                    .ok_or_else(|| RuntimeError {
                        message: format!("Undefined variable: '{}'", name),
                    })?;
                Ok(value)
            }

            Expr::Not(e) => {
                let v = self.eval_expr(e, local)?;
                Ok(Value::Bool(!is_truthy(&v)))
            }

            Expr::BinOp { left, op, right } => {
                let l = self.eval_expr(left, local)?;
                let r = self.eval_expr(right, local)?;
                eval_binop(op, l, r)
            }
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────

fn is_truthy(v: &Value) -> bool {
    match v {
        Value::Bool(b)   => *b,
        Value::Number(n) => *n != 0.0,
        Value::String(s) => !s.is_empty(),
        Value::Nil       => false,
    }
}

fn numeric_op<F: Fn(f64, f64) -> f64>(l: Value, r: Value, f: F) -> Result<Value, RuntimeError> {
    match (l, r) {
        (Value::Number(a), Value::Number(b)) => Ok(Value::Number(f(a, b))),
        (a, b) => Err(RuntimeError {
            message: format!("Numeric operation requires numbers, got {:?} and {:?}", a, b),
        }),
    }
}

fn eval_binop(op: &BinOpKind, l: Value, r: Value) -> Result<Value, RuntimeError> {
    match op {
        BinOpKind::And => Ok(Value::Bool(is_truthy(&l) && is_truthy(&r))),
        BinOpKind::Or  => Ok(Value::Bool(is_truthy(&l) || is_truthy(&r))),
        BinOpKind::Eq  => {
            let eq = match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => (a - b).abs() < 1e-10,
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Bool(a),   Value::Bool(b))   => a == b,
                _ => false,
            };
            Ok(Value::Bool(eq))
        }
        BinOpKind::Add => match (l, r) {
            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(a + &b)),
            (a, b) => Err(RuntimeError {
                message: format!("Cannot add {:?} and {:?}", a, b),
            }),
        },
        BinOpKind::Sub => numeric_op(l, r, |a, b| a - b),
        BinOpKind::Mul => numeric_op(l, r, |a, b| a * b),
        BinOpKind::Div => {
            if let (Value::Number(a), Value::Number(b)) = (&l, &r) {
                if *b == 0.0 {
                    return Err(RuntimeError { message: "Division by zero".into() });
                }
                return Ok(Value::Number(a / b));
            }
            Err(RuntimeError { message: "Division requires numbers".into() })
        }
        BinOpKind::Mod => {
            if let (Value::Number(a), Value::Number(b)) = (&l, &r) {
                if *b == 0.0 {
                    return Err(RuntimeError { message: "Modulo by zero".into() });
                }
                return Ok(Value::Number(a % b));
            }
            Err(RuntimeError { message: "Modulo requires numbers".into() })
        }
        BinOpKind::NotEq => {
            let eq = match (&l, &r) {
                (Value::Number(a), Value::Number(b)) => (a - b).abs() < 1e-10,
                (Value::String(a), Value::String(b)) => a == b,
                (Value::Bool(a),   Value::Bool(b))   => a == b,
                _ => false,
            };
            Ok(Value::Bool(!eq))
        }
        BinOpKind::Lt  => numeric_op(l, r, |a, b| if a <  b { 1.0 } else { 0.0 }).map(|v| Value::Bool(matches!(v, Value::Number(n) if n == 1.0))),
        BinOpKind::Gt  => numeric_op(l, r, |a, b| if a >  b { 1.0 } else { 0.0 }).map(|v| Value::Bool(matches!(v, Value::Number(n) if n == 1.0))),
        BinOpKind::LtEq => numeric_op(l, r, |a, b| if a <= b { 1.0 } else { 0.0 }).map(|v| Value::Bool(matches!(v, Value::Number(n) if n == 1.0))),
        BinOpKind::GtEq => numeric_op(l, r, |a, b| if a >= b { 1.0 } else { 0.0 }).map(|v| Value::Bool(matches!(v, Value::Number(n) if n == 1.0))),
    }
}

// ── Error type ───────────────────────────────────────────────

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RuntimeError: {}", self.message)
    }
}
