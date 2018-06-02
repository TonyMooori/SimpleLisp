use std::io;

pub struct Interpreter{

}

impl Interpreter{
    pub fn new()->Interpreter{
        Interpreter{
            
        }
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

impl Interpreter{
    pub fn rep(&self,s:String){
        let ast = self.read(s);
        let result = self.eval(ast);

        self.print(result);
    }
}

impl Interpreter{
    pub fn read(&self,s:String) -> String{
        s
    }
}

impl Interpreter{
    pub fn eval(&self,s:String)-> String{
        s
    }
}

impl Interpreter{
    pub fn print(&self,s:String)->String{
        println!("{}",s);
        s
    }
}

