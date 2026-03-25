// ============================================================
//  Lumi Language — Main Entry Point
// ============================================================

mod lexer;
mod ast;
mod parser;
mod interpreter;

use std::env;
use std::fs;
use std::process;

use lexer::lex;
use parser::Parser;
use interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "--help" | "-h" => print_usage(),
        "--example" => run_source(EXAMPLE_PROGRAM, false),
        "--ast" => {
            if args.len() < 3 {
                eprintln!("Usage: lumi --ast <file.lu>");
                process::exit(1);
            }
            run_file(&args[2], true);
        }
        path => run_file(path, false),
    }
}

fn run_file(path: &str, show_ast: bool) {
    if !path.ends_with(".lu") {
        eprintln!("Warning: Lumi files should end in .lu");
    }
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Cannot read '{}': {}", path, e); process::exit(1); }
    };
    run_source(&source, show_ast);
}

fn run_source(source: &str, show_ast: bool) {
    let tokens = match lex(source) {
        Ok(t) => t,
        Err(e) => { eprintln!("Lex Error: {}", e); process::exit(1); }
    };

    let mut p = Parser::new(tokens);
    let program = match p.parse_program() {
        Ok(prog) => prog,
        Err(e) => { eprintln!("Parse Error: {}", e); process::exit(1); }
    };

    if show_ast {
        println!("-- AST --");
        for node in &program { println!("{:#?}", node); }
        return;
    }

    println!("=== Lumi v0.1 =====================================");
    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("Runtime Error: {}", e);
        process::exit(1);
    }

    println!("\n-- Component Tree --");
    for comp in &interp.components {
        print_component(comp, 0);
    }
    println!("===================================================");
}

fn print_component(c: &interpreter::ComponentInstance, depth: usize) {
    let indent = "  ".repeat(depth);
    println!("{}[{}] \"{}\"", indent, c.kind, c.name);
    for (k, v) in &c.properties {
        println!("{}  .{} = {}", indent, k, v);
    }
    for child in &c.children {
        print_component(child, depth + 1);
    }
}

fn print_usage() {
    println!("Lumi v0.1 - Code, simplified.");
    println!("Usage: lumi <file.lu> | --example | --ast <file.lu> | --help");
}

const EXAMPLE_PROGRAM: &str = "
# Lumi Demo Program

create window main:
    width is 800
    height is 600
    title is \"Lumi Demo App\"

    create layout content:
        direction is \"vertical\"
        spacing is 16

        create text headline:
            content is \"Welcome to Lumi\"
            size is 32

        create button greet:
            text is \"Say Hello\"
            on click:
                print \"Hello from Lumi!\"

let app_name is \"Lumi Demo\"
print app_name

if true:
    print \"Lumi is running!\"
else:
    print \"Something went wrong.\"
";
