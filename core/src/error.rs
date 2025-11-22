use nix::errno::Errno;
use derive_more::{Debug, Display, Error,From};

#[derive(Debug, Display, Error,From)]
pub enum FsError {
    #[debug("CwdError(errno={errno:?},")]
    #[display("Failed to fetch current working directory")]
    CwdError {
        #[error(source)]
        #[from]
        errno: Errno,
    },

    #[display("Filesystem error: {_0}")]
    Other(#[error(not(source))] String),
}
