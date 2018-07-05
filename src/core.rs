use types::MalType;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

fn to_integer_vec(xs: Vec<MalType>)->Result<Vec<i64>,String>{
    let mut v = Vec::new();

    for x in xs{
        if let MalType::Integer(n) = x{
            v.push(n);
        }else{
            return Err(format!("Expected integer, found {:?}.",x));
        }
    }

    Ok(v)
}

pub fn sequence_to_pair(xs: Vec<MalType>)->Result<Vec<(MalType,MalType)>,String>{
    // (:a "s" :b "d")->[(:a "s"),(:b "d")]
    let mut v = Vec::new();

    if xs.len()%2 == 1{
        return Err(format!("Expected an even number of arguments, we got odd number of them."));
    }

    for i in 0..xs.len()/2{
        let x = xs[2*i+0].clone();
        let y = xs[2*i+1].clone();

        v.push((x,y));
    }

    Ok(v)
}

pub fn mal_add(xs: Vec<MalType>)->Result<MalType,String>{
    let xs = match to_integer_vec(xs){
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let mut result = 0;

    for x in xs{
        result += x;
    }

    Ok(MalType::Integer(result))
}

pub fn mal_sub(xs: Vec<MalType>)->Result<MalType,String>{
    let xs = match to_integer_vec(xs){
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if xs.len() == 0{
        Err(format!("Wrong number of argument(0)."))
    }else if xs.len() == 1{
        Ok(MalType::Integer(-xs[0]))
    }else{
        let mut result = xs[0];
        
        for x in xs[1..].into_iter(){
            result -= x;
        }

        Ok(MalType::Integer(result))
    }
}

pub fn mal_mul(xs: Vec<MalType>)->Result<MalType,String>{
    let xs = match to_integer_vec(xs){
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let mut result = 1;

    for x in xs{
        result *= x;
    }

    Ok(MalType::Integer(result))
}

pub fn mal_div(xs: Vec<MalType>)->Result<MalType,String>{
    let xs = match to_integer_vec(xs){
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if xs.len() == 0{
        Err(format!("Wrong number of argument(0)."))
    }else if xs.len() == 1{
        if xs[0] == 0{
            Err(format!("Divided by zero."))
        }else{
            Ok(MalType::Integer(0))
        }
    }else{
        let mut result = xs[0];
        
        for x in xs[1..].into_iter(){
            if *x == 0{
                return Err(format!("Divided by zero."));
            }else{
                result /= x;
            }
        }

        Ok(MalType::Integer(result))
    }
}

pub fn mal_hashmap(xs: Vec<MalType>)->Result<MalType,String>{
    // eprintln!("{:?} is mal_hashmap",xs);
    mal_assoc(HashMap::new(), xs)
}

pub fn mal_lt(xs: Vec<MalType>)->Result<MalType,String>{
    let xs = match to_integer_vec(xs){
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    
    Ok(MalType::Bool(xs[0]<xs[1]))
}

pub fn mal_eq(mut xs: Vec<MalType>)->Result<MalType,String>{
    let a = xs.pop().unwrap();
    let b = xs.pop().unwrap();

    if let Some(v1) = a.unwrap_sequence(){
        if let Some(v2) = b.unwrap_sequence(){
            if v1.len() != v2.len(){
                return Ok(MalType::Bool(false));
            }

            for i in 0..v1.len(){
                let a = v1[i].clone();
                let b = v2[i].clone();

                // 一つずつ確認していって一つでも間違ってたらfalse
                match mal_eq(vec![a,b]){
                    Ok(v) => if !v.unwrap_bool().unwrap(){
                        return Ok(MalType::Bool(false));
                    },
                    _ => {},
                }
            }
            return Ok(MalType::Bool(true));
        }
    }

    Ok(MalType::Bool(a==b))
}

pub fn mal_nth(xs: MalType,n:MalType)->Result<MalType,String>{
    let n = match n.unwrap_integer(){
        Some(v) => v,
        None => return Err(format!("The second argument of nth must be integer.")),
    };
    let xs = match xs.unwrap_sequence(){
        Some(v) => v,
        None => return Err(format!("The first argument of nth must be sequence."))
    };
    if n < 0{
        return Err(format!(
            "The second argument of nth must be 0 or positive number, we got {}.",n));
    }
    
    let n = n as usize;

    if xs.len() > n{
        Ok(xs[n].clone())
    }else{
        Err(format!(
            "The index is out of bounds."))
    }
}

pub fn mal_rest(x: MalType)->Result<MalType,String>{
    if let MalType::Vector(mut v) = x{
        if v.len() == 0 {
            Ok(MalType::Vector(Vec::new()))
        }else{
            v.remove(0);
            Ok(MalType::Vector(v))
        }
    }else if let MalType::List(mut v)=x{
        if v.len() == 0 {
            Ok(MalType::List(Vec::new()))
        }else{
            v.remove(0);
            Ok(MalType::List(v))
        }
    }else{
        Err(format!("The argument of rest must be sequence"))
    }
}

pub fn mal_typestr(x:MalType)->Result<MalType,String>{
    Ok(MalType::Str(match x{
        MalType::Identifier(_) => "symbol",
        MalType::Integer(_) => "int",
        MalType::Str(_) => "str",
        MalType::Bool(_) => "bool",
        MalType::Vector(_) => "vector",
        MalType::List(_) => "list",
        MalType::Function(_,_,_,_,_) => "func",
        MalType::BuiltInFunction(_) => "built-in-func",
        MalType::Keyword(_) => "keyword",
        MalType::Dict(_) => "dict",
        MalType::Atom(_) => "atom",
        MalType::Nil => "nil",
    }.to_string()))
}

pub fn mal_insert(mut xs:Vec<MalType>)->Result<MalType,String>{
    if xs.len() != 3{
        Err(format!(
            "The function insert needs exactly 3 arguments, we got {}.",xs.len()))
    }else{
        let element = xs.pop().unwrap();
        let index = match xs.pop().unwrap(){
            MalType::Integer(n) => n,
            a => return Err(format!(
                "The second argument of insert must be integer, we got {:?}.",a))
        };
        if index < 0 {
            return Err(format!(
                "The index is must be positive, get {}.",index))
        };
        let index = index as usize;
        let ys = xs.pop().unwrap();
        let is_list = match ys{
            MalType::List(_) => true,
            MalType::Vector(_) => false,
            a => return Err(format!(
                "The first argument of insert must be sequence, we got {:?}.",a))
        };
        let mut ys = ys.unwrap_sequence().unwrap();
        if ys.len() < index {
            return Err(format!(
                "The index must be little than the length."));
        }

        ys.insert(index,element);

        if is_list{
            Ok(MalType::List(ys))
        }else{
            Ok(MalType::Vector(ys))
        }
    }
}

pub fn mal_err(x:MalType)->Result<MalType,String>{
    match x{
        MalType::Str(s) => 
            Err(s),
        _ =>
            Err(format!("The argument of err function must be string"))
    }
}


pub fn mal_slurp(x:MalType) -> Result<MalType,String> {
    match x{
        MalType::Str(filename) => {
            let file = match File::open(filename.clone()){
                Ok(v) => v,
                Err(_) => return Err(format!("Cannot open file {}.",filename)),
            };
            let mut buf_reader = BufReader::new(file);
            let mut code = String::new();
            match buf_reader.read_to_string(&mut code){
                Ok(_) => 
                    Ok(MalType::Str(code)),
                Err(_) => 
                    Err(format!("Cannot read file {}.",filename)),
            }
        },
        _ => {
            Err(format!("The argument of slurp function must be string"))
        }
    }
}

pub fn mal_atom_at(x:MalType)-> Result<MalType,String> {
    match x {
        MalType::Integer(n) => Ok(MalType::Atom(n as usize)),
        _ => Err(format!(
            "The argument of atom-at must be integer, we got {}",
            x.to_string(true))),
    }
}

pub fn mal_concat(xs:Vec<MalType>) -> Result<MalType,String>{
    let mut ys = vec![];

    for x in xs{
        if let Some(mut v) = x.unwrap_sequence(){
            ys.append(&mut v);
        }else{
            return Err(format!(
                "The argument of concat must be sequence, we got {}",
                x.to_string(true)));
        }
    }

    Ok(MalType::List(ys))
}

pub fn mal_assoc(mut hm: HashMap<String,MalType>,xs:Vec<MalType>)->Result<MalType,String>{
    let pairs = sequence_to_pair(xs);
    
    if let Err(e) = pairs {
        Err(e)
    }else{
        let pairs = pairs.unwrap();

        for pair in pairs{
            let (x,y) = pair;

            if let MalType::Keyword(k) = x {
                hm.insert(k.clone(),y);
            }else if let MalType::Str(s) = x {
                hm.insert(format!(" {}",s),y);
            }else{
                return Err(format!("{:?} is not supported as key of Dictonary",x));
            }

        }

        Ok(MalType::Dict(hm))
    }
}

pub fn mal_get(dic:MalType,key:MalType)->Result<MalType,String>{
    if let MalType::Dict(dic) = dic{
        if let MalType::Str(key) = key{
            return match dic.get(&format!(" {}",key)){
                Some(v) => Ok(v.clone()),
                None => Ok(MalType::Nil)
            };
        }else if let MalType::Keyword(key) = key{
            return match dic.get(&key){
                Some(v) => Ok(v.clone()),
                None => Ok(MalType::Nil)
            };
        }
    }

    Ok(MalType::Nil)
}

pub fn mal_contains(dic:MalType,key:MalType)->Result<MalType,String>{
    if let MalType::Dict(dic) = dic{
        if let MalType::Str(key) = key{
            return match dic.get(&format!(" {}",key)){
                Some(_) => Ok(MalType::Bool(true)),
                None => Ok(MalType::Bool(false)),
            };
        }else if let MalType::Keyword(key) = key{
            return match dic.get(&key){
                Some(_) => Ok(MalType::Bool(true)),
                None => Ok(MalType::Bool(false)),
            };
        }
    }

    Ok(MalType::Bool(false))
}

pub fn mal_keys(dic:MalType)->Result<MalType,String>{
    if let MalType::Dict(dic) = dic{
        let mut xs = vec![];

        for (key,_) in &dic{
            let mut key = key.clone();
            let key = if key.chars().nth(0).unwrap() == ' '{
                key.remove(0);
                MalType::Str(key)
            }else{
                MalType::Keyword(key)
            };
            xs.push(key);
        }

        Ok(MalType::List(xs))
    }else{
        Err(format!(
            "The argument of key must be hash-map, we got {}",
            dic.to_string(false)))
    }
}

pub fn mal_vals(dic:MalType)->Result<MalType,String>{
    if let MalType::Dict(dic) = dic{
        let mut xs = vec![];

        for (_,val) in &dic{
            xs.push(val.clone());
        }

        Ok(MalType::List(xs))
    }else{
        Err(format!(
            "The argument of vals must be hash-map, we got {}",
            dic.to_string(false)))
    }
}

pub fn mal_dissoc(mut xs:Vec<MalType>)->Result<MalType,String>{
    if xs.len() == 0 {
        return Err(format!(
            "The function dissoc needs at least one argument, we got 0."));
    }

    let mut dic = match xs.remove(0){
        MalType::Dict(dic) => dic,
        v => return Err(format!(
            "The first argument of dissoc must be dictonary, we got {}."
            ,v.to_string(false)))
    };

    for x in xs{
        let mut key = match x{
            MalType::Keyword(key) => key,
            MalType::Str(s) => format!(" {}",s),
            _ => continue,
        };

        dic.remove(&key);
    }

    Ok(MalType::Dict(dic))
}