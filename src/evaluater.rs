use interpreter::Interpreter;
use types::MalType;
use std::io;

impl Interpreter{
    pub fn rep(&self,s:String){
        let asts = self.read(s); // Vec<Result<MalType,String>> 
        let mut last : MalType = MalType::Nil;

        if let Err(e) = asts{
            last = MalType::Error(e);
        }else{
            let asts = asts.unwrap();
            for ast in asts{
                last = self.eval(ast);
            }
        }

        self.print(last);
    }
}

impl Interpreter{
    pub fn eval(&self,ast:MalType)-> MalType{
        ast
    }
}

impl Interpreter{
    pub fn repl_loop(&self){
        loop{
            let code = self.read_code();
            self.rep(code);
        }
    }

    fn read_code(&self) -> String{
        let mut s = String::new();        
        io::stdin().read_line(&mut s).unwrap();
        
        if s.trim() == ""{
            s
        }else{
            format!("{}{}",s,self.read_code())
        }
    }
}
