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
            let f = match self.eval(xs[0].clone()){
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            xs.remove(0);

            match f {
                MalType::BuiltInFunction(func_type) => {
                    self.call_built_in_function(func_type,xs)
                },
                MalType::Function(_,_,_) =>{
                    let (argnames,body,is_rest) = f.unwrap_function().unwrap();
                    self.env.let_start();
                    let ret = self.call_function(argnames,body,is_rest,xs);
                    self.env.let_end();
                    ret
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
            match self.eval(x){
                Ok(y) => ys.push(y),
                Err(e) => return Err(e),
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

    fn call_function(&mut self,names: Vec<String>,body:MalType,is_rest:bool,args:Vec<MalType>)
        -> Result<MalType,String>{
        
        // evaluate arguments
        let args = match self.eval_sequence(args){
            Err(e) => return Err(e),
            Ok(v) => v,
        };

        if is_rest{
            if names.len() - 1 > args.len() {
                return Err(
                    format!(
                        "This function needs at least {} arguments, we got {}."
                        ,names.len()-1
                        ,args.len()));
            }
            
            // split normal argument and & rest arguments
            let (args,rest_val) = args.split_at(names.len()-1);
            let (names,rest_name) = names.split_at(names.len()-1);

            // assign arguments
            for (name,val) in names.into_iter().zip(args.into_iter()){
                self.env.set(name.clone(),val.clone());
            }
            self.env.set(rest_name[0].clone(),MalType::List(rest_val.to_vec()));
        }else{
            if names.len() != args.len(){
                return Err(
                    format!(
                        "This function needs exactly {} arguments, we got {}."
                        ,names.len()
                        ,args.len()));
            }

            // assign arguments
            for (name,val) in names.into_iter().zip(args.into_iter()){
                self.env.set(name,val);
            }
        }

        self.eval(body)
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
            },
            BuiltInFunction::Fn =>{
                self.mal_fn(xs)
            },
            BuiltInFunction::If =>{
                Err(format!("unimplemented if*"))
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
            let val = match self.eval(xs[1].clone()){
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            
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
            let vars = match xs[0].unwrap_list_vector(){
                Some(v)=>v,
                None => return Err(format!(
                    "The first argument of let* must be list or vector. We get {:?}.",xs[0])),
            };

            let vars = match sequence_to_pair(vars){
                Ok(v) => v,
                Err(e) => return Err(e),
            };

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

    fn mal_fn(&mut self,xs: Vec<MalType>)->Result<MalType,String>{
        // (fn* [n] (+ n 1))
        if xs.len() != 2{
            return Err(format!(
                "The function fn* needs exactly 2 arguments, we got {}.",xs.len()))
        }
        // take out vec from xs[1]
        let arg_vec = match xs[0].clone() {
            MalType::Vector(v) => v,
            MalType::List(v) => v,
            _ => return Err(format!(
                "The second argument of fn* must be sequcence, we got {:?}.",xs[1])),
        };

        // take out parameter names from vec
        let mut names = vec![];
        for arg in arg_vec{
            match arg.unwrap_identifier(){
                Some(v) => names.push(v),
                None => return Err(format!(
                    "The argument name is must be identifier, we got {:?}.",arg))
            }
        }

        // detect position of & and remove it
        let mut is_rest = false;
        for i in 0..names.len(){
            if names[i] != "&" {
                continue;
            }

            if i == names.len() - 2{
                is_rest = true;
            }else{
                return Err(format!(
                    "The {} nth parameter cannot be variadic function parameter because this is not the last parameter."
                    ,i+2));
            }
        }

        if is_rest{
            let pos = names.len() - 2;
            names.remove(pos);
        }


        Ok(MalType::Function(names,Box::new(xs[1].clone()),is_rest))
    }
}
