use thiserror::Error;

#[derive(Debug, Error)]
pub enum FsError {
    #[error("disk error {0}")]
    DiskError(#[from] DiskError),
}

pub type FsResult<T> = Result<T, FsError>;

#[derive(Error, Debug)]
pub enum DiskError {
    #[error("disk has no space")]
    NoSpace,
}

pub type DiskResult<T> = Result<T, DiskError>;
