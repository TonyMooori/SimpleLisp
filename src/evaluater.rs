use interpreter::Interpreter;
use types::{MalType,BuiltInFunction};
use std::io;
use core::*;
use std::process::exit;

impl Interpreter{
    fn eval(&self,ast:MalType)-> Result<MalType,String>{
        // eprintln!("evaluating {:?} in eval",ast);

        match ast{
            MalType::Identifier(ident) 
                => self.eval_identifier(ident),
            MalType::List(v)
                => self.eval_list(v),
            MalType::Vector(v)
                => self.eval_vector(v),
            _
                => Ok(ast)
        }
    }

    fn eval_identifier(&self,ident:String)-> Result<MalType,String>{
        match self.env.get(&ident){
            Some(v) => Ok(v.clone()),
            None => Err(format!("Unknown symbol: {}",ident)),
        }
    }

    fn eval_list(&self,mut xs:Vec<MalType>)-> Result<MalType,String>{
        if xs.len() == 0{
            Ok(MalType::List(xs))
        }else{
            let f = self.eval(xs[0].clone());
            xs.remove(0);

            if f.is_err(){
                return f;
            }

            let f = f.unwrap();

            // TODO: think about `if`
            let xs = self.eval_sequence(xs);

            if let Err(e) = xs{
                return Err(e);
            }

            let xs = xs.unwrap();

            match f {
                MalType::BuiltInFunction(func_type) => {
                    self.call_built_in_function(func_type,xs)
                },
                MalType::Function(_) =>{
                    Err(format!("Unimplemented."))
                },
                _ =>{
                    Err(format!("{:?} is not callable.",f))
                }
            }
        }
    }
    
    fn eval_sequence(&self,xs:Vec<MalType>)->Result<Vec<MalType>,String>{
        let mut ys = vec![];

        for x in xs{
            let y = self.eval(x);
            
            if let Err(e) = y{
                return Err(e);
            }else{
                ys.push(y.unwrap());
            }
        }

        Ok(ys)
    }


    fn eval_vector(&self,xs:Vec<MalType>)-> Result<MalType,String>{
        let xs = self.eval_sequence(xs);

        if let Err(e) = xs{
            Err(e)
        }else{
            Ok(MalType::Vector(xs.unwrap()))
        }
    }

    fn call_built_in_function(&self,func_type:BuiltInFunction,xs: Vec<MalType>)
        -> Result<MalType,String>{

        match func_type{
            BuiltInFunction::Add => {
                mal_add(xs)
            },
            BuiltInFunction::Sub => {
                mal_sub(xs)
            },
            BuiltInFunction::Mul => {
                mal_mul(xs)
            },
            BuiltInFunction::Div => {
                mal_div(xs)
            },
            BuiltInFunction::HashMap => {
                mal_hashmap(xs)
            },
            BuiltInFunction::Exit =>{
                println!("Have a nice day!");
                exit(0)
            }
        }
    }
}

impl Interpreter{
    fn rep(&self,s:String){
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
    pub fn repl_loop(&self){
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
