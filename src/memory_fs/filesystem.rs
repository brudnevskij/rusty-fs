use std::collections::HashMap;

use crate::{
    domain::{BlockId, DirectoryEntry, Inode, InodeId},
    error::{FsError, FsResult},
    memory_fs::metadata::FsMetadata,
    storage::Disk,
};

pub struct FileSystem<D: Disk> {
    fs_metadata: FsMetadata,
    inode_table: HashMap<InodeId, Inode>,
    disk: D,
}

impl<D: Disk> FileSystem<D> {
    fn new() -> FileSystem<D> {
        todo!()
    }

    fn init_fs(&mut self, super_block_id: BlockId) -> FsResult<()> {
        // init super block
        let super_block = self.disk.read_block(super_block_id)?;
        let metadata = super_block
            .try_into()
            .map_err(|_| FsError::InitError("error while initializing metadata".into()))?;
        self.fs_metadata = metadata;

        // init inode table

        todo!()
    }

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
