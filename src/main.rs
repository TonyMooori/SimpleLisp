
pub mod interpreter;
pub mod reader;
pub mod printer;
pub mod evaluater;
pub mod types;
pub mod lexer;
pub mod core;
pub mod env;

use interpreter::Interpreter;

fn main() {
    let mut lisp = Interpreter::new();

    lisp.repl_loop();
}
