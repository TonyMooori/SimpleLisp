use interpreter::Interpreter;
use types::MalType;

impl Interpreter{
    pub fn print(&self,mt:MalType){
        println!("{:?}",mt);
    }
}
