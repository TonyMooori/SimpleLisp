
pub mod interpreter;
pub mod reader;
pub mod printer;
pub mod evaluater;
pub mod token;
pub mod lexer;

use interpreter::Interpreter;

fn main() {
    let lisp = Interpreter::new();

    lisp.repl_loop();
}
