mod graph;
mod scope;
mod interpreter;
mod report;
mod ast;
mod parser;
mod exec;

use std::env;
use std::fs;
use std::process;

use interpreter::InterpreterState;
use report::SafetyReport;
use exec::{execute, ExecContext};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 2 {
        match args[1].as_str() {
            "--version" => {
                println!("chiru {}", env!("CARGO_PKG_VERSION"));
                process::exit(0);
            }
            "--help" | "-h" => {
                println!("Chiru — verification-first systems language");
                println!();
                println!("Usage:");
                println!("  chiru <file.chiru>     Verify a Chiru file");
                println!("  chiru --version        Show version");
                println!("  chiru --help           Show this help");
                process::exit(0);
            }
            _ => {}
        }
    }

    if args.len() != 2 {
        eprintln!("Usage: chiru <file.chiru>");
        process::exit(3);
    }

    let source = match fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            process::exit(3);
        }
    };

    let ast = match parser::parse(&source) {
        Ok(ast) => ast,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            process::exit(3);
        }
    };

    let mut state = InterpreterState::new();
    let mut ctx = ExecContext::new();

    if let Err(e) = execute(&ast, &mut state, &mut ctx) {
        eprintln!("Semantic error:\n{}", e);
        process::exit(2);
    }

    let report = SafetyReport::generate(&state);
    report.print(&state);

    let code = report.exit_code();
    eprintln!("DEBUG: exit code = {}", code);
    process::exit(code);
}
