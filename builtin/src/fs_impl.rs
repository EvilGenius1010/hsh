pub mod fs_impl{
    use core::fs::syscalls::{change_working_dir_impl, get_cwd_impl};
    use core::error::FsError;
    use std::path::Path;

    
    pub fn change_dir(path:&Path)->Result<(), FsError>{
        change_working_dir_impl(path)

    }

    pub fn get_cwd()->Result<std::path::PathBuf, FsError>{
        get_cwd_impl()
    }


}   