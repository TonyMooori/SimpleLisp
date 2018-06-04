use std::collections::HashMap;
use types::{MalType,BuiltInFunction};

pub fn defualt_env()->HashMap<String,MalType>{
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

    env
}
