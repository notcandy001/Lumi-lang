// ============================================================
//  Lumi Language — Main Entry Point
// ============================================================

mod lexer;
mod ast;
mod parser;
mod interpreter;
mod gui;

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
        "--example"     => run_source(EXAMPLE_PROGRAM, false, true),
        "--ast" => {
            if args.len() < 3 {
                eprintln!("Usage: lumi --ast <file.lu>");
                process::exit(1);
            }
            run_file(&args[2], true, false);
        }
        "--print" => {
            // Legacy: just print the component tree, no GUI
            if args.len() < 3 {
                eprintln!("Usage: lumi --print <file.lu>");
                process::exit(1);
            }
            run_file(&args[2], false, false);
        }
        path => run_file(path, false, true),
    }
}

fn run_file(path: &str, show_ast: bool, launch_gui: bool) {
    if !path.ends_with(".lu") {
        eprintln!("Warning: Lumi files should end in .lu");
    }
    let source = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Cannot read '{}': {}", path, e); process::exit(1); }
    };
    run_source(&source, show_ast, launch_gui);
}

fn run_source(source: &str, show_ast: bool, launch_gui: bool) {
    // Lex
    let tokens = match lex(source) {
        Ok(t) => t,
        Err(e) => { eprintln!("Lex Error: {}", e); process::exit(1); }
    };

    // Parse
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

    // Interpret
    let mut interp = Interpreter::new();
    if let Err(e) = interp.run(&program) {
        eprintln!("Runtime Error: {}", e);
        process::exit(1);
    }

    // Launch GUI or print tree
    if launch_gui {
        // Find the root window component
        let window = interp.components.into_iter().find(|c| c.kind == "window");
        match window {
            Some(root) => {
                if let Err(e) = gui::run(root) {
                    eprintln!("GUI Error: {:?}", e);
                    process::exit(1);
                }
            }
            None => {
                eprintln!("No 'window' component found. Use 'create window <name>:' as your root.");
                process::exit(1);
            }
        }
    } else {
        // --print mode: legacy terminal output
        println!("-- Component Tree --");
        for comp in &interp.components {
            print_component(comp, 0);
        }
    }
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
    println!("Lumi v0.2 - Code, simplified.");
    println!("Usage:");
    println!("  lumi <file.lu>           Run and open GUI window");
    println!("  lumi --example           Run the built-in example");
    println!("  lumi --print <file.lu>   Print component tree (no GUI)");
    println!("  lumi --ast <file.lu>     Print AST");
    println!("  lumi --help              Show this message");
}

const EXAMPLE_PROGRAM: &str = r#"
create window main:
    width is 800
    height is 500
    title is "Lumi Demo App"

    create layout content:
        direction is "vertical"
        spacing is 16

        create text headline:
            content is "Welcome to Lumi"
            size is 32

        create text subtitle:
            content is "A declarative UI language powered by Rust"
            size is 16

        create input name_field:
            placeholder is "Enter your name"
            value is ""

        create button greet:
            text is "Say Hello"
            on click:
                print "Hello from Lumi!"

        create button count:
            text is "Click Me"
            on click:
                print "Button was clicked!"
"#;
