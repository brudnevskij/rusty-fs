use crate::{
    domain::{BlockId, DirectoryEntry},
    error::FsResult,
};

pub struct FileSystem {
    root_id: BlockId,
}

impl FileSystem {
    fn mkdir() -> FsResult<()> {
        todo!()
    }

    fn create_file() -> FsResult<()> {
        todo!()
    }

    fn read_file() -> FsResult<()> {
        todo!()
    }

    fn write_file() -> FsResult<()> {
        todo!()
    }

    fn delete_file() -> FsResult<()> {
        todo!()
    }

    fn list_directory() -> FsResult<Vec<DirectoryEntry>> {
        todo!()
    }

    fn rename() -> FsResult<()> {
        todo!()
    }
}
