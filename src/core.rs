use types::MalType;
use std::collections::HashMap;

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

    let pairs = sequence_to_pair(xs);
    
    if let Err(e) = pairs {
        Err(e)
    }else{
        let pairs = pairs.unwrap();
        let mut hm = HashMap::new();

        for pair in pairs{
            let (x,y) = pair;

            if let MalType::Keyword(_) = x {
                hm.insert(format!("{:?}",x),y);
            }else if let MalType::Str(_) = x {
                hm.insert(format!("{:?}",x),y);
            }else{
                return Err(format!("{:?} is not supported as key of Dictonary",x));
            }

        }

        Ok(MalType::Dict(hm))
    }
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
            return Ok(MalType::Bool(v1==v2));
        }
    }

    Ok(MalType::Bool(a==b))
}

pub fn mal_first(x: MalType)->Result<MalType,String>{
    match x.unwrap_sequence(){
        Some(mut xs) => if xs.len() == 0{
            Ok(MalType::Nil)
        }else{
            Ok(xs.remove(0))
        },
        None => Err(format!(
            "The argument of first must be sequence")),
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
