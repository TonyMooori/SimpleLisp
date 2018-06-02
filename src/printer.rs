use interpreter::Interpreter;

impl Interpreter{
    pub fn print(&self,s:String)->String{
        println!("{}",s);
        s
    }
}
