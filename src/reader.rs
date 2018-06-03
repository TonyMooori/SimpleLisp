use interpreter::Interpreter;
use lexer::Lexer;
use token::{TokenKind,MalType,Token};

impl Interpreter{
    pub fn read(&self,code:String) -> Vec<Result<MalType,String>>{
        let mut lexer = Lexer::new(code);
        let mut result = Vec::new();

        while ! lexer.is_end(){
            result.push(self.read_form(&mut lexer));
        }

        result
    }

    fn read_form(&self,lexer : &mut Lexer) -> Result<MalType,String>{
        if let Some(token) = lexer.next(){
            match token.kind {
                TokenKind::Symbol(s) => match s.chars().nth(0).unwrap() {
                    '[' => {
                        self.read_vector(lexer)
                    },
                    '(' => {
                        self.read_list(lexer)
                    },
                    _ => {
                        Err(format!("Unexpected symbol: {} ",s))
                    }
                },

                TokenKind::Identifier(s) => {
                    Ok(
                        if s == "true"{
                            MalType::Bool(true)
                        }else if s == "false" {
                            MalType::Bool(false)
                        }else{
                            MalType::Identifier(s)
                        }
                    )
                },

                TokenKind::Str(s) 
                    => Ok(MalType::Str(s)),

                TokenKind::Integer(n) 
                    => Ok(MalType::Integer(n)),
            }
        }else{
            Err(format!("It's bug! Check it! : read_form"))
        }
    }

    fn read_sequence(&self,lexer:&mut Lexer,closer : TokenKind)->Result<Vec<MalType>,String>{
        let mut v = vec![];
        let mut flag = false;

        while let Some(token) = lexer.peek(){
            if token.kind == closer{
                lexer.next();
                flag = true;
                break;
            }

            let mt = self.read_form(lexer);

            if let Err(e) = mt{
                return Err(e);
            }else{
                v.push(mt.unwrap());
            }
        }
        
        if flag{
            Ok(v)
        }else{
            Err(format!("Cannot found close symbol: {:?}",closer))
        }
        
    }

    fn read_vector(&self,lexer:&mut Lexer) -> Result<MalType,String>{
        let closer = TokenKind::Symbol("]".to_string());

        match self.read_sequence(lexer,closer){
            Ok(v) => Ok(MalType::Vector(v)),
            Err(s) => Err(s),
        }
    }

    fn read_list(&self,lexer:&mut Lexer) -> Result<MalType,String>{
        let closer = TokenKind::Symbol(")".to_string());

        match self.read_sequence(lexer,closer){
            Ok(v) => Ok(MalType::List(v)),
            Err(s) => Err(s),
        }
    }
}
