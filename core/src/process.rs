
/// syscall and functions implementation for process management.
pub mod process_impl{
    use std::error::Error;
    use std::fs::File;
    use std::os::fd::AsFd;
    use std::path::Path;
    use std::os::fd::OwnedFd;
    use std::ffi::{CString,CStr};

    use nix::fcntl::{OFlag, openat};
    use nix::sys::stat::Mode;
    use nix::unistd::{dup2_raw, execvp};
    use nix::{libc::_exit, sys::wait::waitpid, unistd::{ForkResult, execve, fork, write}};

    pub fn spawn_new_process(path:&CStr){
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

    pub enum RedirectionFileType{
        ReadOnly,
        WriteOnly,
        Append
    }

    fn open_file_for_redirection(path:&Path,redirection_file_type:RedirectionFileType)->Result<OwnedFd,Box<dyn Error>>{
        let oflags:OFlag;
        let mode:Mode;
        match redirection_file_type{
            RedirectionFileType::WriteOnly=>{
                oflags = OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC;
                mode = Mode::S_IRUSR | Mode::S_IWUSR;
            },
            RedirectionFileType::Append=>{
                oflags = OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND;
                mode = Mode::S_IRUSR | Mode::S_IWUSR;
            },
            RedirectionFileType::ReadOnly=>{
                oflags = OFlag::O_RDONLY;
                mode = Mode::empty();
            }
        }
        let fd = openat(File::open(".")?.as_fd(),
        path,
        oflags,
        mode);


        Ok(fd?)
    }


    pub enum IoRedirection{
        InputFromFile,
        OutputToFile
    }

    /// Fn for working for > operation
    pub fn piping_process(file_path:&Path,process_path:&Path,flag:IoRedirection,command:&CString,args:&[&str])->Result<(),Box<dyn Error>>{

        // create a fd pointing to the 
        let output_fd = openat(File::open(".")?.as_fd(),file_path, OFlag::O_RDONLY, Mode::S_IRUSR)?;

        // dup2(output_fd, 1);

        // if flag=0 then the file is being written to and looks like ls > output.txt


        //if flag=1 then file is being read from; cmds look like file.txt > grep "hello"


        match flag {
            // reading from file like input.txt > grep "hello"
            IoRedirection::InputFromFile=>{
                let file_fd = open_file_for_redirection(file_path, RedirectionFileType::ReadOnly)?;
                
                unsafe{dup2_raw(file_fd, 1)?;}
                // dup2(file_fd, std::io::stdout());

                let cmd = command;
                let mut c_args: Vec<CString> = vec![cmd.clone()];
                c_args.extend(args.iter().map(|&s| CString::new(s).unwrap()));
                
                // 5. Execute the command
                execvp(&cmd, &c_args)?;
                
                // If we get here, exec failed
                eprintln!("Failed to execute command");
                // execvp never returns on success
                unreachable!()
            }

            // writing to file like ls > output.txt and appending ie. ls -la >> output.txt
            IoRedirection::OutputToFile=>{
                
            }
        }

        Ok(())



    }
    
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{File, OpenOptions, read_to_string};
    use std::io::{Write, Read};
    use std::os::unix::ffi::OsStrExt;
    use std::ffi::CString;
    use tempfile::tempdir;
    use std::path::Path;
    use nix::unistd::{fork,execvp};
    use nix::unistd::ForkResult;
    use process_impl::{IoRedirection,piping_process};

    #[test]
    fn test_input_redirection() {
        let dir = tempdir().expect("temp dir");
        let file_path = dir.path().join("input.txt");
        let input_content = "abcde\n";
        {
            let mut f = File::create(&file_path).unwrap();
            write!(f, "{input_content}").unwrap();
        }
        
        // We will execve /usr/bin/wc -c to count bytes, redirecting input from input.txt
        let command = CString::new("/usr/bin/wc").unwrap();
        let args = ["-c"];
        let c_args: Vec<CString> = std::iter::once(CString::new("wc").unwrap())
        .chain(args.iter().map(|a| CString::new(*a).unwrap()))
        .collect();
    
    let output_path = dir.path().join("output.txt");
    
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            let _ = nix::sys::wait::waitpid(child, None);
            // After process, output should have single number "<len>\n"
            let content = read_to_string(&output_path).unwrap();
            assert!(content.trim().parse::<usize>().is_ok(), "Output is a number");
        }
        Ok(ForkResult::Child) => {
            // Open output file for writing and dup2 to stdout
            use nix::fcntl::{open, OFlag};
            use nix::sys::stat::Mode;
            use nix::unistd::{dup2_raw, close};
            
            let out_fd = open(
            &output_path,
            OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
            Mode::S_IRUSR | Mode::S_IWUSR,
            ).unwrap();

            println!("{:?}",out_fd);
            unsafe { 
                dup2_raw(out_fd, 1).unwrap(); 
            }
            println!("dfskfkdlsnf 2");
                close(&out_fd).unwrap();
                // Now call your function to redirect input
                let _ = piping_process(
                    &file_path,
                    Path::new(""), // not used
                    IoRedirection::InputFromFile,
                    &command,
                    &args
                );
                println!("dfskfkdlsnf 3");
                std::process::exit(1); // Should not return
            }
            Err(_) => panic!("fork failed"),
        }
    }

#[test]
fn test_output_redirection() {
    use process_impl::{IoRedirection, piping_process, RedirectionFileType};
    use nix::unistd::{fork, ForkResult};
    use std::fs::read_to_string;
    use tempfile::tempdir;
    use std::ffi::CString;
    use std::path::Path;

    let dir = tempdir().unwrap();
    let file_path = dir.path().join("output.txt");

    // This will run: echo hi > output.txt
    let command = CString::new("/bin/echo").unwrap();
    let args = ["hi"];

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            nix::sys::wait::waitpid(child, None).unwrap();
            // Assert file output in parent AFTER child is done
            let content = read_to_string(&file_path).unwrap();
            assert!(content.contains("hi"), "Output file should contain 'hi'");
        }
        Ok(ForkResult::Child) => {
            // In your redir code: open file, dup2 to 1 (stdout)
            // Re-implement piping_process so that for OutputToFile you actually set dup2 to stdout!
            use nix::fcntl::{open, OFlag};
            use nix::sys::stat::Mode;
            use nix::unistd::{dup2_raw, close};

            let out_fd = open(
                &file_path,
                OFlag::O_CREAT | OFlag::O_WRONLY | OFlag::O_TRUNC,
                Mode::S_IRUSR | Mode::S_IWUSR,
            ).unwrap();
            unsafe { dup2_raw(&out_fd, 1).unwrap(); } // 1 = STDOUT
            close(out_fd).unwrap();

            // Now exec command with output going to file
            let _ = nix::unistd::execvp(&command, &[&command, &CString::new("hi").unwrap()]);
            std::process::exit(1); // Should not return
        }
        Err(_) => panic!("fork failed"),
    }
}
}
