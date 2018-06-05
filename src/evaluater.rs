use interpreter::Interpreter;
use types::{MalType,BuiltInFunction};
use core::*;
use std::process::exit;

impl Interpreter{
    pub fn eval(&mut self,ast:MalType)-> Result<MalType,String>{
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

    fn eval_identifier(&mut self,ident:String)-> Result<MalType,String>{
        match self.env.get(&ident){
            Some(v) => Ok(v.clone()),
            None => Err(format!("Unknown symbol: {}",ident)),
        }
    }

    fn eval_list(&mut self,mut xs:Vec<MalType>)-> Result<MalType,String>{
        if xs.len() == 0{
            Ok(MalType::List(xs))
        }else{
            let f = self.eval(xs[0].clone());
            xs.remove(0);

            if f.is_err(){
                return f;
            }

            let f = f.unwrap();

            // let xs = self.eval_sequence(xs);
            // if let Err(e) = xs{
            //     return Err(e);
            // }
            // let xs = xs.unwrap();

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
    
    fn eval_sequence(&mut self,xs:Vec<MalType>)->Result<Vec<MalType>,String>{
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


    fn eval_vector(&mut self,xs:Vec<MalType>)-> Result<MalType,String>{
        let xs = self.eval_sequence(xs);

        if let Err(e) = xs{
            Err(e)
        }else{
            Ok(MalType::Vector(xs.unwrap()))
        }
    }

    fn call_built_in_function(&mut self,func_type:BuiltInFunction,xs: Vec<MalType>)
        -> Result<MalType,String>{

        match func_type{
            BuiltInFunction::Add => {
                let xs = self.eval_sequence(xs);
                match xs{
                    Ok(ys) => mal_add(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Sub => {
                let xs = self.eval_sequence(xs);
                match xs{
                    Ok(ys) => mal_sub(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Mul => {
                let xs = self.eval_sequence(xs);
                match xs{
                    Ok(ys) => mal_mul(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Div => {
                let xs = self.eval_sequence(xs);
                match xs{
                    Ok(ys) => mal_div(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::HashMap => {
                let xs = self.eval_sequence(xs);
                match xs{
                    Ok(ys) => mal_hashmap(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Exit =>{
                println!("Have a nice day!");
                exit(0)
            },
            BuiltInFunction::Def =>{
                self.mal_def(xs)
            },
            BuiltInFunction::Let =>{
                self.env.let_start();
                let ret = self.mal_let(xs);
                self.env.let_end();
                ret
            }
        }
    }
}


impl Interpreter{
    fn mal_def(&mut self,xs : Vec<MalType>)->Result<MalType,String>{
        if xs.len() != 2{
            Err(format!("The function def! needs exactly 2 arguments, we got {}.",xs.len()))
        }else{
            let sym = xs[0].clone();
            let val = self.eval(xs[1].clone());
            
            if let Err(e) = val{
                return Err(e);
            }
            let val = val.unwrap();

            match sym{
                MalType::Identifier(ident) => {
                    self.env.set(ident.clone(),val);
                    Ok(MalType::Identifier(ident))
                },

                _ =>
                    Err(format!("Cannot assign value to {:?}",sym)),
            }
        }
    }
    fn mal_let(&mut self,mut xs : Vec<MalType>)->Result<MalType,String>{
        if xs.len() <= 2{
            Err(format!("The function let* needs at least 2 arguments, we got {}.",xs.len()))
        }else{
            let vars = xs[0].unwrap_list_vector();
            if vars.is_none(){
                return Err(format!("The first argument of let* must be list or vector. We get {:?}.",xs[0]));
            }
            let vars = sequence_to_pair(vars.unwrap());
            if let Err(e) = vars{
                return Err(e);
            }
            let vars = vars.unwrap();
            
            for (name,val) in vars{
                if let Err(e) = self.mal_def(vec![name,val]){
                    return Err(e)
                }
            }
            
            xs.remove(0);
            match self.eval_sequence(xs){
                Ok(mut v) => Ok(v.pop().unwrap()),
                Err(e) => Err(e),
            }
        }
    }
}
