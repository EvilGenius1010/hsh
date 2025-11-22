use std::path::{Path, PathBuf};

use crate::fs::syscalls::get_cwd_impl;
mod tokenizer;
mod process;
pub mod error;



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
        },
        "pwd"=>{
            match get_cwd_impl() {
                Ok(path)=>{
                    println!("{:?}",path.as_path())
                },
                Err(err)=>{
                    
                }
            }
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

