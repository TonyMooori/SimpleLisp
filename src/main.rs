
pub mod interpreter;

use interpreter::Interpreter;

fn main() {
    let lisp = Interpreter::new();

    lisp.repl_loop();
}
