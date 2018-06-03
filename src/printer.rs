use interpreter::Interpreter;
use token::MalType;

impl Interpreter{
    pub fn print(&self,mt:MalType){
        println!("{:?}",mt);
    }
}
