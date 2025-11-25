///All sys calls implementations related to files and directories.



// #[doc(hidden)]  // hides from public docs
#[cfg(feature = "builtin_access")]
pub mod syscalls {
    use std::path::{Path, PathBuf};

    use nix::unistd::{getcwd,chdir};

    use crate::error::FsError;

    pub fn get_cwd_impl()->Result<PathBuf,FsError> {
        // println!("get_cwd syscall called!");
        // println!("cwd = {:?}", path);
        
        let path = getcwd();
        match path {
            Ok(path)=>Ok(path),
            
            Err(errno)=>{
                Err(FsError::DisplayCwdError {errno: errno })
            }
        }
    }

    pub fn change_working_dir_impl(path:&Path)->Result<(), FsError>{
        match chdir(path) {
            Ok(output_path)=>Ok(()),
            Err(errno)=>{
                Err(FsError::ChangeCwdError { errno })
            }
        }
    }


}

#[cfg(test)]
pub mod filesystem_syscallfns_tests{
    use super::*;
    use nix::errno::Errno as NixError;
    use crate::error::FsError;

    #[test]
    pub fn test_get_curr_dir() {
        let result = syscalls::get_cwd_impl();
        assert!(result.is_ok());
        let path = result.unwrap();
        println!("Current directory: {:?}", path);
        assert!(path.is_absolute());
    }

        #[test]
    pub fn test_error_display() {
        // Create a mock error to test Display output
        let mock_nix_error = NixError::EACCES;
        let fs_error = FsError::DisplayCwdError {
            errno: NixError::EACCES,
        };

        // Test Display (user-facing message)
        let display_msg = format!("{}", fs_error);
        assert_eq!(display_msg, "Failed to fetch current working directory");

        // Test Debug (detailed diagnostic)
        let debug_msg = format!("{:?}", fs_error);
        assert!(debug_msg.contains("CwdError"));
        assert!(debug_msg.contains("errno"));
        assert!(debug_msg.contains("EACCES"));

        println!("Display: {}", display_msg);
        println!("Debug: {:?}", fs_error);
    }

    #[test]
    pub fn test_error_source_chain() {
        use std::error::Error;

        let mock_nix_error = NixError::ENOTDIR;
        let fs_error = FsError::DisplayCwdError {
            errno: NixError::ENOTDIR,
        };

        // Test that source() returns the underlying NixError
        assert!(fs_error.source().is_some());

        println!("Error: {}", fs_error);
        println!("Source: {:?}", fs_error.source());
    }
}



