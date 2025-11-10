use std::path::Path;
mod tokenizer;

#[cfg(feature = "builtin_access")]
pub mod fs;

#[cfg(not(feature = "builtin_access"))]
pub(crate) mod fs;  // still available internally

pub struct TokenizedOutput<'a>{
    pub command:&'a str,
    pub args:Vec<&'a str>
}

pub fn match_expression(tokens:TokenizedOutput){
    match tokens.command {
        "echo"=>{println!("{}",tokens.args.concat())},
        "exit"=>{
            println!("bye");
            std::process::exit(0);
        }
        _ =>{eprintln!("Command not found!")}
    }
}

pub fn load_system_variables(){
    // if hshrc file exists in /etc load variables into memory
    if Path::new("/etc.txt").exists(){

    }


    // if not, create the file


}

