# Lumi Language Specification v0.1
## "Code, simplified."

---

## 1. Overview

Lumi is a declarative, English-like programming language for building UI and structured applications.
File extension: `.lu`  
Entry keyword: `lumi` (future: module system root)  
Runtime: Rust-powered interpreter (tree-walk, v0.1)

---

## 2. Philosophy

| Principle | Meaning |
|-----------|---------|
| **Readable first** | Code reads like English prose |
| **Minimal symbols** | No braces, semicolons, or sigils |
| **Declarative UI** | Describe *what*, not *how* |
| **Indentation structure** | Hierarchy via whitespace (like Python) |
| **English keywords** | `is`, `on`, `create`, `let`, `print` |

---

## 3. Full Grammar (EBNF)

```ebnf
program       = { statement } ;

statement     = component_def
              | var_decl
              | var_set
              | print_stmt
              | if_else_stmt ;

component_def = "create" IDENT IDENT ":" NEWLINE
                INDENT { component_item } DEDENT ;

component_item = property
               | event_handler
               | component_def ;

property      = IDENT "is" expr NEWLINE ;

event_handler = "on" IDENT ":" NEWLINE
                INDENT { statement } DEDENT ;

var_decl      = "let" IDENT "is" expr NEWLINE ;
var_set       = "set" IDENT "is" expr NEWLINE ;
print_stmt    = "print" expr NEWLINE ;

if_else_stmt  = "if" expr ":" NEWLINE
                INDENT { statement } DEDENT
                [ "else" ":" NEWLINE INDENT { statement } DEDENT ] ;

expr          = or_expr ;
or_expr       = and_expr { "or" and_expr } ;
and_expr      = unary_expr { "and" unary_expr } ;
unary_expr    = [ "not" ] primary ;
primary       = STRING | NUMBER | "true" | "false" | IDENT ;

STRING        = '"' { char } '"' ;
NUMBER        = [ '-' ] DIGIT { DIGIT } [ '.' DIGIT { DIGIT } ] ;
IDENT         = ALPHA { ALPHA | DIGIT | '_' } ;
```

---

## 4. Keywords

| Keyword | Role |
|---------|------|
| `create` | Define a new UI component |
| `is` | Property assignment / comparison |
| `on` | Event handler declaration |
| `let` | Declare a variable |
| `set` | Reassign a variable |
| `print` | Output to console |
| `if` / `else` | Conditional branching |
| `true` / `false` | Boolean literals |
| `and` / `or` / `not` | Boolean operators |

---

## 5. Built-in Components

### `window`
The root application window.

```lumi
create window main:
    width is 1024
    height is 768
    title is "My App"
```

| Property | Type | Default |
|----------|------|---------|
| `width` | number | 800 |
| `height` | number | 600 |
| `title` | string | "Lumi Window" |

---

### `button`
An interactive button element.

```lumi
create button submit:
    text is "Submit"
    on click:
        print "Submitted!"
```

| Property | Type | Default |
|----------|------|---------|
| `text` | string | "Button" |

**Events:** `click`

---

### `text`
A text display element.

```lumi
create text label:
    content is "Hello, World!"
    size is 18
```

| Property | Type | Default |
|----------|------|---------|
| `content` | string | "" |
| `size` | number | 14 |

---

### `input`
A text input field.

```lumi
create input email:
    placeholder is "Enter your email"
    value is ""
```

| Property | Type | Default |
|----------|------|---------|
| `placeholder` | string | "" |
| `value` | string | "" |

---

### `layout`
A container for grouping and arranging children.

```lumi
create layout sidebar:
    direction is "vertical"
    spacing is 12
```

| Property | Type | Default |
|----------|------|---------|
| `direction` | string | "vertical" |
| `spacing` | number | 8 |

---

## 6. Variables

```lumi
# Declaration
let name is "Nisa"
let count is 42
let active is true

# Reassignment
set count is 100
set name is "Lumi"
```

---

## 7. Events

Events use the `on <event_name>:` syntax inside a component body.

```lumi
create button btn:
    text is "Go"
    on click:
        print "Button clicked!"
        let msg is "Hello"
        print msg
```

Supported events (v0.1):
- `click` — user clicks the component

---

## 8. Control Flow

### if / else

```lumi
let logged_in is true

if logged_in:
    print "Welcome back!"
else:
    print "Please sign in."
```

### Boolean expressions

```lumi
let a is true
let b is false

if a and not b:
    print "Only a is true"

if a or b:
    print "At least one is true"
```

---

## 9. Comments

Use `#` for line comments:

```lumi
# This is a comment
create window main:  # inline comment
    width is 800     # pixels
```

---

## 10. Full Example Program

```lumi
# dashboard.lu
# A simple analytics dashboard

let page_title is "Analytics"
let user_name is "Nisa"

create window dashboard:
    width is 1200
    height is 800
    title is "Lumi Analytics"

    create layout main_layout:
        direction is "horizontal"
        spacing is 0

        create layout sidebar:
            direction is "vertical"
            spacing is 8

            create text logo:
                content is "Lumi"
                size is 28

            create button nav_home:
                text is "Home"
                on click:
                    print "Navigating to Home"

            create button nav_reports:
                text is "Reports"
                on click:
                    print "Navigating to Reports"

        create layout content_area:
            direction is "vertical"
            spacing is 16

            create text page_heading:
                content is page_title
                size is 26

            create text welcome_msg:
                content is "Good morning!"
                size is 16

            create button refresh:
                text is "Refresh Data"
                on click:
                    print "Refreshing dashboard..."
                    print "Data up to date."

print "Dashboard loaded."
print user_name
```

---

## 11. Architecture

```
Source (.lu)
    │
    ▼
┌─────────┐
│  Lexer  │  → Tokens (with INDENT/DEDENT)
└─────────┘
    │
    ▼
┌─────────┐
│ Parser  │  → AST (Program / Statement / Expr)
└─────────┘
    │
    ▼
┌─────────────┐
│ Interpreter │  → ComponentInstances + stdout
└─────────────┘
```

### Token Types
```
Create, Is, On, If, Else, Print, Let, Set,
True, False, And, Or, Not,
Identifier(String), StringLit(String), NumberLit(f64),
Colon, Newline, Indent, Dedent, Eof
```

### AST Node Types
```
Statement::ComponentDef { kind, name, body }
Statement::VarDecl { name, value }
Statement::VarSet { name, value }
Statement::Print(Expr)
Statement::IfElse { condition, then_body, else_body }

ComponentItem::Property { name, value }
ComponentItem::EventHandler { event, body }
ComponentItem::Child(Statement)

Expr::StringLit | NumberLit | BoolLit | Var | BinOp | Not
```

---

## 12. Running Lumi

### Build from source
```bash
git clone https://github.com/lumi-lang/lumi
cd lumi
cargo build --release
```

### Run a program
```bash
./lumi myapp.lu
```

### View the AST
```bash
./lumi --ast myapp.lu
```

### Run the built-in example
```bash
./lumi --example
```

---

## 13. Lumi vs QML Comparison

| Feature | Lumi | QML |
|---------|------|-----|
| Syntax style | English-like prose | JavaScript-adjacent |
| Property assignment | `width is 500` | `width: 500` |
| Event handlers | `on click:` | `onClicked: {}` |
| Variables | `let x is 5` | `property int x: 5` |
| Backend | Rust | C++ / Qt |
| Learning curve | Minimal | Moderate |
| Indentation | Meaningful | Optional |
| Comments | `#` | `//` |
| Target audience | Beginners + UI devs | Qt ecosystem devs |

---

## 14. Roadmap

### v0.2 — Logic
- `while` loops
- `repeat N times` loops
- `greater than`, `less than` comparisons
- String interpolation: `"Hello {name}"`

### v0.3 — Modules
- `import` system
- `Nisa` entry module
- Multi-file programs

### v0.4 — UI Runtime
- Connect to a real rendering backend (egui / wgpu)
- Hot reload on file save
- Live component preview

### v0.5 — Tooling
- LSP (Language Server Protocol) for IDE support
- Syntax highlighting for VS Code
- Error recovery in parser
- REPL (interactive mode)

### v1.0 — Production
- Compile to native binary
- WASM target
- Package manager ("Glow")
- Standard library

---

## 15. Taglines

1. **"Code, simplified."** — Clean and direct
2. **"Write in light."** — Poetic, on-brand with "Lumi"
3. **"Think it. Write it. Lumi."** — Action-oriented
4. **"Programming, made clear."** — Approachable
5. **"Light syntax. Powerful ideas."** — Developer-facing
6. **"The language that gets out of your way."** — Pragmatic
