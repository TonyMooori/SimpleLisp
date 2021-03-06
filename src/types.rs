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
    // varnames, body, & rest, local_env, is_macro
    Function(Vec<String>,Box<MalType>,bool,HashMap<String,MalType>,bool), 
    BuiltInFunction(BuiltInFunction), 
    Keyword(String),
    Dict(HashMap<String,MalType>),
    Atom(usize),
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
    Nth,
    Rest,
    TypeStr,
    Insert,
    Eval,
    Err,
    PrintString,
    PrStr,
    Str,
    Apply,
    Do,
    Slurp,
    ReadString,
    Atom,
    AtomAt,
    Deref,
    Reset,
    UnQuote,
    SpliceUnQuote,
    QuasiQuote,
    ConCat,
    DefMacro,
    Throw,
    Try,
    Catch,
    Symbol,
    Keyword,
    Vector,
    Assoc,
    Get,
    Contains,
    Keys,
    Vals,
    Dissoc,
    ReadLine,
    Seq,
    TimeMs,
}

pub const BUILD_IN_FUNCTION_NAMES : [(&str,BuiltInFunction);51] = [
    ("+",BuiltInFunction::Add),
    ("-",BuiltInFunction::Sub),
    ("*",BuiltInFunction::Mul),
    ("/",BuiltInFunction::Div),
    ("hash-map",BuiltInFunction::HashMap),
    ("exit",BuiltInFunction::Exit),
    ("def!",BuiltInFunction::Def),
    ("let*",BuiltInFunction::Let),
    ("fn*",BuiltInFunction::Fn),
    ("if",BuiltInFunction::If),
    ("load-file",BuiltInFunction::LoadFile),
    ("<",BuiltInFunction::Lt),
    ("=",BuiltInFunction::Eq),
    ("quote",BuiltInFunction::Quote),
    ("nth",BuiltInFunction::Nth),
    ("rest",BuiltInFunction::Rest),
    ("type-str",BuiltInFunction::TypeStr),
    ("insert",BuiltInFunction::Insert),
    ("eval",BuiltInFunction::Eval),
    ("err",BuiltInFunction::Err),
    ("print-string",BuiltInFunction::PrintString),
    ("pr-str",BuiltInFunction::PrStr),
    ("str",BuiltInFunction::Str),
    ("apply",BuiltInFunction::Apply),
    ("do",BuiltInFunction::Do),
    ("slurp",BuiltInFunction::Slurp),
    ("read-string",BuiltInFunction::ReadString),
    ("atom",BuiltInFunction::Atom),
    ("atom-at",BuiltInFunction::AtomAt),
    ("deref",BuiltInFunction::Deref),
    ("reset!",BuiltInFunction::Reset),
    ("unquote",BuiltInFunction::UnQuote),
    ("splice-unquote",BuiltInFunction::SpliceUnQuote),
    ("quasiquote",BuiltInFunction::QuasiQuote),
    ("concat",BuiltInFunction::ConCat),
    ("defmacro!",BuiltInFunction::DefMacro),
    ("throw",BuiltInFunction::Throw),
    ("try*",BuiltInFunction::Try),
    ("catch*",BuiltInFunction::Catch),
    ("symbol",BuiltInFunction::Symbol),
    ("keyword",BuiltInFunction::Keyword),
    ("vector",BuiltInFunction::Vector),
    ("assoc",BuiltInFunction::Assoc),
    ("get",BuiltInFunction::Get),
    ("contains?",BuiltInFunction::Contains),
    ("keys",BuiltInFunction::Keys),
    ("vals",BuiltInFunction::Vals),
    ("dissoc",BuiltInFunction::Dissoc),
    ("readline",BuiltInFunction::ReadLine),
    ("seq",BuiltInFunction::Seq),
    ("time-ms",BuiltInFunction::TimeMs),
];

impl MalType{
    pub fn to_string(&self,print_readably:bool)->String{
        match self{
            MalType::Identifier(s) => {
                s.clone()
            },
            MalType::Integer(n) => {
                format!("{}",n)
            },
            MalType::Str(s) => {
                if print_readably{
                    format!("\"{}\"",
                        s.replace("\n","\\n")
                        .replace("\r","\\r")
                        .replace("\t","\\t")
                        .replace("\"","\\\""))
                }else{
                    s.clone()
                }
            },
            MalType::Bool(b) => {
                format!("{}",b)
            },
            MalType::Vector(v) => {
                let xs : Vec<String> = v
                    .iter()
                    .map(|x| x.to_string(print_readably))
                    .collect();
                let joined = xs.join(" ");

                format!("[{}]",joined)
            },
            MalType::List(v) => {
                let xs : Vec<String> = v
                    .iter()
                    .map(|x| x.to_string(print_readably))
                    .collect();
                let joined = xs.join(" ");

                format!("({})",joined)
            },
            MalType::Function(args,ast,flag,_,_)=>{
                let mut args = args.clone();
                if *flag{
                    let idx = args.len()-1;
                    args.insert(idx, "&".to_string());
                }

                format!("(fn* [{}] {})",
                    args.join(" "),
                    ast.to_string(print_readably))
            },
            MalType::BuiltInFunction(t) => {
                for (fname,ftype) in BUILD_IN_FUNCTION_NAMES.iter(){
                    if ftype == t{
                        return fname.to_string();
                    }
                }

                format!("unknown-build-in-function-{:?}",t)
            },
            MalType::Keyword(k) => {
                k.clone()
            },
            MalType::Nil => {
                "nil".to_string()
            },
            MalType::Dict(d) => {
                let mut xs = vec![];
                for (key,val) in d{
                    let mut key = key.clone();
                    let key = if key.chars().nth(0).unwrap() == ' '{
                        key.remove(0);
                        MalType::Str(key)
                    }else{
                        MalType::Keyword(key)
                    };
                    xs.push(
                        format!("{} {}",
                            key.to_string(true),
                            val.to_string(print_readably)));
                }
                let joined = xs.join(",");

                format!("{{{}}}",joined)
            },
            MalType::Atom(i) => {
                format!("(atom-at {})",i)
            },
        }
    }
}

impl MalType{
    pub fn unwrap_function(&self)->Option<(Vec<String>,MalType,bool,HashMap<String,MalType>,bool)>{
        if let MalType::Function(a,b,c,d,e) = self{
            // let b = b;
            // let b = (*b).clone();
            // let b = *b;
            // Some((a.clone(),b,c.clone()))
            Some((a.clone(),*((*b).clone()),c.clone(),d.clone(),e.clone()))
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

    pub fn unwrap_build_in_function(&self) -> Option<BuiltInFunction>{
        if let MalType::BuiltInFunction(v) = self{
            Some(v.clone())
        }else{
            None
        }
    }
}

impl MalType{
    pub fn is_list(&self)->bool{
        if let MalType::List(_)=self{
            true
        }else{
            false
        }
    }
    
    pub fn is_vector(&self)->bool{
        if let MalType::Vector(_)=self{
            true
        }else{
            false
        }
    }

    pub fn is_sequence(&self)->bool{
        self.is_list() || self.is_vector()
    }
}

impl MalType{
    pub fn get_all_identifier(&self) -> Vec<String>{
        match self {
            MalType::Identifier(s) => vec![s.clone()],
            MalType::List(xs) => {
                let mut ys = Vec::new();
                
                for x in xs.iter(){
                    ys.append(&mut (x.get_all_identifier()));
                }

                ys
            },
            MalType::Vector(xs) => {
                let mut ys = Vec::new();
                
                for x in xs.iter(){
                    ys.append(&mut (x.get_all_identifier()));
                }

                ys
            },
            _ => Vec::new(),
        }
    }
}