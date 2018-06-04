use std::collections::HashMap;
use types::{MalType,BuiltInFunction};

pub struct Env{
    envs : Vec<HashMap<String,MalType>>,
}

impl Env{
    pub fn new() -> Env{
        Env{
            envs : vec![Env::defualt_env()]
        }
    }

    pub fn get(&self,key: &String)->Option<MalType>{
        for i in (0..self.envs.len()).rev(){
            if let Some(v) = self.envs[i].get(key){
                return Some(v.clone());
            }
        }

        None
    }

    pub fn set(&mut self,key: String, val:MalType){
        let n = self.envs.len()-1;
        self.envs[n].insert(key,val);
    }

    fn defualt_env()->HashMap<String,MalType>{
        let mut env = HashMap::new();

        env.insert(
            "+".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Add));
        env.insert(
            "-".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Sub));
        env.insert(
            "*".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Mul));
        env.insert(
            "/".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Div));
        env.insert(
            "exit".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Exit));
        env.insert(
            "def!".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Def));

        env
    }
}