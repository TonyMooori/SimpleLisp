use env::Env;
use types::MalType;
use std::io;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

pub struct Interpreter{
    pub env : Env
}

impl Interpreter{
    pub fn new()->Interpreter{
        let mut lisp = Interpreter{
            env : Env::new(),
        };
        match lisp.load_file("lib.mal".to_string()){
            Ok(_) => {},
            Err(_) => {} ,
        }

        lisp
    }
}

impl Interpreter{
    fn rep(&mut self,s:String)->Result<MalType,String>{
        let asts = self.read(s); // Vec<Result<MalType,String>> 
        let mut last : Result<MalType,String> = Ok(MalType::Nil);

        if let Err(e) = asts{
            last = Err(format!("Parse error: {}",e));
        }else{
            let asts = asts.unwrap();
            for ast in asts{
                last = self.eval(ast);

                if let Err(e) = last{
                    last = Err(format!("Runtime error: {}",e));
                    break;
                }
            }
        }

        last
    }
}

impl Interpreter{
    pub fn repl_loop(&mut self){
        loop{
            let code = self.read_code();
            let last = self.rep(code);
            self.print(last);
        }
    }

    fn read_code(&self) -> String{
        let mut s = String::new();       
        // print!("user=>");
        io::stdin().read_line(&mut s).unwrap();
        
        if s.trim() == ""{
            s
        }else{
            format!("{}{}",s,self.read_code())
        }
    }

    pub fn load_file(&mut self,filename:String)->Result<MalType,String>{
        let file = match File::open(filename.clone()){
            Ok(v) => v,
            Err(_) => return Err(format!("Cannot open file {}.",filename)),
        };
        let mut buf_reader = BufReader::new(file);
        let mut code = String::new();
        match buf_reader.read_to_string(&mut code){
            Ok(_) => {},
            Err(_) => return Err(format!("Cannot read file {}.",filename)),
        }

        self.rep(code)
    }
}
