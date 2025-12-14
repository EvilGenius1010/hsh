/// AST node values
pub enum ASTExpr{
    /// Node used for when variables are defined. First arg is key and second is value.
    DefineVar(String,String),

    /// Node for subsistuting specific variable.
    SubstituteVar(String)
}
pub enum ParserError{
    
}

type ParserResult<'a,O> = Result<(O,&'a str),ParserError>;

pub trait Parser<O> {
    fn parse<'a>(&mut self,input:&'a str)->ParserResult<'a,O>;

}




fn parse_dollar(){
    
}