

use nom::{IResult, bytes::complete::take_while1, error::Error};

/// AST node values
#[derive(Debug)]
pub enum ASTExpr<'a>{
    /// Node used for when variables are defined. First arg is key and second is value.
    DefineVar(&'a str,&'a str),

    /// Node for subsistuting specific variable.
    SubstituteVar(&'a str)
}

#[derive(Debug)]
pub enum ParserError{
    UnexpectedInput
}

// type ParserResult<'a,O> = Result<(O,&'a str),ParserError>;

// pub trait Parser<O> {
//     fn parse<'a>(&mut self,input:&'a str)->ParserResult<'a,O>;

// }




fn parse_dollar<'a>(cmd:&'a str)->IResult<&str,ASTExpr>{
    // consume characters until char is alphanumeric and 
    // if = encountered,end variable name and start consuming characters for variable value
    // if whitespace encountered, take it as a variable 

    fn is_alphanumeric(c:char)->bool{
        (c.is_ascii_alphanumeric() || c== '_') && c!= '=' && !c.is_ascii_whitespace()
    }

    let(rest,var_name) = take_while1(is_alphanumeric)(cmd)?;
    match &rest.chars().nth(0){
        Some('=') => {
            // return key-value pair
            let (remaining,var_value) = take_while1(|c:char|
                !c.is_ascii_whitespace())(&rest[1..])?;
            //  .map_err(|_| ParserError::UnexpectedInput)?;
            return Ok((remaining,ASTExpr::DefineVar(var_name,var_value)))
        },
        Some(' ') => {
            return Ok((rest,ASTExpr::SubstituteVar(var_name)))
        }
        _ =>{
            eprintln!("Invalid Input.");
            // return Err();
            return Err(nom::Err::Error(Error::new(cmd, nom::error::ErrorKind::Char)));
            
        }
    }
}