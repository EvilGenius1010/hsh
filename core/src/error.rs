use nix::errno::Errno;
use derive_more::{Debug, Display, Error};

#[derive(Debug, Display, Error)]
pub enum FsError {
    #[debug("CwdError(errno={errno:?},")]
    #[display("Failed to fetch current working directory")]
    DisplayCwdError {
        #[error(source)]
        // #[from]
        errno: Errno,
    },
    
    #[debug("CwdError(errno={errno:?},")]
    #[display("Failed to fetch current working directory")]
    ChangeCwdError{
        // #[error(source)]
        // #[from]
        errno: Errno,
    },
    #[display("Filesystem error: {_0}")]
    Other(#[error(not(source))] String),
}
