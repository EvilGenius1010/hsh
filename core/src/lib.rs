use std::{ffi::os_str::Display, path::Path};

use crate::fs::syscalls::{change_working_dir_impl, get_cwd_impl};
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
                Err(_err)=>{
                    
                }
            }
        },
        "cd"=>{

            if tokens.args.len()!=1{
                eprintln!("Invalid Length");
                return;
                // return Err(())
            }
            // match change_working_dir_impl(tokens.args) {
            //     Ok(new_path)=>{
            //      print!("{}",new_path.)
            // }
            // }
        }
        _ =>{eprintln!("Command not found!")}
    }
}

pub fn load_startup_path()->String{
    // if hshrc file exists in /etc load variables into memory
    // if not, create the file

    match get_cwd_impl() {
        Ok(path)=>{
            path.display().to_string()
        },
        Err(err)=>{
            return err.to_string();
        }
    }





}

