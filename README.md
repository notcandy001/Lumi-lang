#  Lumi Code, simplified.

> A modern, declarative, English-like programming language powered by Rust.
> File extension: `.lu` · 

---

## Quick Start

```bash
# Build
cargo build --release

# Run a program
./target/release/lumi myapp.lu

# Run the built-in example
./target/release/lumi --example

# Inspect the AST
./target/release/lumi --ast myapp.lu
```

---

## Hello, Lumi

```lumi
# hello.lu
create window main:
    width is 500
    height is 300
    title is "My First Lumi App"

    create button greet:
        text is "Say Hello"
        on click:
            print "Hello from Lumi!"

let name is "Nisa"
print name
```

---

## Language at a Glance

| Feature | Syntax |
|---------|--------|
| Create component | `create window main:` |
| Set property | `width is 500` |
| Event handler | `on click:` |
| Variable | `let x is 42` |
| Reassign | `set x is 100` |
| Print | `print "hello"` |
| Condition | `if logged_in:` |
| Comment | `# this is a comment` |

---

## Project Structure

```
lumi/
├── src/
│   ├── main.rs          # CLI entry point
│   ├── lexer.rs         # Tokenizer
│   ├── ast.rs           # AST node types
│   ├── parser.rs        # Recursive descent parser
│   └── interpreter.rs   # Tree-walk interpreter
├── examples/
│   ├── hello.lu
│   ├── counter.lu
│   ├── login.lu
├── SPEC.md              # Full language specification
├── lumi-docs.html       # Visual documentation site
└── lumi-logo.svg        # Brand logo
```

---

## Philosophy

- **Readable first** — Code reads like English
- **Minimal symbols** — No braces or semicolons
- **Declarative UI** — Describe *what*, not *how*
- **Safe** — Powered by Rust

---

*"Write in light."* · Lumi v0.1.0
