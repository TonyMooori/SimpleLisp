
pub mod interpreter;
pub mod reader;
pub mod printer;
pub mod evaluater;

use interpreter::Interpreter;

fn main() {
    let lisp = Interpreter::new();

    lisp.repl_loop();
}
