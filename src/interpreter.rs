use env::Env;
use types::MalType;
use std::io;

pub struct Interpreter{
    pub env : Env
}

impl Interpreter{
    pub fn new()->Interpreter{
        Interpreter{
            env : Env::new(),
        }
    }
}

impl Interpreter{
    fn rep(&mut self,s:String){
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

        self.print(last);
    }
}

impl Interpreter{
    pub fn repl_loop(&mut self){
        loop{
            let code = self.read_code();
            self.rep(code);
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
}
