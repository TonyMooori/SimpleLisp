
#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind{
    Identifier(String), // def!,inc,dec,+,-,...
    Integer(i64), 
    Symbol(String),     // [],(),{},`,',@,~,~@,^
    Str(String),

}

pub struct Token{
    pub kind : TokenKind,
    pub start : usize,
    pub end : usize,
}
