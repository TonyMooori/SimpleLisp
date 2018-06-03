use interpreter::Interpreter;
use lexer::Lexer;
use types::{TokenKind,MalType,Token};

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
        let otoken = lexer.peek();

        if otoken.is_none(){
            return Err(format!("It's a bug! See read_form."))
        }

        let token = otoken.unwrap();

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
    }

    fn read_sequence(&self,lexer:&mut Lexer,start : TokenKind,end : TokenKind)->Result<Vec<MalType>,String>{
        let mut v = vec![];
        let mut flag = false;

        // read left bracket
        if lexer.next().unwrap().kind != start {
            return Err(format!("It's a bug! See read_sequence."));
        }

        while let Some(token) = lexer.peek(){
            // read right bracket
            if token.kind == end{
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
            Err(format!("Cannot found close symbol: {:?}",end))
        }
        
    }

    fn read_vector(&self,lexer:&mut Lexer) -> Result<MalType,String>{
        let start = TokenKind::Symbol("[".to_string());
        let end = TokenKind::Symbol("]".to_string());

        match self.read_sequence(lexer,start,end){
            Ok(v) => Ok(MalType::Vector(v)),
            Err(s) => Err(s),
        }
    }

    fn read_list(&self,lexer:&mut Lexer) -> Result<MalType,String>{
        let start = TokenKind::Symbol("(".to_string());
        let end = TokenKind::Symbol(")".to_string());


        match self.read_sequence(lexer,start,end){
            Ok(v) => Ok(MalType::List(v)),
            Err(s) => Err(s),
        }
    }
}
