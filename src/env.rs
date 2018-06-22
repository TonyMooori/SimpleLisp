use std::collections::HashMap;
use types::{MalType,BUILD_IN_FUNCTION_NAMES};
use std::env;

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

        for (f,ftype) in &BUILD_IN_FUNCTION_NAMES{
            env.insert(
                f.to_string(),
                MalType::BuiltInFunction(ftype.clone())
            );
        }

        let mut argv = vec![];
        for argument in env::args(){
            argv.push(MalType::Str(argument.to_string()));
        }
        argv.remove(0); // file path of program
        env.insert(
            "*ARGV*".to_string(),
            MalType::List(argv)
        );

        env
    }
}