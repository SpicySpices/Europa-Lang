extern crate maplit;

mod environment;
mod error;
mod functions;
mod interpreter;
mod lexer;
mod nodes;
mod parser;
mod token;
mod types;

use std::io::{stdin, stdout, Write};
use std::time::Instant;
use std::{env, fs, process};

use interpreter::Interpreter;
use lexer::Lexer;
use parser::Parser;

use crate::environment::Environment;
use crate::error::Error;
use crate::nodes::stmt::Stmt;
use crate::token::Token;

use clap::{App, Arg};

const PROGRAM_NAME: &str = env!("CARGO_PKG_NAME");
const PROGRAM_VERSION: &str = env!("CARGO_PKG_VERSION");
const PROGRAM_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const PROGRAM_ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() {
    let matches = App::new(PROGRAM_NAME)
        .version(PROGRAM_VERSION)
        .author(PROGRAM_AUTHORS)
        .about(PROGRAM_ABOUT)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Log extra information"),
        )
        .arg(
            Arg::with_name("repl")
                .short("r")
                .long("repl")
                .help("Run file and continue to REPL"),
        )
        .arg(
            Arg::with_name("eval")
                .short("e")
                .long("eval")
                .value_name("CODE")
                .help("Evaluate CODE instead of a file")
                .takes_value(true)
                .conflicts_with("FILE"),
        )
        .arg(Arg::with_name("FILE").help("File to run").index(1))
        .get_matches();

    let verbose = matches.is_present("verbose");

    let code = if let Some(file) = matches.value_of("FILE") {
        // run file contents

        fs::read_to_string(file).unwrap_or_else(|err| {
            eprintln!("Error reading file: {}", err);
            process::exit(1)
        })
    } else if let Some(code) = matches.value_of("eval") {
        // run argument value

        String::from(code)
    } else {
        // no code to run, drop into repl

        println!("Welcome to the Europa Interactive Repl.");

        // start no-context repl
        let environ = Box::new(Environment::new());
        init_repl(environ, verbose);

        return;
    };

    // load and run code
    match init(code, Box::new(Environment::new()), verbose) {
        Err(e) => e.display(),
        Ok(environ) => {
            if matches.is_present("repl") {
                // drop into repl with environment
                init_repl(environ, verbose);
            }
        }
    }
}

// Loader for code, returns Environment mutated from environ
fn init(code: String, environ: Box<Environment>, verbose: bool) -> Result<Box<Environment>, Error> {
    let mut time = Instant::now();
    let tokens: Vec<Token> = match Lexer::new(&code).init() {
        Err(e) => return Err(e),
        Ok(toks) => {
            if verbose {
                eprintln!("lexer {:?}", time.elapsed());
            }

            toks
        }
    };

    // Turn tokens into AST
    time = Instant::now();
    let tree: Vec<Stmt> = match Parser::new(tokens).init() {
        Err(e) => return Err(e),
        Ok(tree) => {
            if verbose {
                eprintln!("parser {:?}", time.elapsed());
            }
            tree
        }
    };

    // Interpret and return environment
    time = Instant::now();
    let mut interpreter = Interpreter::new(tree, environ.clone());
    match interpreter.init() {
        Err(e) => return Err(e),
        Ok(_) => {
            if verbose {
                eprintln!("interpreter {:?}", time.elapsed());
            }

            Ok(interpreter.environ)
        }
    }
}

// Loops until exited
fn init_repl(mut environ: Box<Environment>, verbose: bool) {
    loop {
        // Same line print
        print!("\x1b[33m>\x1b[0m ");
        stdout().flush().unwrap();

        // Wait for input from user
        let mut input = String::new();
        match stdin().read_line(&mut input) {
            Err(e) => {
                println!("Unexpected REPL Error: {:?}", e);
                process::exit(1);
            }
            Ok(n) => {
                if n == 0 {
                    println!("\n");
                    process::exit(0);
                }
                input = input.trim().to_string();
            }
        }

        // Exit out of program
        if input.eq("exit") {
            process::exit(0);
        }

        // Attempt to run code
        match init(input, environ.clone(), verbose) {
            Err(e) => e.display(),
            Ok(env) => {
                // Change environ values if no errors
                environ = env;
                println!("{:?}", environ);
            }
        };
    }
}
