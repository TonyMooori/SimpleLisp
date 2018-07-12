use interpreter::Interpreter;
use types::{MalType,BuiltInFunction};
use core::*;
use std::process::exit;
use std::collections::HashMap;

impl Interpreter{
    pub fn eval(&mut self,mut ast:MalType)-> Result<MalType,String>{
        let mut result : Result<MalType,String> = Ok(MalType::Nil);
        let env_level = self.env.get_level();
        
        loop{
            // eprintln!("evaluating {} in eval",ast.to_string(true));
            ast = match self.mal_macroexpand(ast){
                Err(e) => return Err(e),
                Ok(v) => v,
            };

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
            }else if let MalType::Function(_,_,_,_,_) = f{
                let (argnames,body,is_rest,local_env,_) = 
                    f.unwrap_function().unwrap();
                // eprintln!("{:?}",body.to_string(true));
                self.env.let_start();
                ast = match self.ready_call_function(argnames,body,is_rest,xs,local_env,true){
                    Ok(body) => body,
                    Err(e) => {
                        result = Err(e);
                        break;
                    }
                };
            }else{
                result = Err(format!("{:?} is not callable.",f));
                break;
            }
        }

        // TODO: MalType::Functionのときにやってしまったほうが良いのでは
        //       このままだと呼び出し側の変数にアクセスできてしまうのでは
        //       根本的に何かがおかしい気はする
        while self.env.get_level() != env_level{
            self.env.let_end();
        }

        result
    }

    fn eval_identifier(&self,ident:String)-> Result<MalType,String>{
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
        local_env:HashMap<String,MalType>,
        is_eval_arg:bool)
        -> Result<MalType,String>{
        
        for (key,val) in local_env{
            self.env.set(key,val);
        }

        // evaluate arguments
        let args = if is_eval_arg {
            match self.eval_sequence(args){
                Err(e) => return Err(e),
                Ok(v) => v,
            }
        }else{
            args
        };

        if is_rest{
            if names.len() - 1 > args.len() {
                return Err(
                    format!(
                        "This function {} needs at least {} arguments, we got {}."
                        ,body.to_string(true)
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
                        "This function {} needs exactly {} arguments, we got {}."
                        ,body.to_string(true)
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
                // eprintln!("argument = {}",MalType::List(xs.clone()).to_string(false));
                match self.eval_sequence(xs){
                    Ok(ys) => {
                        // eprintln!("result = {}",MalType::List(ys.clone()).to_string(false));
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
                if xs.len() < 2{
                    Err(format!(
                        "The function apply needs at least 2 arguments, we got {}.",xs.len()))
                }else{
                    let f = match self.eval(xs.remove(0)){
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                    self.mal_apply(f,xs)
                }
            },
            BuiltInFunction::Do => {
                Err(format!("It's a bug! `do` must be evaluated in eval"))
            },
            BuiltInFunction::Slurp =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function slurp needs exactly 1 arguments, we got {}.",xs.len()))
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
                        "The function read-string needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => self.mal_read_string(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Atom =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function atom needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => self.mal_atom(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::AtomAt =>{
                if xs.len() != 1{
                    Err(format!(
                        "The function atom-at needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => mal_atom_at(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Deref => {
                if xs.len() != 1{
                    Err(format!(
                        "The function deref needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(y) => self.mal_deref(y),
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Reset => {
                if xs.len() != 2{
                    Err(format!(
                        "The function reset! needs exactly 2 arguments, we got {}.",xs.len()))
                }else{
                    let val = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                    let atom = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v,
                        Err(e) => return Err(e)
                    };
                    self.mal_reset(atom,val)
                }
            },
            BuiltInFunction::QuasiQuote => {
                if xs.len() != 1{
                    Err(format!(
                        "The function quasiquote needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    self.mal_quasiquote(xs.pop().unwrap())
                }
            },
            BuiltInFunction::SpliceUnQuote => {
                if xs.len() != 1{
                    Err(format!(
                        "The function splice-unquote needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    self.eval(xs.pop().unwrap())
                }
            },
            BuiltInFunction::UnQuote => {
                if xs.len() != 1{
                    Err(format!(
                        "The function unquote needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    self.eval(xs.pop().unwrap())
                }
            },
            BuiltInFunction::ConCat => {
                match self.eval_sequence(xs){
                    Ok(ys) => 
                        mal_concat(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::DefMacro => {
                self.mal_defmacro(xs)
            },
            BuiltInFunction::Throw => {
                if xs.len() != 1{
                    Err(format!(
                        "The function throw needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    self.error = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v,
                        Err(e) => return Err(e),
                    };
                    Err(format!("Throwed an error."))
                }
            },
            BuiltInFunction::Try => {
                if xs.len() != 2{
                    Err(format!(
                        "The function throw needs exactly 2 arguments, we got {}.",xs.len()))
                }else{
                    self.mal_try(xs)
                }
            },
            BuiltInFunction::Catch => {
                Err(format!("The function catch* must be called in try*."))
            },
            BuiltInFunction::Symbol => {
                if xs.len() != 1{
                    Err(format!(
                        "The function symbol needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(v) => if let MalType::Str(s) = v {
                            Ok(MalType::Identifier(s))
                        }else{
                            Err(format!(
                                "The argument of symbol must be string, we got {}",
                                v.to_string(false)))
                        },
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Keyword => {
                if xs.len() != 1{
                    Err(format!(
                        "The function keyword needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    match self.eval(xs.pop().unwrap()){
                        Ok(v) => if let MalType::Str(s) = v {
                            Ok(MalType::Keyword(format!(":{}", s)))
                        }else{
                            Err(format!(
                                "The argument of symbol must be string, we got {}",
                                v.to_string(false)))
                        },
                        Err(e) => Err(e),
                    }
                }
            },
            BuiltInFunction::Vector => {
                match self.eval_sequence(xs){
                    Ok(ys) => Ok(MalType::Vector(ys)),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::Assoc => {
                if xs.len() < 1{
                    Err(format!(
                        "The function assoc needs at least 1 arguments, we got {}.",xs.len()))
                }else{
                    let dic = match self.eval(xs.remove(0)){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    match dic {
                        MalType::Dict(dic) => mal_assoc(dic,xs),
                        _ => Err(format!(
                            "The first argument of assoc must be hash-map, we got {}.",
                            dic.to_string(false)))
                    }
                }
            },
            BuiltInFunction::Get => {
                if xs.len() != 2{
                    Err(format!(
                        "The function get needs exactly 2 arguments, we got {}.",xs.len()))
                }else{
                    let key = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    let dic = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    mal_get(dic,key)
                }
            },
            BuiltInFunction::Contains => {
                if xs.len() != 2{
                    Err(format!(
                        "The function contains? needs exactly 2 arguments, we got {}.",xs.len()))
                }else{
                    let key = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    let dic = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    mal_contains(dic,key)
                }
            },
            BuiltInFunction::Keys => {
                if xs.len() != 1{
                    Err(format!(
                        "The function keys needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    let dic = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    mal_keys(dic)
                }
            },
            BuiltInFunction::Vals => {
                if xs.len() != 1{
                    Err(format!(
                        "The function vals needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    let dic = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    mal_vals(dic)
                }
            },
            BuiltInFunction::Dissoc => {
                match self.eval_sequence(xs){
                    Ok(ys) => mal_dissoc(ys),
                    Err(e) => Err(e),
                }
            },
            BuiltInFunction::ReadLine => {
                if xs.len() != 1{
                    Err(format!(
                        "The function readline needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    let s = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    if let MalType::Str(s) = s{
                        println!("{}",s);
                        Ok(MalType::Str(self.read_line().trim().to_string()))
                    }else{
                        Err(format!(
                            "The first argument of readline must be string, we got {}",
                            s.to_string(true)))
                    }
                }
            },
            BuiltInFunction::Seq => {
                if xs.len() != 1{
                    Err(format!(
                        "The function seq needs exactly 1 arguments, we got {}.",xs.len()))
                }else{
                    let x = match self.eval(xs.pop().unwrap()){
                        Ok(v) => v, Err(e) => return Err(e)
                    };
                    mal_seq(x)
                }
            },
            BuiltInFunction::TimeMs => {
                if xs.len() != 0{
                    Err(format!(
                        "The function time-ms needs exactly 0 arguments, we got {}.",xs.len()))
                }else{
                    mal_time_ms()
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
                    self.env.set(ident.clone(),val.clone());
                    Ok(val)
                },

                _ =>
                    Err(format!("Cannot assign value to {:?}",sym)),
            }
        }
    }

    fn mal_defmacro(&mut self,mut xs : Vec<MalType>)->Result<MalType,String>{
        if xs.len() != 2{
            Err(format!("The function def! needs exactly 2 arguments, we got {}.",xs.len()))
        }else{
            let sym = match xs.remove(0) {
                MalType::Identifier(ident) => ident,
                _ => return Err(format!(
                        "The first argument of defmacro! must be symbol.")),
            };
            let val = match self.eval(xs.remove(0)){
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            
            match val {
                MalType::Function(varnames,body,is_rest,local_env,_) => {
                    let val = MalType::Function(
                        varnames,
                        body,
                        is_rest,
                        local_env,
                        true
                    );
                    self.env.set(sym,val.clone());
                    Ok(val)
                },
                _ => {
                    Err(format!(
                        "The argument of defmacro! must be function, we got {:?}.",
                        val
                    ))
                }
            }
        }
    }

    fn is_macro_call(&mut self,x:&MalType)->bool{
        if let MalType::List(xs) = x{
            if xs.len() == 0{
                false
            }else{
                let mut f = xs[0].clone();
                if let MalType::Identifier(s) = f{
                    f = match self.eval_identifier(s) {
                        Ok(v) => v,
                        Err(_) => return false,
                    };
                }

                if let MalType::Function(_,_,_,_,is_macro) = f{
                    is_macro
                }else{
                    false
                }
            }
        }else{
            false
        }
    }
    fn mal_macroexpand(&mut self,mut x : MalType)->Result<MalType,String>{
        // eprintln!("x = {}",x.to_string(true));
        while self.is_macro_call(&x){
            // eprintln!("we got x = {}",x.to_string(true));
            let mut xs = x.unwrap_sequence().unwrap();
            let mut f = xs.remove(0);
            if let MalType::Identifier(s) = f{
                f = match self.eval_identifier(s) {
                    Ok(v) => v,
                    Err(e) => return Err(e),
                };
            }

            let (varnames,body,is_rest,local_env,_) = 
                f.unwrap_function().unwrap();
            self.env.let_start();
            x = match self.ready_call_function(varnames,body,is_rest,xs,local_env,false){
                Ok(body) => body,
                Err(e) => return Err(e)
            };

            // eprintln!("we changed x = {}",x.to_string(true));
            x = match self.eval(x){
                Ok(v) => v,
                Err(e) => return Err(e),
            };
            // eprintln!("macroexpanded x = {}",x.to_string(true));
        }

        Ok(x)
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

        Ok(MalType::Function(names,Box::new(ast),is_rest,local_env,false))
    }


    fn mal_apply(&mut self,f: MalType,mut xs :Vec<MalType>)->Result<MalType,String>{
        if f.unwrap_function().is_none() && f.unwrap_build_in_function().is_none() {
            return Err(format!("The first argument of apply must be function."))
        }
        let mut ys = match self.eval(xs.pop().unwrap()){
            Ok(v) => 
                if v.is_list() || v.is_vector(){
                    v.unwrap_sequence().unwrap()
                }else{
                    return Err(format!(
                        "The last argument of apply must be sequence, we got {}.",
                        v.to_string(false)))
                },
            Err(e) => return Err(e),
        };

        for x in xs.into_iter().rev(){
            match self.eval(x){
                Ok(v) => ys.insert(0,v),
                Err(e) => return Err(e),
            }
        }

        // let mut ys : Vec<MalType>= ys
        //     .into_iter()
        //     .map(|y| MalType::List(vec![
        //         MalType::BuiltInFunction(BuiltInFunction::Quote),
        //         y]))
        //     .collect();

        // eprintln!("apply result = {}",MalType::List(ys.clone()).to_string(false));

        match f{
            MalType::Function(_,_,_,_,_) =>{
                let (argnames,body,is_rest,local_env,_) = 
                    f.unwrap_function().unwrap();
                // eprintln!("{:?}",body.to_string(true));
                self.env.let_start();
                let ast = match self.ready_call_function(
                    argnames,body,is_rest,ys,local_env,false){
                    Ok(body) => body,
                    Err(e) => return Err(e),
                };

                self.eval(ast)
            },
            MalType::BuiltInFunction(_) =>{
                ys.insert(0,f);
                self.eval(MalType::List(ys))
            }

            _ => Err(format!("It's bug at apply."))
        }
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

    fn mal_atom(&mut self,x:MalType)->Result<MalType,String>{
        let at = self.new_atom();
        self.set_atom(at,x);
        Ok(MalType::Atom(at))
    }

    fn mal_deref(&self,x:MalType) -> Result<MalType,String>{
        match x{
            MalType::Atom(n) => Ok(self.get_atom(n)),
            _ => Err(format!(
                "The argument of deref must be atom, we got {}",
                x.to_string(true)))
        }
    }

    fn mal_reset(&mut self,atom:MalType,val:MalType)->Result<MalType,String>{
        if let MalType::Atom(n) = atom{
            self.set_atom(n,val.clone());

            Ok(val)
        }else{
            Err(format!(
                "The first argument of reset! must be atom, we got {}",
                atom.to_string(true)))
        }
    }

    fn new_atom(&mut self)->usize{
        let at = self.atoms.len();
        self.atoms.insert(at,MalType::Nil);
        at
    }

    fn set_atom(&mut self,pos:usize,x:MalType){
        self.atoms.insert(pos,x);
    }

    fn get_atom(&self,pos:usize)->MalType{
        match self.atoms.get(&pos) {
            Some(v) => v.clone(),
            None => MalType::Nil,
        }
    }
    
    fn get_first_build_in_function(&self,xs:Vec<MalType>) -> Result<BuiltInFunction,()>{
        if xs.len() == 0{
            return Err(());
        }

        let mut x = xs[0].clone();

        if let MalType::Identifier(s) = x{
            x = match self.eval_identifier(s){
                Ok(v) => v,
                Err(_) => return Err(()),
            };
        }
        if let MalType::BuiltInFunction(f) = x{
            Ok(f)
        }else{
            Err(())
        }
    }

    fn mal_quasiquote(&mut self,x: MalType) -> Result<MalType,String>{
        match self.inner_quasiquote(x){
            Ok(v) => Ok(v.0),
            Err(e) => Err(e),
        }
    }

    fn inner_quasiquote(&mut self,x: MalType) -> Result<(MalType,bool),String>{
        if ! x.is_sequence(){
            return Ok((x,false));
        }

        let is_list = x.is_list();
        let xs = x
            .unwrap_sequence()
            .unwrap();
        let mut ys = vec![];

        if is_list{
            let res_f = self.get_first_build_in_function(xs.clone());
            
            if res_f == Ok(BuiltInFunction::UnQuote){
                return match self.eval(MalType::List(xs)){
                    Ok(v) => Ok((v,false)),
                    Err(e) => Err(e),
                };
            }else if res_f == Ok(BuiltInFunction::SpliceUnQuote) {
                return match self.eval(MalType::List(xs)){
                    Ok(v) => Ok((v,true)),
                    Err(e) => Err(e)
                };
            }
        }

        for x in xs.into_iter(){
            let (x,f) = match self.inner_quasiquote(x){
                Ok(v) => v,
                Err(e) => return Err(e),
            };

            if f && x.is_sequence(){
                ys.append(&mut x.unwrap_sequence().unwrap());
            }else{
                ys.push(x);
            }
        }

        Ok((MalType::List(ys),false))
    }

    fn mal_try(&mut self,mut xs:Vec<MalType>)->Result<MalType,String>{
        let err_str = match self.eval(xs.remove(0)){
            Ok(v) => return Ok(v),
            Err(e) => e,
        };

        let err_val = if err_str == "Throwed an error."{
            self.error.clone()
        }else{
            MalType::Str(err_str)
        };

        let xs = xs.remove(0);
        if ! xs.is_list() {
            return Err(format!(
                "The second argument of try* must be catch* function call."))
        }

        let mut xs = xs.unwrap_sequence().unwrap();
        if xs.len() != 3{
            return Err(format!(
                "The second argument of try* is (catch* err-var result). We got {}.",
                MalType::List(xs).to_string(false)));
        }

        let f = match self.eval(xs.remove(0)){
            Ok(v) => v,
            Err(e) => return Err(e),
        };
        let varname = xs.remove(0);
        let result = xs.remove(0);

        if f != MalType::BuiltInFunction(BuiltInFunction::Catch){
            return Err(format!(
                "The second argument of try* is (catch* err-var result). "));
        }

        if varname.unwrap_identifier().is_none() {
            return Err(format!(
                "The first argument of catch* must be identifier, we got {}.",
                varname.to_string(false)));
        }
        
        self.env.let_start();
        self.env.set(varname.unwrap_identifier().unwrap(), err_val);

        let result = self.eval(result);

        self.env.let_end();

        result
    }
}
