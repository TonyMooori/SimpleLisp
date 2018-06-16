use interpreter::Interpreter;
use types::MalType;

impl Interpreter{
    pub fn print(&self,mt:Result<MalType,String>){
        match mt {
            Ok(v) => println!("{}",v.to_string(true)),
            Err(e) => println!("{}",e),
        }
    }
}
