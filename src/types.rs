
#[derive(PartialEq, Debug, Clone)]
pub enum TokenKind{
    Identifier(String), // def!,inc,dec,+,-,...
    Integer(i64), 
    Symbol(String),     // [],(),{},`,',@,~,~@,^
    Str(String),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Token{
    pub kind : TokenKind,
    pub start : usize,
    pub end : usize,
}

#[derive(PartialEq, Debug, Clone)]
pub enum MalType{
    Identifier(String),
    Integer(i64),
    Str(String),
    Bool(bool),
    Vector(Vec<MalType>),
    List(Vec<MalType>),
    Function(Vec<MalType>),
    Error(String),
    Keyword(String),
    Nil,
}
