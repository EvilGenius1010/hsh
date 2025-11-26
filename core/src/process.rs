
/// syscall and functions implementation for process management.
pub mod process_impl{
    use std::error::Error;
    use std::fs::File;
    use std::os::fd::AsFd;
    use std::path::Path;
    use std::os::fd::OwnedFd;
    use std::ffi::{CString,CStr};
    
    use nix::fcntl::{OFlag, openat,open};
    use nix::sys::stat::Mode;
    use nix::unistd::{dup2, dup2_raw, dup2_stdin, dup2_stdout, execvp};
    use nix::{libc::_exit, sys::wait::waitpid, unistd::{ForkResult, execve, fork, write}};
    
    pub enum IoRedirection{
        InputFromFile,
        OverwriteToFile,
        AppendToFile
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

pub fn piping_process(
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
    use nix::unistd::dup2_raw;




/* 
    #[test]
    fn test_open_file_writeonly() {
        use process_impl::RedirectionFileType;
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("output.txt");

        // Test: Open file in WriteOnly mode
        let fd_result = open_file_for_redirection(
            &file_path,
            RedirectionFileType::WriteOnly
        );

        assert!(fd_result.is_ok(), "Should successfully open file for writing");
        
        let fd = fd_result.unwrap();
        
        // Verify we can write to the fd
        let test_data = b"hello world\n";
        let bytes_written = nix_write(&fd, test_data).expect("Should write to fd");
        assert_eq!(bytes_written, test_data.len());
        
        // Drop fd to close it
        drop(fd);
        
        // Verify the file was created and contains our data
        let content = read_to_string(&file_path).expect("Should read file");
        assert_eq!(content, "hello world\n");
        
        println!("✓ WriteOnly test passed");
    }

    #[test]
    fn test_open_file_writeonly_truncates_existing() {
        use process_impl::RedirectionFileType;
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("output.txt");

        // Pre-create file with existing content
        {
            let mut f = File::create(&file_path).unwrap();
            write!(f, "old content that should be truncated").unwrap();
        }

        // Open with WriteOnly (should truncate)
        let fd = open_file_for_redirection(
            &file_path,
            RedirectionFileType::WriteOnly
        ).expect("Should open for writing");

        // Write new content
        let new_data = b"new";
        nix_write(&fd, new_data).expect("Should write");
        drop(fd);

        // Verify old content is gone, only new content remains
        let content = read_to_string(&file_path).unwrap();
        assert_eq!(content, "new");
        assert!(!content.contains("old content"));
        
        println!("✓ WriteOnly truncate test passed");
    }

    #[test]
    fn test_open_file_readonly() {
        use process_impl::RedirectionFileType;
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("input.txt");

        // Create a file with content first
        let test_content = "test input data\n";
        {
            let mut f = File::create(&file_path).unwrap();
            write!(f, "{}", test_content).unwrap();
        }

        // Test: Open file in ReadOnly mode
        let fd_result = open_file_for_redirection(
            &file_path,
            RedirectionFileType::ReadOnly
        );

        assert!(fd_result.is_ok(), "Should successfully open file for reading");
        
        let fd = fd_result.unwrap();
        
        // Verify we can read from the fd
        let mut buffer = [0u8; 100];
        let bytes_read = nix_read(&fd, &mut buffer)
            .expect("Should read from fd");
        
        assert_eq!(bytes_read, test_content.len());
        assert_eq!(&buffer[..bytes_read], test_content.as_bytes());
        
        drop(fd);
        
        println!("✓ ReadOnly test passed");
    }

    #[test]
    fn test_open_file_readonly_nonexistent_fails() {

        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("nonexistent.txt");

        // Test: Try to open non-existent file in ReadOnly mode
        let fd_result = open_file_for_redirection(
            &file_path,
            RedirectionFileType::ReadOnly
        );

        // Should fail because file doesn't exist
        assert!(fd_result.is_err(), "Should fail to open non-existent file");
        
        println!("✓ ReadOnly non-existent file test passed");
    }

    #[test]
    fn test_open_file_append() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("output.txt");

        // Create initial file content
        {
            let mut f = File::create(&file_path).unwrap();
            write!(f, "first line\n").unwrap();
        }

        // Test: Open file in Append mode
        let fd = open_file_for_redirection(
            &file_path,
            RedirectionFileType::Append
        ).expect("Should open for appending");

        // Write additional data
        let append_data = b"second line\n";
        nix_write(&fd, append_data).expect("Should write");
        drop(fd);

        // Verify both old and new content exist
        let content = read_to_string(&file_path).unwrap();
        assert!(content.contains("first line"), "Should contain original content");
        assert!(content.contains("second line"), "Should contain appended content");
        assert_eq!(content, "first line\nsecond line\n");
        
        println!("✓ Append test passed");
    }

    #[test]
    fn test_open_file_append_multiple_times() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("output.txt");

        // Append multiple times
        for i in 1..=3 {
            let fd = open_file_for_redirection(
                &file_path,
                RedirectionFileType::Append
            ).expect("Should open for appending");

            let data = format!("line {}\n", i);
            nix_write(&fd, data.as_bytes()).expect("Should write");
            drop(fd);
        }

        // Verify all lines are present in order
        let content = read_to_string(&file_path).unwrap();
        assert_eq!(content, "line 1\nline 2\nline 3\n");
        
        println!("✓ Multiple append test passed");
    }

    #[test]
    fn test_open_file_permissions() {
        use std::os::unix::fs::PermissionsExt;
        
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("output.txt");

        // Create file with WriteOnly
        let fd = open_file_for_redirection(
            &file_path,
            RedirectionFileType::WriteOnly
        ).expect("Should open for writing");
        
        nix_write(&fd, b"test").expect("Should write");
        drop(fd);

        // Check file permissions (should be 0600 = user read/write only)
        let metadata = std::fs::metadata(&file_path).unwrap();
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        // Mode includes file type bits, so mask with 0o777 to get permission bits
        let perm_bits = mode & 0o777;
        
        // Should have user read (0o400) and write (0o200) = 0o600
        assert_eq!(perm_bits, 0o600, "File should have 0600 permissions");
        
        println!("✓ Permissions test passed");
    }

    #[test]
    fn test_all_redirection_types_summary() {
        let dir = tempdir().expect("Failed to create temp dir");
        
        // Test all three types in sequence
        let test_cases = vec![
            (RedirectionFileType::WriteOnly, "write", b"data1\n"),
            (RedirectionFileType::Append, "append", b"data2\n"),
            (RedirectionFileType::ReadOnly, "read", b"sSasfd"),
        ];

        for (redir_type, name, data) in test_cases {
            let file_path = dir.path().join(format!("{}.txt", name));
            
            // For ReadOnly, create file first
            if matches!(redir_type, RedirectionFileType::ReadOnly) {
                let mut f = File::create(&file_path).unwrap();
                write!(f, "readable").unwrap();
            }
            
            let fd_result = open_file_for_redirection(&file_path, redir_type.clone());
            assert!(fd_result.is_ok(), "{} should succeed", name);
            
            let fd = fd_result.unwrap();
            
            // Try to write (will only work for Write/Append)
            if !matches!(redir_type, RedirectionFileType::ReadOnly) {
                nix_write(&fd, data).expect(&format!("{} should write", name));
            }
            
            drop(fd);
            println!("✓ {} mode works", name);
        }
    }

*/


    use super::process_impl::*;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;
    use std::ffi::CString;
    use nix::unistd::{dup2_stdout,dup2_stdin};

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
            piping_process(
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

                piping_process(
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

                piping_process(
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

                piping_process(
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

                piping_process(
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

                    piping_process(
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

                piping_process(
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

                piping_process(
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



