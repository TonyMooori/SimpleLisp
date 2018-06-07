use std::collections::HashMap;

#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind{
    Identifier(String), // def!,inc,dec,+,-,...
    Integer(i64), 
    Symbol(String),     // [],(),{},`,',@,~,~@,^
    Str(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token{
    pub kind : TokenKind,
    pub start : usize,
    pub end : usize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum MalType{
    Identifier(String),
    Integer(i64),
    Str(String),
    Bool(bool),
    Vector(Vec<MalType>),
    List(Vec<MalType>),
    Function(Vec<String>,Box<MalType>,bool), // varnames, body, & rest
    BuiltInFunction(BuiltInFunction), 
    Keyword(String),
    Dict(HashMap<String,MalType>),
    Nil,
}

#[derive(PartialEq, Debug, Clone)]
pub enum BuiltInFunction{
    Add,
    Sub,
    Mul,
    Div,
    HashMap,
    Exit,
    Def,
    Let,
    Fn,
    If,
    LoadFile,
    Lt,
    Eq,
    Quote,
    First,
    Rest,
    TypeStr,
    // Err,
    // Apply,
}

impl MalType{
    pub fn unwrap_function(&self)->Option<(Vec<String>,MalType,bool)>{
        if let MalType::Function(a,b,c) = self{
            // let b = b;
            // let b = (*b).clone();
            // let b = *b;
            // Some((a.clone(),b,c.clone()))
            Some((a.clone(),*((*b).clone()),c.clone()))
        }else{
            None
        }
    }

    pub fn unwrap_sequence(&self) -> Option<Vec<MalType>>{
        if let MalType::Vector(v) = self{
            Some(v.clone())
        }else if let MalType::List(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }

    pub fn unwrap_integer(&self) -> Option<i64>{
        if let MalType::Integer(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }
    
    pub fn unwrap_keyword(&self) -> Option<String>{
        if let MalType::Keyword(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }
    
    pub fn unwrap_bool(&self) -> Option<bool>{
        if let MalType::Bool(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }
    
    pub fn unwrap_identifier(&self) -> Option<String>{
        if let MalType::Identifier(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }
}
