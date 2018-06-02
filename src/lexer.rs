use token::{TokenKind,Token};

pub struct Lexer{
    source : String,
    index : usize,
}

fn is_symbol_char(c:char)->bool{
    c == '~'
    || c == '@'
    || c == '`'
    || c == '\''
    || c == '^'
    || c == '['
    || c == ']'
    || c == '('
    || c == ')'
    || c == '{'
    || c == '}'
    || c == '{'
    || c == '}'
}

impl Lexer{
    pub fn new(source:String)->Lexer{
        Lexer{
            source : source,
            // 次のトークンの最初の文字のインデックス
            index : 0 ,
        }
    }

    fn next_char(&self) -> Option<char> {
        self.source.chars().nth(self.index+1)
    }

    fn current_char(&self) -> Option<char> {
        self.source.chars().nth(self.index)
    }

    fn skip_while(&mut self,f : &Fn(char) -> bool){
        while let Some(c) = self.current_char(){
            if ! f(c) {
                break;
            }
            self.index += 1;
        }
    }

    fn skip_whitespace(&mut self){
        self.skip_while(
            &|c| c.is_whitespace() || c == ','
        );
    }
}

impl Lexer{
    pub fn read_next_token(&mut self)->Option<Token>{
        self.skip_whitespace();
        
        if let Some(c) = self.current_char(){
            match c{
                c if is_symbol_char(c)
                    => Some(self.read_symbol()),
                c if c.is_numeric() 
                    => Some(self.read_integer()),
                ';' => {
                    self.skip_comment();
                    self.read_next_token()
                },
                '\"'=> Some(self.read_string()),
                 _  => Some(self.read_identifier()),
            }
        }else{
            None
        }
    }

    fn skip_comment(&mut self){
        self.skip_while(&|c| c != '\n');
    }

    fn read_string(&mut self) -> Token{
        let start = self.index;
        let mut result = String::new();
        let mut backslash = false;
        let mut end = false;

        self.index += 1;
        while let Some(c) = self.current_char(){
            if backslash{
                result.push(
                    match c {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        c => c,
                    }
                );
            }else if c == '\\'{
                backslash = true;
            }else if c == '\"'{
                end = true;
            }else{
                result.push(c);
            }

            self.index += 1;
            if end {
                break;
            }
        }

        if !end{
            panic!("Unexpedted end of code: found EOF while reading literal");
        }

        Token{
            kind : TokenKind::Str(result),
            start : start,
            end : self.index,
        }
    }

    fn read_identifier(&mut self)->Token{
        let start = self.index;
        self.skip_while(&|c| 
            !is_symbol_char(c)
            && c != '\"'
            && !c.is_whitespace()
            && c != ','
            && c != ';');
        let token_str = self.source[start..self.index].to_string();

        Token{
            kind : TokenKind::Identifier(token_str),
            start : start,
            end : self.index,
        }
    }
    
    fn read_integer(&mut self)->Token{
        let start = self.index;
        self.skip_while(&|c| c.is_numeric());
        let token_str = self.source[start..self.index].to_string();
        let num : i64 = token_str.parse().unwrap();

        Token{
            kind : TokenKind::Integer(num),
            start : start,
            end : self.index,
        }
    }

    fn read_symbol(&mut self)->Token{
        let start = self.index;
        let c = self.current_char().unwrap();
        let symbol_str = if c == '~' && self.next_char() == Some('@'){
            String::from("~@")
        }else{
            format!("{}",c)
        };

        self.index += symbol_str.len();
        Token{
            kind : TokenKind::Symbol(symbol_str),
            start : start,
            end : self.index,
        }
    }
}

#[test]
fn test_lexer_0(){
    let src = "(def! gensym (fn* [] ; This is comment \n (symbol (str \"G__\" (swap! *gensym-counter* (fn* [x] (+ 1 x)))))))".to_string();    
    let mut lexer = Lexer::new(src);
    let token_list = vec![
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("def!".to_string()),
        TokenKind::Identifier("gensym".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("fn*".to_string()),
        TokenKind::Symbol("[".to_string()),
        TokenKind::Symbol("]".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("symbol".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("str".to_string()),
        TokenKind::Str("G__".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("swap!".to_string()),
        TokenKind::Identifier("*gensym-counter*".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("fn*".to_string()),
        TokenKind::Symbol("[".to_string()),
        TokenKind::Identifier("x".to_string()),
        TokenKind::Symbol("]".to_string()),
        TokenKind::Symbol("(".to_string()),
        TokenKind::Identifier("+".to_string()),
        TokenKind::Integer(1),
        TokenKind::Identifier("x".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
        TokenKind::Symbol(")".to_string()),
    ];

    for (idx,kind) in token_list.into_iter().enumerate(){
        eprintln!("current idx is {}",idx);
        assert_eq!(lexer.read_next_token().unwrap().kind,kind);
    }

    assert_eq!(lexer.read_next_token().is_some(),false);
}