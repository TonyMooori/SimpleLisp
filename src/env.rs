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

    pub fn let_start(&mut self){
        self.envs.push(HashMap::new());
    }

    pub fn let_end(&mut self){
        if self.envs.len() == 1{
            eprintln!("Something wrong in let_end.");
        }else{
            self.envs.pop();
        }
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
        env.insert(
            "let*".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Let));
        env.insert(
            "fn*".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Fn));
        env.insert(
            "if".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::If));
        env.insert(
            "<".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Lt));
        env.insert(
            "=".to_string(), 
            MalType::BuiltInFunction(BuiltInFunction::Eq));
        env.insert(
            "inc".to_string(),
            MalType::Function(
                vec!["n".to_string()],
                Box::new(MalType::List(
                    vec![
                        MalType::BuiltInFunction(BuiltInFunction::Add),
                        MalType::Identifier("n".to_string()),
                        MalType::Integer(1)])),
                false,
            ));
        env.insert(
            "list".to_string(),
            MalType::Function(
                vec!["rest".to_string()],
                Box::new(MalType::Identifier("rest".to_string())),
                true));
        env
    }
}