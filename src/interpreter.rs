use types::MalType;
use std::collections::HashMap;
use env::defualt_env;

pub struct Interpreter{
    pub env : HashMap<String,MalType>
}

impl Interpreter{
    pub fn new()->Interpreter{
        Interpreter{
            env : defualt_env()
        }
    }
}
