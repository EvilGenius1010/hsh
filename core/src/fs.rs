///All sys calls implementations related to files and directories.


#[doc(hidden)]  // hides from public docs
#[cfg(feature = "builtin_access")]
pub mod syscalls {
    use nix::unistd;

    pub fn get_cwd_impl() {
        println!("get_cwd syscall called!");
        let path = unistd::getcwd().unwrap();
        println!("cwd = {:?}", path);
    }
}





