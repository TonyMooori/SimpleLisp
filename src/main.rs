
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

    match lisp.load_file("lib.mal".to_string()){
        Ok(_) => {},
        Err(e) => println!("Setup error: {}",e) ,
    }

    lisp.repl_loop();
}
