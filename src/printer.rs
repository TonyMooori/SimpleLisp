use interpreter::Interpreter;
use types::MalType;

impl Interpreter{
    pub fn print(&self,mt:MalType){
        println!("{:?}",mt);
    }

    // STEP 1: Defferrable 1
    // pub fn to_readable(&self:mt:MalType) -> String{
    //     match mt{
    //         MalType::Identifier(s) => s,
    //         MalType::Integer(n) => format!("{}",n),
    //         MalType::Str(s) => s, // TODO: "\n"
    //         MalType::Bool(f) => format!("{}",f),
    //         MalType::Vector(v) => {
    //         }
    //     }
    // }
}
