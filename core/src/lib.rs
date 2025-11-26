use std::{ffi::os_str::Display, path::Path};

use crate::fs::syscalls::{change_working_dir_impl, get_cwd_impl};
mod tokenizer;
mod process;
pub mod error;

use std::ffi::CString;
use crate::process::process_impl::{piping_process, IoRedirection};



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
        },
        // "check"=>{
        //     println!("Running interactive tests...\n");
        //     run_interactive_tests(&tokens.args);
        // }
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



/* 
/// Run interactive tests based on arguments
fn run_interactive_tests(args: &[&str]) {
    if args.is_empty() {
        println!("Available tests:");
        println!("  check cat       - Test: cat < input.txt");
        println!("  check echo      - Test: echo 'hello' > output.txt");
        println!("  check append    - Test: echo 'more' >> output.txt");
        println!("  check ls        - Test: ls > output.txt");
        println!("  check grep      - Test: grep 'pattern' < input.txt");
        return;
    }

    match args[0] {
        "cat" => test_cat_redirect(),
        "echo" => test_echo_redirect(),
        "append" => test_append_redirect(),
        "ls" => test_ls_redirect(),
        "grep" => test_grep_redirect(),
        _ => println!("Unknown test: {}", args[0]),
    }
}

/// Test: cat < input.txt
fn test_cat_redirect() {
    use std::fs::File;
    use std::io::Write;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;

    println!("=== Testing: cat < input.txt ===");
    
    // Create test input file
    let input_path = Path::new("test_input.txt");
    {
        let mut f = File::create(input_path).expect("Failed to create test file");
        writeln!(f, "Hello from input file!").expect("Failed to write");
        writeln!(f, "Line 2").expect("Failed to write");
    }
    println!("✓ Created test_input.txt");

    // Fork and test
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("✓ Forked child process (pid: {})", child);
            let status = waitpid(child, None).expect("waitpid failed");
            println!("✓ Child exited with status: {:?}", status);
            
            // Cleanup
            std::fs::remove_file(input_path).ok();
            println!("✓ Cleaned up test file\n");
        }
        Ok(ForkResult::Child) => {
            println!("Child: Running cat with stdin from file...");
            
            let command = CString::new("/bin/cat").unwrap();
            if let Err(e) = piping_process(
                input_path,
                IoRedirection::InputFromFile,
                &command,
                &[]
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            
            std::process::exit(0); // Should never reach
        }
        Err(e) => {
            eprintln!("Fork failed: {}", e);
        }
    }
}

/// Test: echo "hello world" > output.txt
fn test_echo_redirect() {
    use std::fs::read_to_string;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;

    println!("=== Testing: echo 'hello world' > output.txt ===");
    
    let output_path = Path::new("test_output.txt");
    
    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            println!("✓ Forked child process (pid: {})", child);
            waitpid(child, None).expect("waitpid failed");
            
            // Read and display output
            match read_to_string(output_path) {
                Ok(content) => {
                    println!("✓ Output file contents:");
                    println!("  {}", content.trim());
                }
                Err(e) => eprintln!("✗ Failed to read output: {}", e),
            }
            
            // Cleanup
            std::fs::remove_file(output_path).ok();
            println!("✓ Cleaned up test file\n");
        }
        Ok(ForkResult::Child) => {
            let command = CString::new("/bin/echo").unwrap();
            if let Err(e) = piping_process(
                output_path,
                IoRedirection::OverwriteToFile,
                &command,
                &["hello", "world"]
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Err(e) => eprintln!("Fork failed: {}", e),
    }
}

/// Test: echo "more text" >> output.txt
fn test_append_redirect() {
    use std::fs::{File, read_to_string};
    use std::io::Write;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;

    println!("=== Testing: echo 'more' >> output.txt ===");
    
    let output_path = Path::new("test_output.txt");
    
    // Create initial file
    {
        let mut f = File::create(output_path).expect("Failed to create file");
        writeln!(f, "Initial line").expect("Failed to write");
    }
    println!("✓ Created file with initial content");

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).expect("waitpid failed");
            
            match read_to_string(output_path) {
                Ok(content) => {
                    println!("✓ Output file contents:");
                    for line in content.lines() {
                        println!("  {}", line);
                    }
                }
                Err(e) => eprintln!("✗ Failed to read: {}", e),
            }
            
            std::fs::remove_file(output_path).ok();
            println!("✓ Cleaned up\n");
        }
        Ok(ForkResult::Child) => {
            let command = CString::new("/bin/echo").unwrap();
            if let Err(e) = piping_process(
                output_path,
                IoRedirection::AppendToFile,
                &command,
                &["Appended", "line"]
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Err(e) => eprintln!("Fork failed: {}", e),
    }
}

/// Test: ls -la > output.txt
fn test_ls_redirect() {
    use std::fs::read_to_string;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;

    println!("=== Testing: ls -la > output.txt ===");
    
    let output_path = Path::new("test_output.txt");

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).expect("waitpid failed");
            
            match read_to_string(output_path) {
                Ok(content) => {
                    println!("✓ Output file created, {} bytes", content.len());
                    println!("  First few lines:");
                    for line in content.lines().take(5) {
                        println!("    {}", line);
                    }
                }
                Err(e) => eprintln!("✗ Failed to read: {}", e),
            }
            
            std::fs::remove_file(output_path).ok();
            println!("✓ Cleaned up\n");
        }
        Ok(ForkResult::Child) => {
            let command = CString::new("/bin/ls").unwrap();
            if let Err(e) = piping_process(
                output_path,
                IoRedirection::OverwriteToFile,
                &command,
                &["-la"]
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Err(e) => eprintln!("Fork failed: {}", e),
    }
}

/// Test: grep "pattern" < input.txt
fn test_grep_redirect() {
    use std::fs::File;
    use std::io::Write;
    use nix::unistd::{fork, ForkResult};
    use nix::sys::wait::waitpid;

    println!("=== Testing: grep 'hello' < input.txt ===");
    
    let input_path = Path::new("test_input.txt");
    
    // Create test file
    {
        let mut f = File::create(input_path).expect("Failed to create file");
        writeln!(f, "hello world").expect("Failed");
        writeln!(f, "goodbye world").expect("Failed");
        writeln!(f, "hello again").expect("Failed");
    }
    println!("✓ Created test file with sample data");

    match unsafe { fork() } {
        Ok(ForkResult::Parent { child, .. }) => {
            waitpid(child, None).expect("waitpid failed");
            std::fs::remove_file(input_path).ok();
            println!("✓ Cleaned up\n");
        }
        Ok(ForkResult::Child) => {
            let command = CString::new("/usr/bin/grep").unwrap();
            if let Err(e) = piping_process(
                input_path,
                IoRedirection::InputFromFile,
                &command,
                &["hello"]
            ) {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
            std::process::exit(0);
        }
        Err(e) => eprintln!("Fork failed: {}", e),
    }
}

*/