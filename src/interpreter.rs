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
    ) -> Result<(), RuntimeError> {
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
                if is_truthy(&cond) {
                    for s in then_body {
                        self.exec_statement(s, local)?;
                    }
                } else if let Some(else_stmts) = else_body {
                    for s in else_stmts {
                        self.exec_statement(s, local)?;
                    }
                }
            }

            Statement::While { .. } | Statement::Return(_) => todo!(),
        }
        Ok(())
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

        println!("▸ Creating {} \"{}\"", kind, name);

        for item in body {
            match item {
                ComponentItem::Property { name: prop_name, value } => {
                    let v = self.eval_expr(value, local)?;
                    println!("  {} = {}", prop_name, v);
                    instance.properties.insert(prop_name.clone(), v);
                }

                ComponentItem::EventHandler { event, body: handler_body } => {
                    println!("  [event: on {}]", event);
                    // In a real UI runtime this would register a callback.
                    // For the interpreter demo we fire the handler immediately
                    // so you can see console output.
                    for stmt in handler_body {
                        self.exec_statement(stmt, local)?;
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
        _ => todo!()
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
