use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiskError {
    #[error("disk has no space")]
    NoSpace,
}

pub type DiskResult<T> = Result<T, DiskError>;
