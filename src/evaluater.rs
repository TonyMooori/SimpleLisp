use interpreter::Interpreter;
use types::{MalType,BuiltInFunction};
use core::*;
use std::process::exit;
use std::collections::HashMap;

impl Interpreter{
    pub fn eval(&mut self,mut ast:MalType)-> Result<MalType,String>{
        // eprintln!("evaluating {:?} in eval",ast);
        let mut result : Result<MalType,String> = Ok(MalType::Nil);
        let mut n_let = 0;
        
        loop{
            if let MalType::Identifier(ident) = ast{
                result = self.eval_identifier(ident);
                break;
            }else if let MalType::Vector(v) = ast{
                result = self.eval_vector(v);
                break;
            }else if ast.is_list() == false {
                result = Ok(ast);
                break;
            }

            // listの場合
            let mut xs = ast.unwrap_sequence().unwrap();

            if xs.len() == 0{
                result = Ok(MalType::List(xs));
                break;
            }
            
            // xsは引数のみとなる
            let f = match self.eval(xs.remove(0)){
                Ok(v) => v,
                Err(e) => {
                    result = Err(e);
                    break;
                }
            };

            if let MalType::BuiltInFunction(func_type) = f{
                if func_type == BuiltInFunction::If{
                    ast = match self.ready_eval_if(xs){
                        Ok(v) => v,
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    }
                }else if func_type == BuiltInFunction::Let{
                    self.env.let_start();
                    n_let += 1;
                    let mut let_body = match self.ready_eval_let(xs){
                        Ok(v) => v.unwrap_sequence().unwrap(),
                        Err(e) => {
                            result = Err(e);
                            break;
                        }
                    };
                    // letの中身をdoの引数として評価する
                    let_body.insert(0,MalType::BuiltInFunction(BuiltInFunction::Do));
                    ast = MalType::List(let_body);
                }else if func_type == BuiltInFunction::Do{
                    if xs.len() == 0{
                        result = Ok(MalType::Nil);
                        break;
                    }

                    // 一番うしろのastは出しておく
                    ast = xs.pop().unwrap();

                    // 前のやつは普通に評価する
                    for x in xs{
                        if let Err(e) = self.eval(x){
                            result = Err(e);
                            break;
                        }
                    }

                    if result.is_err(){
                        break;
                    }
                }else{
                    result = self.call_built_in_function(func_type,xs);
                    break;
                }
            }else if let MalType::Function(_,_,_,_) = f{
                let (argnames,body,is_rest,local_env) = f.unwrap_function().unwrap();
                self.env.let_start();
                n_let += 1;
                ast = match self.ready_call_function(argnames,body,is_rest,xs,local_env){
                    Ok(body) => body,
                    Err(e) => {
                        result = Err(e);
                        break;
                    }
                }
            }else{
                result = Err(format!("{:?} is not callable.",f));
                break;
            }
        }

        // TODO: MalType::Functionのときにやってしまったほうが良いのでは
        //       このままだと呼び出し側の変数にアクセスできてしまうのでは
        for _ in 0..n_let{
            self.env.let_end();
        }

        result
    }

    fn eval_identifier(&mut self,ident:String)-> Result<MalType,String>{
        match self.env.get(&ident){
            Some(v) => Ok(v.clone()),
            None => Err(format!("Unknown symbol: {}",ident)),
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

    fn ready_call_function(
        &mut self,names: Vec<String>,
        body:MalType,
        is_rest:bool,
        args:Vec<MalType>,
        local_env:HashMap<String,MalType>)
        -> Result<MalType,String>{
        
        for (key,val) in local_env{
            self.env.set(key,val);
        }

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

        Ok(body)
    }

    fn call_built_in_function(&mut self,func_type:BuiltInFunction,mut xs: Vec<MalType>)
        -> Result<MalType,String>{

        match func_type{
            BuiltInFunction::Add => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_add(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Sub => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_sub(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Mul => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_mul(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Div => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_div(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::HashMap => {
                match self.eval_sequence(xs){
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
                Err(format!("It's a bug! `let` must be evaluated in eval"))
            },
            BuiltInFunction::Fn =>{
                self.mal_fn(xs)
            },
            BuiltInFunction::If =>{
                Err(format!("It's a bug! `if` must be evaluated in eval"))
            },
            BuiltInFunction::LoadFile =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function quote needs exactly 1 arguments, we got {}.",xs.len()))
                }else if let MalType::Str(filename) = xs[0].clone(){
                    self.load_file(filename)
                }else{
                    Err(format!(""))
                }
            },
            BuiltInFunction::Lt =>{
                if xs.len() != 2{
                    Err(format!(
                        "The function < needs exactly 2 arguments, we got {}."
                        ,xs.len()))
                }else{
                    match self.eval_sequence(xs){
                        Ok(ys) => mal_lt(ys),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Eq =>{
                if xs.len() != 2{
                    Err(format!(
                        "The function = needs exactly 2 arguments, we got {}."
                        ,xs.len()))
                }else{
                    match self.eval_sequence(xs){
                        Ok(ys) => mal_eq(ys),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Quote => {
                if xs.len() != 1{
                    Err(format!(
                        "The function quote needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    Ok(xs[0].clone())
                }
            },
            BuiltInFunction::Nth => {
                if xs.len() != 2{
                    return Err(format!(
                        "The function nth needs exactly 2 arguments, we got {}.",xs.len()))
                }
                let n = match self.eval(xs.pop().unwrap()){
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
                let xs = match self.eval(xs.pop().unwrap()){
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };

                mal_nth(xs,n)
            },
            BuiltInFunction::Rest => {
                if xs.len() != 1{
                    return Err(format!(
                        "The function first needs exactly 1 arguments, we got {}.",xs.len()))
                }

                match self.eval(xs.pop().unwrap()){
                    Ok(y) => mal_rest(y),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::TypeStr => {
                if xs.len() != 1{
                    return Err(format!(
                        "The function type-str needs exactly 1 arguments, we got {}.",xs.len()))
                }
                
                match self.eval(xs.pop().unwrap()){
                    Ok(y) => mal_typestr(y),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Insert => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_insert(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Eval => {
                if xs.len() != 1{
                    Err(format!(
                        "The function err needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => self.eval(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Err =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function eval needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => mal_err(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::PrintString => {
                match self.eval_sequence(xs){
                    Ok(ys) => {
                        for y in ys{
                            match  y {
                                MalType::Str(s) =>
                                    print!("{}",s),
                                _ => 
                                    return Err(format!(
                                        "The argument of print-string must be string."))
                            }
                        }
                        Ok(MalType::Nil)
                    },
                    Err(e) => Err(e),
                }
            }
            BuiltInFunction::PrStr => {
                let ys : Vec<String> = xs
                    .into_iter()
                    .map(|x| x.to_string(true))
                    .collect();
                Ok(MalType::Str(ys.join(" ")))
            },
            BuiltInFunction::Str => {
                match self.eval_sequence(xs){
                    Ok(ys) => {
                        let ys : Vec<String> = ys
                            .into_iter()
                            .map(|x| x.to_string(false))
                            .collect();
                        Ok(MalType::Str(ys.join("")))
                    },
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Apply => {
                if xs.len() != 2{
                    Err(format!(
                        "The function apply needs exactly 2 arguments, we got {}.",xs.len()))
                }else{
                    let y = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                    let f = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                    self.mal_apply(f,y)
                }
            },
            BuiltInFunction::Do => {
                Err(format!("It's a bug! `do` must be evaluated in eval"))
            },
            BuiltInFunction::Slurp =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function err needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => mal_slurp(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::ReadString => {
                if xs.len() != 1{
                    Err(format!(
                        "The function err needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => self.mal_read_string(y),
                        Err(e) => Err(e),
                    }
                }
            }
        }
    }
}

impl  Interpreter {
    
    fn ready_eval_let(&mut self,mut xs : Vec<MalType>)->Result<MalType,String>{
        if xs.len() < 2{
            Err(format!("The function let* needs at least 2 arguments, we got {}.",xs.len()))
        }else{
            let vars_ast = xs.remove(0);
            let rest_ast = xs;

            let vars = match vars_ast.unwrap_sequence(){
                Some(v)=>v,
                None => return Err(format!(
                    "The first argument of let* must be list or vector. We get {:?}.",vars_ast)),
            };

            let var_pair = match sequence_to_pair(vars){
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            for (name,val) in var_pair{
                if let Err(e) = self.mal_def(vec![name,val]){
                    return Err(e)
                }
            }
            
            Ok(MalType::List(rest_ast))
        }
    }

    fn ready_eval_if(&mut self,mut xs: Vec<MalType>)->Result<MalType,String>{
        if xs.len() != 2 && xs.len() != 3{
            return Err(format!(
                "The function if needs 1 or 2 arguments, we got {}.",xs.len()));
        }

        let cond = match self.eval(xs.remove(0)){
            Ok(v) => match v{
                MalType::Nil => false,
                MalType::Bool(b) => b,
                _ => true,
            },
            Err(e) => return Err(e),
        };

        if cond{
            Ok(xs.remove(0))
        }else{
            if xs.len() == 2{
                Ok(xs.remove(1))
            }else{
                Ok(MalType::Nil)
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

        let ast = xs[1].clone();
        let idents = ast.get_all_identifier();
        let mut local_env = HashMap::new();

        for ident in idents{
            if let Some(v) = self.env.get(&ident){
                local_env.insert(ident,v);
            }
        }

        Ok(MalType::Function(names,Box::new(ast),is_rest,local_env))
    }


    fn mal_apply(&mut self,f: MalType,mut y :MalType)->Result<MalType,String>{
        if f.unwrap_function().is_none() && f.unwrap_build_in_function().is_none() {
            return Err(format!("The first argument of apply must be function."))
        }

        if let MalType::Identifier(s) = y{
            y = match self.eval_identifier(s){
                Ok(v) => v,
                Err(e) => return Err(e),
            };
        }

        let mut xs = if let Some(v) = y.unwrap_sequence(){
            v
        }else{
            return Err(format!("The second argument of apply must be sequence."))
        };

        xs.insert(0,f);

        self.eval(MalType::List(xs))
    }

    fn mal_read_string(&mut self,x:MalType)->Result<MalType,String>{
        match x {
            MalType::Str(code) => {
                match self.read(code){
                    Ok(mut v) => {
                        if v.len() == 0 {
                            Ok(MalType::Nil)
                        }else{
                            Ok(v.remove(0))
                        }
                    },
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            _ => Err(format!(
                "The argument of read-string must be string, we got {}",
                x.to_string(true)))
        }
    }
}
