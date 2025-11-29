
/// syscall and functions implementation for process management.
pub mod process_impl{
    use std::error::Error;
    use std::path::Path;
    use std::os::fd::OwnedFd;
    use std::ffi::{CString,CStr};
    
    use nix::fcntl::{OFlag, open};
    use nix::sys::stat::Mode;
    use nix::unistd::{ dup2_stdin, dup2_stdout, execvp, pipe};
    use nix::{libc::_exit, sys::wait::waitpid, unistd::{ForkResult, execve, fork, write}};

    use crate::tokenizer::ShellTokens;

    pub enum IoRedirection{
        InputFromFile,
        OverwriteToFile,
        AppendToFile
    }
    
    pub struct ProcessToExec<'a>{
        
        /// name of running process
        process_name:String,
        
        /// path of binary of process
        process_path:&'a Path,

        /// arguments passed to the command
        process_args:Vec<&'a str>,
        
        /// The order in which processes are arranged
        order:u16


    }



    pub fn spawn_new_process(path:&CStr){
        match unsafe {fork()}{
            Ok(ForkResult::Parent { child,..}) =>{
                println!("Continuing execution in parent process, new child has pid: {}", child);
                waitpid(child, None).unwrap();
            }
            Ok(ForkResult::Child) => {
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

    pub fn spawn_and_pipe(prev_pipe:Option<OwnedFd>,curr_index:usize,curr_process_item:&ProcessToExec)->Result<Option<OwnedFd>,Box<dyn Error>>{
        match unsafe{fork()} {
            Ok(ForkResult::Parent { child })=>{
                waitpid(child, None);
                Ok(None)
            }
            Ok(ForkResult::Child)=>{
                let c_string_args:Vec<CString> = curr_process_item.process_args.iter()
                .map(|s| CString::new(*s).unwrap())
                .collect();
                
                let c_str_args: Vec<&CStr> = c_string_args
                .iter()
                .map(|s| s.as_c_str())
                .collect();

            
                // match pipe(){
                //     Ok(pipe_ends)=>{
                //         let (receive_end_pipe,send_end_pipe) = (pipe_ends.0,pipe_ends.1);
                //             dup2_stdin(send_end_pipe)?;
                //         }
                //         Err(errno)=>{
                //             println!("{}",errno);
                            
                //         }
                // }
                let (receive_end_pipe,send_end_pipe) = pipe()?;
                dup2_stdout(send_end_pipe)?;
                if curr_index != 0{

                    if let Some(val) = prev_pipe{
                        dup2_stdin(val)?;
                    }
                    // match prev_pipe{
                    //     Some(val)=>dup2_stdin(val),
                    //     None => {
                    //         eprintln!("Error!");
                    //         Ok(())
                    //     }
                        
                    // }
                }


                
                    

                execvp(CString::new(curr_process_item.process_path.to_str().expect("Failed to convert Cstring!"))?.as_c_str(),
                &c_str_args);
                    
                Ok(Some(receive_end_pipe))
            }
            Err(err)=>{
                println!("{}",err);
                Ok(None)
            }
        }
    }


    // fn to perform piping
    pub fn perform_piping(process_list:Vec<ProcessToExec>)->Result<(),Box<dyn Error>>{
        let process_iterator = process_list.iter();

        let mut prev_pipe:Option<OwnedFd> = None;
        //use fork to 
        for (index,item) in process_iterator.enumerate(){
            let next_pipe = spawn_and_pipe(prev_pipe,index,item)?;
                prev_pipe = next_pipe;
        }
        Ok(())
    }



    #[derive(Clone)]
    pub enum RedirectionFileType{
        ReadOnly,
        WriteOnly,
        Append
    }


pub fn open_file_for_redirection(
    path: &Path,
    redirection_file_type: RedirectionFileType
) -> Result<OwnedFd, Box<dyn Error>> {
    let (oflags, mode) = match redirection_file_type {
        RedirectionFileType::WriteOnly => {
            (OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_TRUNC,
             Mode::S_IRUSR | Mode::S_IWUSR)
        },
        RedirectionFileType::Append => {
            (OFlag::O_WRONLY | OFlag::O_CREAT | OFlag::O_APPEND,
             Mode::S_IRUSR | Mode::S_IWUSR)
        },
        RedirectionFileType::ReadOnly => {
            (OFlag::O_RDONLY, Mode::empty())
        }
    };
    
    let fd = open(path, oflags, mode)?;
    Ok(fd)
}

pub fn redirect_process(
    file_path: &Path,
    flag: IoRedirection,
    command: &CString,
    args: &[&str]
) -> Result<(), Box<dyn Error>> {

    match flag {
        // for input.txt > process 
        IoRedirection::InputFromFile => {
            // let stdin = stdin();
            // dup(&stdin()).unwrap();
            let file_fd = open_file_for_redirection(file_path, RedirectionFileType::ReadOnly)?;
            dup2_stdin(&file_fd)?;
            // unsafe { dup2_raw(&file_fd, 0)?; }
            // drop(file_fd);

            let mut c_args: Vec<CString> = vec![command.clone()];
            c_args.extend(args.iter().map(|&s| CString::new(s).unwrap()));
            
            execvp(command, &c_args)?;
            unreachable!()
        }
        // for process > output.txt
        IoRedirection::OverwriteToFile => {
            let file_fd = open_file_for_redirection(file_path, RedirectionFileType::WriteOnly)?;
            dup2_stdout(&file_fd)?;
            // unsafe { dup2_raw(&file_fd, 1)?; }
            // drop(file_fd);

            let mut c_args: Vec<CString> = vec![command.clone()];
            c_args.extend(args.iter().map(|&s| CString::new(s).unwrap()));
            
            execvp(command, &c_args)?;
            unreachable!()
        }
        // for process >> output.txt and it appends
        IoRedirection::AppendToFile => {
            let file_fd = open_file_for_redirection(file_path, RedirectionFileType::Append)?;
            dup2_stdout(&file_fd)?;
            // unsafe { dup2_raw(&file_fd, 1)?; }
            // drop(file_fd);

            let mut c_args: Vec<CString> = vec![command.clone()];
            c_args.extend(args.iter().map(|&s| CString::new(s).unwrap()));
            
            execvp(command, &c_args)?;
            unreachable!()
        }
    }
}











    
}

#[cfg(test)]
mod syscall_tests{


    
    
    use crate::process::process_impl::{open_file_for_redirection,RedirectionFileType};

    use super::*;
    use std::fs::{File, read_to_string};
    use std::io::Write;
    use tempfile::tempdir;









    use super::process_impl::*;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;
    use std::ffi::CString;
    use nix::unistd::dup2_stdout;

#[test]
fn test_input_from_file_cat() {
    let dir = tempdir().expect("tempdir failed");
    let input_path = dir.path().join("input.txt");
    let output_path = dir.path().join("output.txt");  // ← ADD THIS

    // Create input
    {
        let mut f = File::create(&input_path).unwrap();
        write!(f, "hello from file\n").unwrap();
    }

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).unwrap();
            
            // ✅ Verify output file
            let content = read_to_string(&output_path).unwrap();
            assert_eq!(content, "hello from file\n");
        }
        Ok(ForkResult::Child) => {
            // ✅ REDIRECT STDOUT TO FILE FIRST!
            let out_fd = open_file_for_redirection(
                &output_path, 
                RedirectionFileType::WriteOnly
            ).unwrap();
            dup2_stdout(&out_fd).unwrap();
            drop(out_fd);
            
            // NOW redirect stdin and run cat
            let command = CString::new("/bin/cat").unwrap();
            redirect_process(
                &input_path,
                IoRedirection::InputFromFile,
                &command,
                &[]
            ).unwrap();

            std::process::exit(1);
        }
        Err(_) => panic!("fork failed"),
    }
}

    #[test]
    fn test_input_from_file_grep() {
        // Test: grep "hello" < input.txt
        
        let dir = tempdir().expect("tempdir failed");
        let input_path = dir.path().join("input.txt");

        // Create input with multiple lines
        {
            let mut f = File::create(&input_path).unwrap();
            write!(f, "hello world\nfoo bar\nhello again\n").unwrap();
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();
                println!("✓ InputFromFile grep test completed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/usr/bin/grep").unwrap();
                let args = &["hello"];

                redirect_process(
                    &input_path,
                    IoRedirection::InputFromFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }

    #[test]
    fn test_overwrite_to_file() {
        // Test: echo "test message" > output.txt
        
        let dir = tempdir().expect("tempdir failed");
        let output_path = dir.path().join("output.txt");

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();

                // Verify output file was created and has content
                let content = read_to_string(&output_path).unwrap();
                assert!(content.contains("test message"));
                println!("✓ OverwriteToFile test passed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/bin/echo").unwrap();
                let args = &["test", "message"];

                redirect_process(
                    &output_path,
                    IoRedirection::OverwriteToFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }

    #[test]
    fn test_overwrite_truncates_existing_file() {
        // Test that > truncates existing content
        
        let dir = tempdir().expect("tempdir failed");
        let output_path = dir.path().join("output.txt");

        // Pre-create file with old content
        {
            let mut f = File::create(&output_path).unwrap();
            write!(f, "OLD CONTENT THAT SHOULD BE REMOVED\n").unwrap();
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();

                // Verify old content is gone
                let content = read_to_string(&output_path).unwrap();
                assert!(!content.contains("OLD CONTENT"));
                assert!(content.contains("NEW"));
                println!("✓ Overwrite truncates test passed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/bin/echo").unwrap();
                let args = &["NEW"];

                redirect_process(
                    &output_path,
                    IoRedirection::OverwriteToFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }

    #[test]
    fn test_append_to_file() {
        // Test: echo "line2" >> output.txt (preserves existing content)
        
        let dir = tempdir().expect("tempdir failed");
        let output_path = dir.path().join("output.txt");

        // Create initial content
        {
            let mut f = File::create(&output_path).unwrap();
            write!(f, "line1\n").unwrap();
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();

                // Verify both old and new content exist
                let content = read_to_string(&output_path).unwrap();
                assert!(content.contains("line1"));
                assert!(content.contains("line2"));
                assert_eq!(content, "line1\nline2\n");
                println!("✓ AppendToFile test passed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/bin/echo").unwrap();
                let args = &["line2"];

                redirect_process(
                    &output_path,
                    IoRedirection::AppendToFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }

    #[test]
    fn test_append_multiple_times() {
        // Test multiple >> operations preserve all content
        
        let dir = tempdir().expect("tempdir failed");
        let output_path = dir.path().join("output.txt");

        // Append 3 times in sequence
        for i in 1..=3 {
            match unsafe { fork() } {
                Ok(ForkResult::Parent { child, .. }) => {
                    waitpid(child, None).unwrap();
                }
                Ok(ForkResult::Child) => {
                    let command = CString::new("/bin/echo").unwrap();
                    let arg = format!("line{}", i);
                    let args = &[arg.as_str()];

                    redirect_process(
                        &output_path,
                        IoRedirection::AppendToFile,
                        &command,
                        args
                    ).unwrap();

                    std::process::exit(1);
                }
                Err(_) => panic!("fork failed"),
            }
        }

        // Verify all lines are present
        let content = read_to_string(&output_path).unwrap();
        assert_eq!(content, "line1\nline2\nline3\n");
        println!("✓ Multiple append test passed");
    }

    #[test]
    fn test_ls_to_file() {
        // Test: ls > output.txt (real-world command)
        
        let dir = tempdir().expect("tempdir failed");
        let output_path = dir.path().join("output.txt");

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();

                // Verify output file has ls output
                let content = read_to_string(&output_path).unwrap();
                assert!(!content.is_empty(), "ls should produce output");
                println!("✓ ls redirection test passed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/bin/ls").unwrap();
                let args = &["-la"];

                redirect_process(
                    &output_path,
                    IoRedirection::OverwriteToFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }

    #[test]
    fn test_wc_from_file() {
        // Test: wc -l < input.txt (count lines)
        
        let dir = tempdir().expect("tempdir failed");
        let input_path = dir.path().join("input.txt");

        // Create file with known number of lines
        {
            let mut f = File::create(&input_path).unwrap();
            write!(f, "line1\nline2\nline3\n").unwrap();
        }

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child, .. }) => {
                waitpid(child, None).unwrap();
                println!("✓ wc input redirection test passed");
            }
            Ok(ForkResult::Child) => {
                let command = CString::new("/usr/bin/wc").unwrap();
                let args = &["-l"];

                redirect_process(
                    &input_path,
                    IoRedirection::InputFromFile,
                    &command,
                    args
                ).unwrap();

                std::process::exit(1);
            }
            Err(_) => panic!("fork failed"),
        }
    }
}



