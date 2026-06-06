use std::collections::HashMap;

use crate::{
    domain::{BlockId, DirectoryEntry, Inode, InodeId},
    error::{FsError, FsResult},
    memory_fs::metadata::FsMetadata,
    storage::{BLOCK_SIZE, Block, Disk},
};

pub struct FileSystem<D: Disk> {
    fs_metadata: FsMetadata,
    inode_table: InodeTable,
    disk: D,
}

type InodeTable = HashMap<InodeId, Inode>;

fn parse_inode_table(inode_table_blocks: Vec<Block>) -> InodeTable {
    todo!()
}

struct BlockReader {
    blocks: Vec<Block>,
    block_idx: usize,
    position: usize,
}

impl BlockReader {
    fn new(blocks: Vec<Block>) -> BlockReader {
        BlockReader {
            blocks,
            block_idx: 0,
            position: 0,
        }
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self
            .blocks
            .get(self.block_idx)?
            .data
            .get(self.position)
            .copied();

        self.position += 1;
        if self.position >= BLOCK_SIZE {
            self.position = 0;
            self.block_idx += 1;
        }

        byte
    }

    pub fn read_until(&mut self, delimiter: u8) -> Vec<u8> {
        let mut buffer = Vec::new();
        while let Some(b) = self.next_byte() {
            if b == delimiter {
                break;
            }
            buffer.push(b);
        }

        buffer
    }
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
        let mut inode_table_blocks =
            Vec::with_capacity(self.fs_metadata.inode_table_blocks as usize);

        let start = self.fs_metadata.inode_table_start;
        let end = start + self.fs_metadata.inode_table_blocks;
        for block_id in start..end {
            let block = self.disk.read_block(block_id)?;
            inode_table_blocks.push(block);
        }

        let inode_table = parse_inode_table(inode_table_blocks);

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

#[cfg(test)]
mod test {
    use crate::{memory_fs::filesystem::BlockReader, storage::Block};

    #[test]
    fn correct_inode_entry_streaming() {
        let mut data = [0u8; 4096];
        data[..4].copy_from_slice(b"123;");
        let block = Block { id: 1, data: data };

        let mut block_reader = BlockReader::new(vec![block]);
        let buff = block_reader.read_until(b';');
        assert_eq!(buff, b"123");
    }

    #[test]
    fn correct_cross_block_inode_entry_streaming() {
        let mut data_1 = [0u8; 4096];
        let mut data_2 = [0u8; 4096];
        data_1[4094..].copy_from_slice(b";d");
        data_2[..4].copy_from_slice(b"ata;");

        let mut block_reader = BlockReader::new(vec![
            Block {
                id: 0,
                data: data_1,
            },
            Block {
                id: 1,
                data: data_2,
            },
        ]);
        let buff = block_reader.read_until(b';');
        assert_eq!(buff.len(), 4094);

        let buff = block_reader.read_until(b';');
        assert_eq!(buff, b"data");
    }

    #[test]
    fn reads_multiple_entries_from_same_stream() {
        let mut data = [0u8; 4096];
        data[..8].copy_from_slice(b"123;456;");

        let block = Block { id: 1, data };

        let mut block_reader = BlockReader::new(vec![block]);

        assert_eq!(block_reader.read_until(b';').as_slice(), b"123");
        assert_eq!(block_reader.read_until(b';').as_slice(), b"456");
    }
}
