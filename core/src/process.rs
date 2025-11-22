/* 
/// syscall and functions implementation for process management.
pub mod process_impl{
    use std::error::Error;
    use std::os::fd::AsRawFd;
    use std::fs::File;
    use std::path::PathBuf;
    use std::{os::fd::OwnedFd, process};
    use std::ffi::CStr;

    use nix::unistd::dup2;
    use nix::{libc::_exit, sys::wait::waitpid, unistd::{ForkResult, execve, fork, pipe, write}};

    pub fn spawn_new_process(path:&CStr,args:&CStr,env:&CStr){
        match unsafe {fork()}{
            Ok(ForkResult::Parent { child,..}) =>{
                println!("Continuing execution in parent process, new child has pid: {}", child);
                waitpid(child, None).unwrap();
            }
            Ok(ForkResult::Child) => {
            // Unsafe to use `println!` (or `unwrap`) here. See Safety.
             // Safe I/O (no println!)
            let _ = write(std::io::stdout(), b"Child: execve starting\n");

            // argv = [program name only]
            let argv = [path];

            // empty env
            let env: [&CStr; 0] = [];

            match execve(path, &argv, &env) {
                Ok(_) => unreachable!(),
                Err(err) => {
                    let msg = format!("execve failed: {}\n", err);
                    let _ = write(std::io::stderr(), msg.as_bytes());
                    unsafe { _exit(1) };
                }
            }
            }
            Err(_) => println!("Fork failed"),
        }

    }
    pub fn pipe_process(writer:&mut OwnedFd,reader:&mut OwnedFd)->Result<(),Error>{
        let (read_end,write_end) = pipe().unwrap();
        dup2(write_end, writer)?;
        dup2(read_end, reader)?;
        Ok(())
    }
    
    
    // use std::os::raw::c_char;
    // fn make_cstring(data:&CStr){

    // unsafe extern "C" fn work_with(s: *const c_char) {}
    // unsafe { work_with(data.as_ptr()) }

    // }


    
}

#[cfg(test)]
mod process_syscall_tests{
    use std::process::Command;

    use crate::process::process_impl::{pipe_process, spawn_new_process};

    use super::*;

    #[test]
    fn test_spawn_new_process(){
        let args = c"1\n2\n3";
        let path = c"/Users/harshavardhankolhatkar04/Desktop/Projects/Projects to learn Tech/Learn_Rust/hsh/validation_trash/a";
        let env = c"";
        spawn_new_process(path,args,env);
    }

    #[test]
    fn pipe_echo_run(){
        // let ls = Command::new("ls -l");
        // let grep = Command::new("grep \".toml\"");
        // pipe_process(ls, grep);
        
    }
}



    */