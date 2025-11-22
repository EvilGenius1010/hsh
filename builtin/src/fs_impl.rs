pub mod fs_impl{
    use core::fs::syscalls::get_cwd_impl;
    use core::error::FsError;

    
    pub fn change_dir(){
            
    }

    pub fn get_cwd()->Result<std::path::PathBuf, FsError>{
        get_cwd_impl()
    }
}   